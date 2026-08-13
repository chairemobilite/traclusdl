/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// args_config.rs — single source of truth for argument defaults and constraints

use crate::utils::data_types::angle_u16::AngleU16;

pub struct ArgsConfig<T> {
    pub default: T,
    pub min: T,
    pub max: T,
    pub label: &'static str, // display name used in GUI headers and CLI help
}

// ─────────────────────────────────────────────
// AllArgsConfigs — the full set, returned as one struct
// ─────────────────────────────────────────────

pub struct AllArgsConfigs {
    pub max_dist: ArgsConfig<f64>,
    pub min_density: ArgsConfig<u32>,
    pub max_angle: ArgsConfig<f64>,
    pub segment_size: ArgsConfig<f64>,
    pub max_threads: ArgsConfig<u32>,
    pub num_fields_map: ArgsConfig<usize>,
}

// Returns shared parameter configs for CLI and GUI
pub fn get_param_configs() -> AllArgsConfigs {
    AllArgsConfigs {
        segment_size: ArgsConfig {
            default: 500.0,
            min: f64::MIN_POSITIVE, // > 0
            max: f64::MAX,
            label: "SEG SIZE",
        },
        max_angle: ArgsConfig {
            default: 5.0,
            min: AngleU16::MIN_POSITIVE.to_degrees(), // > 0.01
            max: 22.5,
            label: "MAX ANGLE",
        },
        max_dist: ArgsConfig {
            default: 250.0,
            min: 0.0,
            max: f64::MAX,
            label: "MAX DISTANCE",
        },
        min_density: ArgsConfig {
            default: 3,
            min: 1,
            max: u32::MAX,
            label: "MIN DENSITY",
        },
        max_threads: ArgsConfig {
            default: u32::MAX,
            min: 0,
            max: u32::MAX,
            label: "MAX THREADS",
        },
        num_fields_map: ArgsConfig {
            default: 5,
            min: 5,
            max: 6,
            label: "NUM FIELDS MAP",
        },
    }
}
