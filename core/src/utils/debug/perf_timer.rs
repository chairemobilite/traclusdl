// perf_timer.rs — Event subscriber that collects PerfTimer events and prints a summary on exit

// PerfTimer runs on its own dedicated std::thread.
// CPU usage stays near zero — the thread is parked while waiting for events.
// All output is deferred: nothing is printed until the event channel closes.

// How to use it inside the code:
//  utils::events::event_singleton::emit_timed_perf("name_of_the_task", true, Option<thread_index>);
//  THE SECTION OF CODE TO TIME
//  utils::events::event_singleton::emit_timed_perf("name_of_the_task", false, Option<thread_index>);

// If the section of code to time is nested inside another timed section,
// the parent section will be printed first, followed by its children, and so on recursively.

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::utils::events::app_events::AppEvent;
use crate::utils::events::event_singleton::subscribe as singleton_subscribe;

struct PerfRecord {
    display_label: String,
    elapsed_ms: f64,
    instances: usize,
    children: Vec<String>,
}

impl PerfRecord {
    fn new(display_label: String) -> Self {
        Self {
            display_label,
            elapsed_ms: 0.0,
            instances: 0,
            children: Vec::new(),
        }
    }
}

pub struct PerfTimer {
    active_timers: Vec<(String, Instant)>,
    root_elements: Vec<String>,
    all_elements: HashMap<String, PerfRecord>,
}

impl PerfTimer {
    // Spawns the perf timer thread that aggregates timing events
    pub fn start() -> JoinHandle<()> {
        let rx: Receiver<AppEvent> = singleton_subscribe();

        thread::Builder::new()
            .name("traclus-perf-timer".to_string())
            .spawn(move || Self::run(rx))
            .expect("failed to spawn perf timer thread")
    }

    fn run(rx: Receiver<AppEvent>) {
        let mut perf_timer: PerfTimer = PerfTimer {
            active_timers: Vec::new(),
            root_elements: Vec::new(),
            all_elements: HashMap::new(),
        };

        // recv() parks the thread with zero CPU usage until an event is received
        while let Ok(event) = rx.recv() {
            if let AppEvent::PerfTimer {
                event_label,
                exact_instant,
                is_start,
                thread_index,
            } = event
            {
                let mut full_label: String = event_label.clone();

                if let Some(tid) = thread_index {
                    full_label = format!("{}(tid:{})", event_label, tid);
                }

                if is_start {
                    perf_timer.handle_timer_start(event_label, full_label, exact_instant);
                } else {
                    perf_timer.handle_timer_end(event_label, full_label, exact_instant);
                }
            }
        }

        perf_timer.print_summary();
    }

    fn handle_timer_start(
        &mut self,
        event_label: String,
        full_label: String,
        exact_instant: Instant,
    ) {
        // Find the current parent timer from the active stack
        // If none exists, this timer is a root-level task
        let parent_name: String = self
            .active_timers
            .iter()
            .rev()
            .find(|(label, _)| label != &event_label)
            .map(|(label, _)| label.clone())
            .unwrap_or_else(|| "".to_string());

        let parent: Option<&mut PerfRecord> = self.all_elements.get_mut(&parent_name);

        if let Some(parent) = parent {
            if !parent.children.contains(&full_label) {
                parent.children.push(full_label.clone());
            }
        } else {
            self.root_elements.push(full_label.clone());
        }

        self.active_timers
            .push((event_label.clone(), exact_instant));

        // Create the record immediately so children can safely reference it
        // Elapsed time and instance count are updated when the timer ends
        let label: String = full_label.clone();
        self.all_elements
            .entry(full_label)
            .or_insert_with(|| PerfRecord::new(label));
    }

    fn handle_timer_end(
        &mut self,
        event_label: String,
        full_label: String,
        exact_instant: Instant,
    ) {
        // Remove timer from the active stack, compute elapsed time,
        // then update the associated performance record
        let position: usize = self
            .active_timers
            .iter()
            .position(|(label, _)| *label == event_label)
            .expect("Timer end received for event with no matching start");

        let (_, timer): (String, Instant) = self.active_timers.remove(position);
        let record_element: &mut PerfRecord = self.all_elements.get_mut(&full_label).unwrap();
        let elapsed_ms: f64 = exact_instant.duration_since(timer).as_secs_f64() * 1000.0;

        record_element.elapsed_ms += elapsed_ms;
        record_element.instances += 1;
    }

    fn print_summary(&self) {
        println!("\n ──────────────────── SUMMARY ────────────────────");

        let total_ms: f64 = self
            .root_elements
            .iter()
            .filter_map(|label| self.all_elements.get(label))
            .map(|record| record.elapsed_ms)
            .sum();

        for (i, root) in self.root_elements.iter().enumerate() {
            self.print_parent(root, total_ms, 0, format!("{}", i + 1));
        }

        println!(" ─────────────────────────────────────────────────");
        println!("[PERF] {:<50}: {:>10.3} ms", "TOTAL", total_ms);
    }

    fn print_parent(&self, parent_label: &String, parent_ms: f64, depth: usize, index: String) {
        let parent_record: &PerfRecord = self.all_elements.get(parent_label).unwrap();

        Self::print_record(parent_record, parent_ms, depth, &index);

        for (i, child) in parent_record.children.iter().enumerate() {
            self.print_parent(
                child,
                parent_record.elapsed_ms,
                depth + 1,
                format!("{}.{}", index, i + 1),
            );
        }
    }

    fn print_record(record: &PerfRecord, parent_ms: f64, depth: usize, index: &str) {
        let indent = "   ".repeat(depth);

        let label = format!("{}{} {}", indent, index, record.display_label);

        let instance_tag = if record.instances > 1 {
            format!(" ×{}", record.instances)
        } else {
            String::new()
        };

        println!(
            "[PERF] {:<50}: {:>10.3} ms ({:>6.2}%){}",
            label,
            record.elapsed_ms,
            (record.elapsed_ms / parent_ms) * 100.0,
            instance_tag
        );
    }
}
