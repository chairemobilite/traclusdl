/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// args.rs — CLI argument types, parsing, and TraclusArgs definition

use crate::{
    ui::args_config::{AllArgsConfigs, get_param_configs},
    utils::data_types::angle_u16::AngleU16,
};
use clap::{Parser, ValueEnum};
use std::{fmt, hash::Hash};

// ─────────────────────────────────────────────
// ExecutionMode  — algorithm parallelism strategy
// ─────────────────────────────────────────────

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
pub enum ExecutionMode {
    Serial,
    ParallelRayon,
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionMode::Serial => write!(f, "Serial"),
            ExecutionMode::ParallelRayon => write!(f, "ParallelRayon"),
        }
    }
}

fn default_mode() -> ExecutionMode {
    ExecutionMode::ParallelRayon
}

// ─────────────────────────────────────────────
// InterfaceMode  — type of way to interact with the program
// ─────────────────────────────────────────────

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
pub enum InterfaceMode {
    Logger,
    PerfTimer,
    Performance,
}

impl fmt::Display for InterfaceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterfaceMode::Logger => write!(f, "Logger"),
            InterfaceMode::PerfTimer => write!(f, "PerfTimer"),
            InterfaceMode::Performance => write!(f, "Performance"),
        }
    }
}

fn default_interface_mode() -> InterfaceMode {
    InterfaceMode::Performance
}

// ─────────────────────────────────────────────
// Input header mapper - maps input header names to internal field names
// ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingHeader(pub Vec<String>);

impl MappingHeader {
    pub fn new() -> Self {
        Self(vec![String::new(); InputHeaderField::COUNT])
    }

    pub fn get_value(&self, field: InputHeaderField) -> &str {
        &self.0[field as usize]
    }

    pub fn set_value(&mut self, field: InputHeaderField, value: String) {
        self.0[field as usize] = value;
    }
}

impl Default for MappingHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for MappingHeader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_mapping(s)
    }
}

impl std::ops::Deref for MappingHeader {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum InputHeaderField {
    Name = 0, // (Optional)
    Weight = 1,
    XOrigin = 2,
    YOrigin = 3,
    XDest = 4,
    YDest = 5,
}

impl InputHeaderField {
    pub const COUNT: usize = 6;
    pub const NB_OPTIONAL_FIELDS: usize = 1;

    pub fn from_index(i: usize) -> Option<Self> {
        INPUT_HEADER_FIELDS.get(i).copied()
    }

    // Infers column indexes from column count when header/mapping absent
    pub fn default_indexes(num_columns: usize) -> Vec<Option<usize>> {
        let mut indexes: Vec<Option<usize>> = vec![None; Self::COUNT];

        let missing_optional = Self::COUNT.saturating_sub(num_columns);

        for (csv_index, field) in INPUT_HEADER_FIELDS
            .iter()
            .skip(missing_optional)
            .enumerate()
        {
            indexes[*field as usize] = Some(csv_index);
        }

        indexes
    }

    // All fields unmapped
    pub fn empty_mapping() -> Vec<Option<usize>> {
        vec![None; Self::COUNT]
    }
}

// Rust does not have built-in enum reflection
const INPUT_HEADER_FIELDS: [InputHeaderField; InputHeaderField::COUNT] = [
    InputHeaderField::Name,
    // All optional fields must be at the beginning of the enum
    InputHeaderField::Weight,
    InputHeaderField::XOrigin,
    InputHeaderField::YOrigin,
    InputHeaderField::XDest,
    InputHeaderField::YDest,
];

// Parses "field:column,…" map string into MappingHeader
fn parse_mapping(s: &str) -> Result<MappingHeader, String> {
    let lower_cased: String = s.to_lowercase();
    let cleaned: String = lower_cased
        .trim()
        .replace(['{', '}', '[', ']', ' ', '\'', '"', '\t', '\n', '\r'], "");

    let mut mapping = MappingHeader::new();

    if cleaned.is_empty() {
        return Ok(mapping);
    }

    for entry in cleaned.split(',') {
        let (field, column) = entry
            .split_once(':')
            .ok_or_else(|| format!("Expected FIELD:COLUMN in '{entry}'"))?;

        let field = match field {
            "name" => InputHeaderField::Name,
            "weight" => InputHeaderField::Weight,
            "xorigin" => InputHeaderField::XOrigin,
            "yorigin" => InputHeaderField::YOrigin,
            "xdest" => InputHeaderField::XDest,
            "ydest" => InputHeaderField::YDest,
            _ => {
                return Err(format!(
                    "Field '{field}' is not accepted. All accepted fields: {:?}",
                    INPUT_HEADER_FIELDS
                ));
            }
        };

        mapping.set_value(field, column.to_string());
    }

    Ok(mapping)
}

// ─────────────────────────────────────────────
// TraclusArgs
// ─────────────────────────────────────────────

#[derive(Clone, Parser, Debug, PartialEq)]
#[command(author, version, about = "Traclus DL Optimized in Rust")]
pub struct TraclusArgs {
    #[arg(short = 'f', long = "file", default_value = "")]
    pub file: String,

