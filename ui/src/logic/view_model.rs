/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// view_model.rs — GUI-bound form state

use std::time::Instant;

use traclusdl_core::ui::args::TraclusArgs;
use traclusdl_core::utils::events::app_events::ComputationType;

// ─────────────────────────────────────────────
// ArgsBuffer
// ─────────────────────────────────────────────

pub struct ArgsBuffer {
    pub max_dist: String,
    pub min_density: String,
    pub max_angle: String,
    pub segment_size: String,

    pub input_name: String,
}

impl ArgsBuffer {
    pub fn from_args(args: &TraclusArgs) -> Self {
        Self {
            max_dist: args.max_dist.to_string(),
            min_density: args.min_density.to_string(),
            max_angle: args.max_angle.to_string(),
            segment_size: args.segment_size.to_string(),

            input_name: "".to_string(),
        }
    }
}

// ─────────────────────────────────────────────
// ViewModel
// ─────────────────────────────────────────────

pub struct ViewModel {
    pub args_selected: TraclusArgs,
    pub args_buffer: ArgsBuffer,

    // Input file info section
    pub input_name: String,
    pub num_dl: usize,
    pub percent_correlation: f64,
    pub args_when_loaded: TraclusArgs,

    // Computation info section
    pub num_computed: usize,
    pub total_to_compute: usize,
    pub start_time_computation: Instant,
    pub estimated_time_total: f64,
    pub computation_type: ComputationType,

    // Output section
    pub output: String,

    // Error section
    pub error_popup: Option<String>,
}

impl ViewModel {
    pub fn new(args: TraclusArgs) -> Self {
        let args_buffer: ArgsBuffer = ArgsBuffer::from_args(&args);
        let args_when_loaded: TraclusArgs = args.clone();
        Self {
            args_selected: args,
            args_buffer,
            args_when_loaded,

            input_name: String::new(),
            num_dl: 0,
            percent_correlation: 0.0,

            total_to_compute: 0,
            num_computed: 0,
            start_time_computation: Instant::now(),
            estimated_time_total: 0.0,
            computation_type: ComputationType::NotComputing,

            output: String::new(),
            error_popup: None,
        }
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new(TraclusArgs::default())
    }
}
