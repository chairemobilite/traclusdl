/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// event_singleton.rs — shared event bus backed by global sender storage

use std::{
    sync::{
        Mutex, MutexGuard,
        mpsc::{self, Receiver, Sender},
    },
    time::Instant,
};

use super::app_events::{AppError, AppEvent};

static SUBSCRIBERS: Mutex<Vec<Sender<AppEvent>>> = Mutex::new(Vec::new());
static NUM_SUBSCRIBERS: Mutex<usize> = Mutex::new(0);

pub fn subscribe() -> Receiver<AppEvent> {
    let (tx, rx) = mpsc::channel();
    SUBSCRIBERS.lock().unwrap().push(tx);
    *NUM_SUBSCRIBERS.lock().unwrap() += 1;
    rx
}

pub fn shutdown() {
    let mut subscribers: MutexGuard<'_, Vec<Sender<AppEvent>>> = SUBSCRIBERS.lock().unwrap();
    subscribers.clear();

    *NUM_SUBSCRIBERS.lock().unwrap() = 0;
}

pub fn emit(event: AppEvent) {
    if *NUM_SUBSCRIBERS.lock().unwrap() == 0 {
        return;
    }

    let mut subscribers: MutexGuard<'_, Vec<Sender<AppEvent>>> = SUBSCRIBERS.lock().unwrap();
    // retain keeps only the senders whose send() succeeded
    subscribers.retain(|tx| tx.send(event.clone()).is_ok());
}

pub fn emit_error(error: AppError) {
    emit(AppEvent::Error(error));
}

#[allow(unused)]
pub fn emit_timed_perf(event_label: &str, is_start: bool, thread_index: Option<usize>) {
    emit(AppEvent::PerfTimer {
        event_label: event_label.to_string(),
        exact_instant: Instant::now(),
        is_start,
        thread_index,
    });
}
