/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// output_writer.rs — writes corridor and segment list files

use crate::geometry::point::Point;
use crate::objects::cluster_member::ClusterMember;
use crate::objects::corridor::Corridor;
use crate::storage::clustered_trajectories::ClusteredTrajectories;
use crate::ui::args::TraclusArgs;
use crate::utils::events::app_events::{AppError, AppEvent};
use crate::utils::events::event_singleton::{emit, emit_error};
use std::path::Path;

use std::fs::File;
use std::io::{self, BufWriter, Write};

pub enum SegOutFormat {
    OldTraclus,
    NewTraclus,
}

// Writes corridorlist.txt beside input file
pub fn generate_corridor_file(
    args: &TraclusArgs,
    clust_storage: &ClusteredTrajectories,
) -> io::Result<()> {
    let output_filename: String = build_corridor_output_filename(args);

    let file: File = match File::create(&output_filename) {
        Ok(f) => f,
        Err(err) => {
            emit_error(AppError::IoError(format!(
                "Failed to create corridor output file: {}",
                err
            )));
            return Err(err);
        }
    };

    let mut writer: BufWriter<File> = BufWriter::new(file);

    if let Err(err) = write_corridor_header(&mut writer) {
        emit_error(AppError::IoError(format!(
            "Failed to write corridor header: {}",
            err
        )));
        return Err(err);
    }

    for corridor in &clust_storage.corridors {
        if let Err(err) = write_single_corridor(&mut writer, corridor) {
            emit_error(AppError::IoError(format!(
                "Failed to write corridor: {}",
                err
            )));
            return Err(err);
        }
    }

    if let Err(err) = writer.flush() {
        emit_error(AppError::IoError(format!(
            "Failed to flush corridor file: {}",
            err
        )));
        return Err(err);
    }

    emit(AppEvent::PrintInfo {
        messages: vec![format!("Corridor output written to: {}", output_filename)],
    });
    Ok(())
}

// Writes segmentlist.txt (old or new format)
pub fn generate_segment_file(
    args: &TraclusArgs,
    clust_storage: &ClusteredTrajectories,
    format: SegOutFormat,
) -> io::Result<()> {
    let output_filename: String = build_segment_output_filename(args, &format);

    let file: File = match File::create(&output_filename) {
        Ok(f) => f,
        Err(err) => {
            emit_error(AppError::IoError(format!(
                "Failed to create segment output file: {}",
                err
            )));
            return Err(err);
        }
    };

    let mut writer: BufWriter<File> = BufWriter::new(file);

    if let Err(err) = write_segment_header(&mut writer, &format) {
        emit_error(AppError::IoError(format!(
            "Failed to write segment header: {}",
            err
        )));
        return Err(err);
    }

    for (corridor_id, cluster_member) in clust_storage.get_all_cluster_members_iter() {
        let result: io::Result<()> = match format {
            SegOutFormat::OldTraclus => {
                write_single_segment_old(&mut writer, corridor_id, cluster_member)
            }
            SegOutFormat::NewTraclus => {
                write_single_segment_new(&mut writer, corridor_id, cluster_member)
            }
        };

        if let Err(err) = result {
            emit_error(AppError::IoError(format!(
                "Failed to write segment: {}",
                err
            )));
            return Err(err);
        }
    }

    if let Err(err) = writer.flush() {
        emit_error(AppError::IoError(format!(
            "Failed to flush segment file: {}",
            err
        )));
        return Err(err);
    }

    emit(AppEvent::PrintInfo {
        messages: vec![format!("Segment output written to: {}", output_filename)],
    });

    Ok(())
}

fn build_corridor_output_filename(args: &TraclusArgs) -> String {
    let input_path: &Path = Path::new(&args.file);
    let basename: &str = input_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("output");

    let parent_dir: &Path = input_path.parent().unwrap_or_else(|| Path::new("."));
    if args.output.is_empty() {
        format!(
            "{}/{}[{}-{}-{}-{}].corridors.txt",
            parent_dir.display(),
            basename,
            args.segment_size.round(),
            args.max_angle,
            args.max_dist.round(),
            args.min_density,
        )
    } else {
        format!("{}/{}.corridors.txt", parent_dir.display(), args.output)
    }
}

