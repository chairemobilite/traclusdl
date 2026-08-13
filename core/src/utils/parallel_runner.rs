/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// core_parallel_runner.rs — enforces one-task-at-a-time execution for triggered work

// The caller keeps one runner instance so only one computation can run at a time.
// Buttons call try_run(...); if work is already running, the call returns false immediately.
// The worker thread may still use the internal Rayon pool owned by TraclusDLCore.

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::traclusdl_pipeline::TraclusDLPipeline;

// Shared bool: true while any task is executing.
// Shared ownership between GUI thread and worker thread.
type RunningFlag = Arc<Mutex<bool>>;
pub type StopFlag = Arc<AtomicBool>;

// RAII guard: sets the flag back to false when it goes out of scope.
struct ReleaseOnDrop(RunningFlag, StopFlag);

impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        *self.0.lock().unwrap() = false;
        self.1.store(false, Ordering::Relaxed);
    }
}

// ─────────────────────────────────────────────
// CoreParallelRunner
// ─────────────────────────────────────────────

pub struct ParallelRunner {
    is_running: RunningFlag,
    stop_flag: StopFlag,
}

impl Default for ParallelRunner {
    fn default() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl ParallelRunner {
    // Launches one background task if the runner is idle
    pub fn try_run<F>(&self, main: Arc<Mutex<TraclusDLPipeline>>, task: F) -> bool
    where
        F: FnOnce(&mut TraclusDLPipeline, StopFlag) + Send + 'static,
    {
        if !self.try_acquire() {
            return false;
        }
        self.stop_flag.store(false, Ordering::Relaxed);

        let flag: Arc<Mutex<bool>> = Arc::clone(&self.is_running);
        let stop: StopFlag = Arc::clone(&self.stop_flag);

        thread::spawn(move || {
            // Guard releases the flag when this thread scope exits, even on panic
            let _guard: ReleaseOnDrop = ReleaseOnDrop(flag, Arc::clone(&stop));
            let mut core = match main.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            task(&mut core, stop);
        });

        true
    }

    // Atomically checks and sets the flag.
    // Returns false if already running, true if successfully acquired.
    fn try_acquire(&self) -> bool {
        let mut running = self.is_running.lock().unwrap();
        if *running {
            return false;
        }
        *running = true;
        true
    }

    // Signals the running task to stop when one is active
    pub fn stop(&self) {
        if self.is_running() {
            self.stop_flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }
}