    #[arg(short = 'o', long = "output", default_value = "")]
    pub output: String,

    #[arg(
        short = 's',
        long = "segment_size",
        default_value_t = get_param_configs().segment_size.default,
        value_parser = |v: &str| {
            let cfg = get_param_configs().segment_size;
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < cfg.min || val > cfg.max {
                Err(format!("segment_size must be in range {}..={}", cfg.min, cfg.max))
            } else {
                Ok(val)
            }
        }
    )]
    pub segment_size: f64,

    #[arg(
        short = 'a',
        long = "max_angle",
        default_value_t = AngleU16::from_degrees(get_param_configs().max_angle.default),
        value_parser = |v: &str| {
            let cfg = get_param_configs().max_angle;
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < cfg.min || val > cfg.max {
                Err(format!("max_angle must be in range {}..={}", cfg.min, cfg.max))
            } else {
                let val_angle: AngleU16 = AngleU16::from_degrees(val);
                Ok(val_angle)
            }
        }
    )]
    pub max_angle: AngleU16,

    #[arg(
        short = 'd',
        long = "max_dist",
        default_value_t = get_param_configs().max_dist.default,
        value_parser = |v: &str| {
            let cfg = get_param_configs().max_dist;
            let val: f64 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < cfg.min || val > cfg.max {
                Err(format!("max_dist must be in range {}..={}", cfg.min, cfg.max))
            } else {
                Ok(val)
            }
        }
    )]
    pub max_dist: f64,

    #[arg(
        short = 'n',
        long = "min_density",
        default_value_t = get_param_configs().min_density.default,
        value_parser = |v: &str| {
            let cfg = get_param_configs().min_density;
            let val: u32 = v.parse().map_err(|_| String::from("must be a number"))?;
            if val < cfg.min || val > cfg.max {
                Err(format!("min_density must be in range {}..={}", cfg.min, cfg.max))
            } else {
                Ok(val)
            }
        }
    )]
    pub min_density: u32,

    #[arg(
        long = "map",
        default_value = "",
        value_parser = |v: &str| {
            let cfg = get_param_configs().num_fields_map;
            let mapping: MappingHeader = parse_mapping(v).map_err(|e| format!("Invalid map: {}", e))?;
            let val: usize = mapping.iter().filter(|s| !s.is_empty()).count() as usize;
            if (val < cfg.min || val > cfg.max) && val != 0 {
                Err(format!("num_fields_map must be in range {}..={}", cfg.min, cfg.max))
            } else {
                Ok(mapping)
            }
        },
        help = "Mapping of input header fields to CSV columns"
    )]
    pub map: MappingHeader,

    #[arg(short = 't', long = "max_threads", default_value_t = get_param_configs().max_threads.default, value_parser = |v: &str| {
        let cfg = get_param_configs().max_threads;
        let val: u32 = v.parse().map_err(|_| String::from("must be a number"))?;
        if val < cfg.min || val > cfg.max {
            Err(format!("max_threads must be in range {}..={}", cfg.min, cfg.max))
        } else {
            Ok(val)
        }
    })]
    pub max_threads: u32,

    #[arg(short = 'm', long = "mode", value_enum, default_value_t = default_mode())]
    pub mode: ExecutionMode,

    #[arg(short = 'i', long = "interface", value_enum, default_value_t = default_interface_mode())]
    pub interface_mode: InterfaceMode,
}

impl Default for TraclusArgs {
    fn default() -> Self {
        let cfg: AllArgsConfigs = get_param_configs();
        Self {
            file: String::new(),
            output: String::new(),
            max_dist: cfg.max_dist.default,
            min_density: cfg.min_density.default,
            max_angle: AngleU16::from_degrees(cfg.max_angle.default),
            segment_size: cfg.segment_size.default,
            map: MappingHeader::new(),
            max_threads: cfg.max_threads.default,
            mode: default_mode(),
            interface_mode: default_interface_mode(),
        }
    }
}

impl TraclusArgs {
    pub fn print_small_summary(&self) -> String {
        format!(
            "TraclusArgs: max_dist={}, min_density={}, max_angle={}, segment_size={}\n",
            self.max_dist, self.min_density, self.max_angle, self.segment_size
        )
    }
}
