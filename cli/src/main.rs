/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// main.rs — CLI entry point for launching the pipeline

use std::thread::JoinHandle;

use clap::Parser;
use traclusdl_core::traclusdl_pipeline::TraclusDLPipeline;
use traclusdl_core::ui::args::{InterfaceMode, TraclusArgs};
use traclusdl_core::utils::debug::logger::Logger;
use traclusdl_core::utils::debug::perf_timer::PerfTimer;
use traclusdl_core::utils::events::event_singleton;

fn main() {
    let traclus_args: TraclusArgs = TraclusArgs::parse();
    let main_traclusdl: TraclusDLPipeline = TraclusDLPipeline::default();

    TraclusDLPipeline::build_thread_pool(&traclus_args, false);

    // Start the logger thread or perf timer thread
    let handle: Option<JoinHandle<()>> = match traclus_args.interface_mode {
        InterfaceMode::Logger => Some(Logger::start()),
        InterfaceMode::PerfTimer => Some(PerfTimer::start()),
        _ => None,
    };

    main_traclusdl.run_full_traclus(traclus_args);

    // Correctly stop events and logger thread
    event_singleton::shutdown();
    if let Some(handle) = handle {
        handle.join().expect("Thread panicked");
    }
}
