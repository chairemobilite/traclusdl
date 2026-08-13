/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// traclusdl_pipeline.rs — orchestrates load, cluster, and output for GUI and CLI

use std::thread::available_parallelism;

use super::storage::clustered_trajectories::ClusteredTrajectories;
use super::storage::raw_trajectories::RawTrajectories;
use crate::utils::events::app_events::{AppError, AppEvent, ComputationType};

use crate::io::input_loader::parse_input_data;
use crate::io::output_writer::{SegOutFormat, generate_corridor_file, generate_segment_file};
use crate::ui::args::{ExecutionMode, InterfaceMode, TraclusArgs};
use crate::utils::events::event_singleton::{emit, emit_error};
use crate::utils::parallel_runner::StopFlag;
use crate::utils::statistic::{clustering_histogram, directional_correlation};

use super::algorithms::base_traclusdl::TraclusAlgorithm;
use super::algorithms::parallel_rayon_traclusdl::ParallelRayonTraclusDL;
use super::algorithms::serial_traclusdl::SerialTraclusDL;

#[derive(Default)]
pub struct TraclusDLPipeline {
    raw_storage: Option<RawTrajectories>,
    clust_storage: Option<ClusteredTrajectories>,
}

impl TraclusDLPipeline {
    // Parses input file and resets clustered storage
    pub fn load_raw_storage(&mut self, args: &TraclusArgs, _: StopFlag) {
        self.raw_storage = parse_input_data(args);
        self.clust_storage = None;

        if self.raw_storage.is_none() {
            return;
        }

        // Notify GUI of trajectory count and directional correlation
        emit(AppEvent::LoadComplete {
            desire_line_count: self.raw_storage.as_ref().unwrap().get_num_trajectories(),
            correlation: directional_correlation(self.raw_storage.as_ref().unwrap()),
        });
    }

    // Runs DBSCAN clustering and emits summary stats on success
    pub fn run_clustering(&mut self, args: &TraclusArgs, stop: StopFlag) {
        if self.raw_storage.is_none() {
            emit_error(AppError::NoRawStorage);
            return;
        }
        self.clust_storage = None;
        let raw_storage: &RawTrajectories = self.raw_storage.as_ref().unwrap();
        let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new(args);

        let mut clustering_algorithm: Box<dyn TraclusAlgorithm> = Self::get_proper_algorithm(args);
        clustering_algorithm.set_stop_flag(stop);
        let result: bool = clustering_algorithm.db_scan_clustering(raw_storage, &mut clust_storage);
        if result {
            emit(AppEvent::PrintInfo {
                messages: clust_storage.get_summary(),
            });
            emit(AppEvent::PrintInfo {
                messages: clustering_histogram(&clust_storage).get_summary(),
            });
            self.clust_storage = Some(clust_storage);
        }
    }

    // Writes corridor and segment output files from clustered storage
    pub fn generate_outputs(&mut self, _: &TraclusArgs, _: StopFlag) {
        if self.clust_storage.is_none() {
            emit_error(AppError::NoClustStorage);
            return;
        }

        let clust_storage: &ClusteredTrajectories = self.clust_storage.as_ref().unwrap();
        let args: &TraclusArgs = &clust_storage.args_snapshot;

        // Notify GUI of output generation start, but no progress tracking
        emit(AppEvent::ComputationStart {
            computation_type: ComputationType::CreateOutputs,
            max_progress: 0,
        });

        generate_corridor_file(args, clust_storage).unwrap();
        generate_segment_file(args, clust_storage, SegOutFormat::NewTraclus).unwrap();
    }

    // CLI entry point: full pipeline without GUI progress overhead
    pub fn run_full_traclus(&self, args: TraclusArgs) {
        let raw_storage: RawTrajectories =
            parse_input_data(&args).expect("Failed to parse input data");

        let mut clust_storage: ClusteredTrajectories = ClusteredTrajectories::new(&args);
        let clustering_algorithm: Box<dyn TraclusAlgorithm> = Self::get_proper_algorithm(&args);
        clustering_algorithm.db_scan_clustering(&raw_storage, &mut clust_storage);

        generate_corridor_file(&args, &clust_storage).unwrap();
        generate_segment_file(&args, &clust_storage, SegOutFormat::NewTraclus).unwrap();
        // If you want to generate the old Traclus segment output format, uncomment the following line
        //generate_segment_file(&args, &clust_storage, SegOutFormat::OldTraclus);
    }

    // Configures Rayon thread pool, reserving CPUs for logger/GUI when active
    pub fn build_thread_pool(args: &TraclusArgs, gui_active: bool) {
        let available: usize = available_parallelism().map(|n| n.get()).unwrap_or(2).max(1);

        let mut reserved: usize = match args.interface_mode {
            InterfaceMode::Logger => 1,      // 1 CPU for the logger thread
            InterfaceMode::PerfTimer => 0, // no reservation — perf timer events are very lightweight and at the end
            InterfaceMode::Performance => 0, // no reservation — all CPUs to computation
        };

        if gui_active {
            reserved += 1; // 1 CPU for the GUI thread
        }

        let computation: usize = available.saturating_sub(reserved).max(1);
        let mut threads_to_use: usize = args.max_threads.min(computation as u32) as usize;

        if args.mode == ExecutionMode::Serial {
            threads_to_use = 1; // Force single-threaded execution for serial mode
        }

        rayon::ThreadPoolBuilder::new()
            .num_threads(threads_to_use)
            .build_global()
            .expect("Failed to build Rayon thread pool");

        println!(
            "Available CPUs: {}, reserved for UI/Logger: {}, used for computation: {}",
            available, reserved, threads_to_use
        );
    }

    fn get_proper_algorithm(args: &TraclusArgs) -> Box<dyn TraclusAlgorithm> {
        match args.mode {
            ExecutionMode::Serial => Box::new(SerialTraclusDL::new(args.clone())),
            ExecutionMode::ParallelRayon => Box::new(ParallelRayonTraclusDL::new(args.clone())),
        }
    }
}
