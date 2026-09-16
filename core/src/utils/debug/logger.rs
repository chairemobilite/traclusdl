// logger.rs — Event subscriber that prints AppEvents to stdout, excluding perf timers

// The logger runs on its own dedicated std::thread
// CPU usage stays near zero — the thread is parked while waiting for events.

use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::utils::events::app_events::AppEvent;
use crate::utils::events::event_singleton::subscribe as singleton_subscribe;

pub struct Logger;

impl Logger {
    // Spawns the logger thread that prints application events
    pub fn start() -> JoinHandle<()> {
        let rx: Receiver<AppEvent> = singleton_subscribe();

        thread::Builder::new()
            .name("traclus-logger".to_string())
            .spawn(move || Self::run(rx))
            .expect("failed to spawn logger thread")
    }

    fn run(rx: Receiver<AppEvent>) {
        let start_time: Instant = Instant::now();

        // recv() parks the thread with zero CPU usage until an event is received
        while let Ok(event) = rx.recv() {
            match event {
                AppEvent::LoadComplete {
                    desire_line_count: traj_count,
                    correlation,
                } => {
                    println!(
                        "[LOG] LOAD COMPLETED at {:?} — {} trajectories loaded, correlation: {:.2}%.",
                        start_time.elapsed(),
                        traj_count,
                        correlation * 100.0
                    );
                }

                AppEvent::ComputationStart {
                    computation_type,
                    max_progress,
                } => {
                    println!(
                        "[LOG] COMPUTATION STARTED at {:?} — {:?} with {} total steps.",
                        start_time.elapsed(),
                        computation_type,
                        max_progress,
                    );
                }

                AppEvent::ComputationProgress {
                    computation_type,
                    increment_progress,
                } => {
                    println!(
                        "[LOG] {:?} progress: +{} steps at {:?}.",
                        computation_type,
                        increment_progress,
                        start_time.elapsed()
                    );
                }

                AppEvent::ComputationComplete { computation_type } => {
                    println!(
                        "[LOG] {:?} COMPLETED at {:?}.",
                        computation_type,
                        start_time.elapsed()
                    );
                }

                AppEvent::PrintInfo { messages } => {
                    for message in messages {
                        println!("[LOG] {}", message);
                    }
                }

                // PerfTimer events are handled exclusively by PerfTimer — ignore here
                AppEvent::PerfTimer { .. } => {}

                AppEvent::Error(msg) => {
                    eprintln!("[LOG][ERROR] {}", msg);
                }
            }
        }
    }
}
