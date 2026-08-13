/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// input_loader.rs — reads CSV/TSV desire lines and builds RawTrajectories
use super::super::geometry::input_od_line::InputODLine;
use super::super::geometry::point::Point;
use super::super::geometry::trajectory::Trajectory;
use super::super::storage::raw_trajectories::RawTrajectories;
use crate::ui::args::InputHeaderField;
use crate::ui::args::MappingHeader;
use crate::ui::args::TraclusArgs;
use crate::ui::args_config::get_param_configs;
use crate::utils::events::app_events::AppError;
use crate::utils::events::event_singleton::emit_error;

use std::fs;
use std::io;
use std::io::Error;
use std::io::ErrorKind::InvalidData;
use std::path::Path;
use std::str::FromStr;

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

// Returns the CSV column index corresponding to each InputHeaderField.
// If the CSV has no header, or if no mapping is provided: indexes are inferred from the number of columns
// If a header and a mapping are provided: the mapped fields are searched in the header
fn get_header_mapping_indexes(
    first_line: &str,
    mapping: &MappingHeader,
) -> io::Result<Vec<Option<usize>>> {
    let is_header: bool = is_header(first_line);

    let num_columns: usize = first_line.split(detect_separator(first_line)).count();
    let num_mapping_fields: usize = mapping.iter().filter(|s: &&String| !s.is_empty()).count();

    // Default column order from field count
    let default_indexes: Vec<Option<usize>> = InputHeaderField::default_indexes(num_columns);

    // No header or incomplete map — use defaults
    if !is_header || num_mapping_fields < get_param_configs().num_fields_map.min {
        return Ok(default_indexes);
    }

    let mut mapping_indexes: Vec<Option<usize>> = InputHeaderField::empty_mapping();
    let sep: char = detect_separator(first_line);
    let header_fields: Vec<String> = first_line
        .split(sep)
        .map(|s| s.trim().to_lowercase())
        .collect();

    // Replace the inferred indexes with the positions found in the header.
    for i in 0..mapping.len() {
        let mapped_name: String = mapping[i].clone();

        if mapped_name.is_empty() {
            continue;
        }

        // Resolve mapped name to header column index
        let new_index: Option<usize> = header_fields.iter().position(|s| *s == mapped_name);

        if new_index.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Field '{mapped_name}' is not found in the header: '{first_line}\n'"),
            ));
        }

        mapping_indexes[i] = new_index;
    }
    Ok(mapping_indexes)
}

// Header if any field is non-numeric
fn is_header(line: &str) -> bool {
    let sep: char = detect_separator(line);
    line.split(sep)
        .any(|field: &str| field.trim().parse::<f64>().is_err())
}

// Detects whether the line uses tabs, commas, or semicolons as separator
fn detect_separator(line: &str) -> char {
    if line.contains('\t') {
        '\t'
    } else if line.contains(';') {
        ';'
    } else {
        ','
    }
}

// Parses a string into a type T, returning an io::Error if parsing fails
fn parse_to_type<T: FromStr>(s: &str, line_number: usize, field_name: &str) -> io::Result<T> {
    s.parse::<T>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Failed to parse {} at line {}: '{}'",
                field_name, line_number, s
            ),
        )
    })
}