fn build_segment_output_filename(args: &TraclusArgs, format: &SegOutFormat) -> String {
    let input_path: &Path = Path::new(&args.file);
    let basename: &str = input_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("output");

    let parent_dir: &Path = input_path.parent().unwrap_or_else(|| Path::new("."));

    let suffix: &str = match format {
        SegOutFormat::OldTraclus => "segments_old",
        SegOutFormat::NewTraclus => "segments",
    };

    if args.output.is_empty() {
        format!(
            "{}/{}[{}-{}-{}-{}].{}.txt",
            parent_dir.display(),
            basename,
            args.segment_size.round(),
            args.max_angle,
            args.max_dist.round(),
            args.min_density,
            suffix
        )
    } else {
        format!("{}/{}.{}.txt", parent_dir.display(), args.output, suffix)
    }
}

// Format: {corridor_id}\t{trajectory_id}\t{segment_id}\t{weight}\t{angle}\t{xorigin}\t{yorigin}\t{xdestination}\t{ydestination}
fn write_single_segment_new(
    writer: &mut BufWriter<File>,
    corridor_id: i32,
    cluster_member: &ClusterMember,
) -> io::Result<()> {
    let end_point: Point = cluster_member.end_point();
    writeln!(
        writer,
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        corridor_id,
        cluster_member.traj_id,
        cluster_member.segment_id,
        cluster_member.weight,
        cluster_member.display_angle(),
        cluster_member.start.x,
        cluster_member.start.y,
        end_point.x,
        end_point.y
    )
}

// Format: {trajectory_id:segment_id}\t{weight}\t{angle}\t{corridor_id}\tLINESTRING({x1} {y1}, {x2} {y2})
fn write_single_segment_old(
    writer: &mut BufWriter<File>,
    corridor_id: i32,
    cluster_member: &ClusterMember,
) -> io::Result<()> {
    let end_point: Point = cluster_member.end_point();
    let start_str: String =
        cluster_member.start.x.to_string() + ":" + &cluster_member.start.y.to_string();
    let segment_id: String = cluster_member.traj_id.to_string() + ":" + &start_str;

    writeln!(
        writer,
        "{}\t{}\t{}\t{}\tLINESTRING({} {}, {} {})",
        segment_id,
        cluster_member.weight,
        cluster_member.display_angle(),
        corridor_id,
        cluster_member.start.x,
        cluster_member.start.y,
        end_point.x,
        end_point.y
    )
}

// Format: {id}\t{weight}\t{start.x}\t{start.y}\t{end.x}\t{end.y}
fn write_single_corridor(writer: &mut BufWriter<File>, corridor: &Corridor) -> io::Result<()> {
    writeln!(
        writer,
        "{}\t{}\t{}\t{}\t{}\t{}",
        corridor.id,
        corridor.weight,
        corridor.start.x,
        corridor.start.y,
        corridor.end.x,
        corridor.end.y
    )
}

// Writes the segment header based on the specified format.
// Old Traclus: id weight angle corridor_id coordinates
// New Traclus: corridor_id trajectory_id segment_id weight angle xorigin yorigin xdestination ydestination
fn write_segment_header(writer: &mut BufWriter<File>, format: &SegOutFormat) -> io::Result<()> {
    match format {
        SegOutFormat::OldTraclus => {
            writeln!(writer, "id\tweight\tangle\tcorridor_id\tcoordinates")
        }
        SegOutFormat::NewTraclus => {
            writeln!(
                writer,
                "corridor_id\ttrajectory_id\tsegment_id\tweight\tangle\txorigin\tyorigin\txdestination\tydestination"
            )
        }
    }
}

// Writes the corridor header: id weight xorigin yorigin xdestination ydestination
fn write_corridor_header(writer: &mut BufWriter<File>) -> io::Result<()> {
    writeln!(
        writer,
        "id\tweight\txorigin\tyorigin\txdestination\tydestination"
    )
}