// Parses one input row into an InputODLine
#[inline]
fn parse_line_to_od(
    line: &str,
    header_indexes: &[Option<usize>],
    index_line: usize,
) -> io::Result<InputODLine> {
    let sep: char = detect_separator(line);
    let parts: Vec<&str> = line.split(sep).map(|p: &str| p.trim()).collect();

    let name_index: usize = header_indexes[InputHeaderField::Name as usize].unwrap_or(0);
    let name: &str = parts.get(name_index).unwrap_or(&"");

    let weight_index: usize =
        header_indexes[InputHeaderField::Weight as usize].ok_or_else(|| {
            Error::new(
                InvalidData,
                format!(
                    "Weight field is missing in the mapping for line {}: '{}'",
                    index_line, line
                ),
            )
        })?;
    let weight: u32 = parse_to_type::<u32>(parts[weight_index], index_line, "weight")?;

    let x_start_index: usize =
        header_indexes[InputHeaderField::XOrigin as usize].ok_or_else(|| {
            Error::new(
                InvalidData,
                format!(
                    "x_origin field is missing in the mapping for line {}: '{}'",
                    index_line, line
                ),
            )
        })?;
    let x_start: f64 = parse_to_type::<f64>(parts[x_start_index], index_line, "x_start")?;

    let y_start_index: usize =
        header_indexes[InputHeaderField::YOrigin as usize].ok_or_else(|| {
            Error::new(
                InvalidData,
                format!(
                    "y_origin field is missing in the mapping for line {}: '{}'",
                    index_line, line
                ),
            )
        })?;
    let y_start: f64 = parse_to_type::<f64>(parts[y_start_index], index_line, "y_start")?;

    let x_end_index: usize = header_indexes[InputHeaderField::XDest as usize].ok_or_else(|| {
        Error::new(
            InvalidData,
            format!(
                "x_dest field is missing in the mapping for line {}: '{}'",
                index_line, line
            ),
        )
    })?;
    let x_end: f64 = parse_to_type::<f64>(parts[x_end_index], index_line, "x_end")?;

    let y_end_index: usize = header_indexes[InputHeaderField::YDest as usize].ok_or_else(|| {
        Error::new(
            InvalidData,
            format!(
                "y_dest field is missing in the mapping for line {}: '{}'",
                index_line, line
            ),
        )
    })?;
    let y_end: f64 = parse_to_type::<f64>(parts[y_end_index], index_line, "y_end")?;

    Ok(InputODLine {
        name: name.to_string(),
        line_id: index_line,
        weight,
        start: Point {
            x: x_start,
            y: y_start,
        },
        end: Point { x: x_end, y: y_end },
    })
}

// Loads and segments all valid OD lines while skipping zero-length lines
pub fn parse_input_data(args: &TraclusArgs) -> Option<RawTrajectories> {
    let mut trajectory_storage: RawTrajectories = RawTrajectories::new(args.max_angle);

    // Read file or emit error
    let content: String = match read_file(&args.file) {
        Ok(c) => c,
        Err(err) => {
            emit_error(AppError::IoError(format!(
                "Failed to read input file: {}",
                err
            )));
            return None;
        }
    };

    // Resolve header column mapping
    let first_line: &str = content.lines().next().unwrap_or("");
    let header_indexes: Vec<Option<usize>> = match get_header_mapping_indexes(first_line, &args.map)
    {
        Ok(indexes) => indexes,
        Err(err) => {
            emit_error(AppError::IoError(format!(
                "Failed to get header mapping indexes: {}",
                err
            )));
            return None;
        }
    };

    // Parse body lines into trajectories
    let mut number_point_lines: i32 = 0;
    let mut trajectory_id: usize = 0;
    for (index, line) in content.lines().enumerate() {
        let line: &str = line.trim();

        if line.is_empty() {
            continue;
        }

        if index == 0 && is_header(line) {
            continue;
        }

        let od_line: InputODLine = match parse_line_to_od(line, &header_indexes, trajectory_id) {
            Ok(od) => od,
            Err(err) => {
                emit_error(AppError::IoError(format!("{}", err)));
                return None;
            }
        };

        if od_line.start == od_line.end {
            number_point_lines += 1;
            continue;
        }

        let trajectory: Trajectory = Trajectory::new(od_line, args.segment_size);
        trajectory_storage.add_trajectory(trajectory);
        trajectory_id += 1;
    }

    // Emit a warning if any lines were ignored due to being points
    // This is not a fatal error, but it may indicate an issue with the input data
    if number_point_lines > 0 {
        emit_error(AppError::IoError(format!(
            "WARNING: {} lines were ignored because they represent points (start and end are the same).\n\
            Consider removing these lines from the input file.",
            number_point_lines
        )));
    }

    Some(trajectory_storage)
}
