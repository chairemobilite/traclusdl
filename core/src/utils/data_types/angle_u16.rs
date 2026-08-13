/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// angle_u16.rs — fixed-point angle type (hundredths of a degree) for hot-loop comparisons

use std::{fmt, str::FromStr};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]

pub struct AngleU16(u16);

pub const SCALE: f64 = 100.0; // 2 decimal places of precision
pub const FULL_CIRCLE: u16 = 36_000; // 360.00° * SCALE, exclusive upper bound

// Strict range invariant: the inner u16 is ALWAYS in 0..=35999 (0.00°..=359.99°).
impl AngleU16 {
    // Normalizes any input degree value into 0..=35999 via integer rem_euclid,
    // so out-of-range input (negative, >360, NaN-adjacent) can never escape the invariant.
    pub fn from_degrees(angle_deg: f64) -> Self {
        debug_assert!(!angle_deg.is_nan(), "AngleU16::from_degrees received NaN");

        let scaled: i64 = (angle_deg * SCALE).round() as i64;
        let normalized: i64 = scaled.rem_euclid(FULL_CIRCLE as i64);
        Self(normalized as u16)
    }

    pub fn from_u16(angle_deg: u16) -> Self {
        let normalized: i64 = (angle_deg as i64).rem_euclid(FULL_CIRCLE as i64);
        Self(normalized as u16)
    }

    pub fn to_degrees(self) -> f64 {
        self.0 as f64 / SCALE
    }

    pub fn raw(self) -> u16 {
        self.0
    }

    // Smallest angular distance, accounting for wraparound
    // Result is always 0..=18000 (0.00°..=180.00°).
    pub fn min_diff(self, other: Self) -> u16 {
        let diff: i32 = (self.0 as i32 - other.0 as i32).abs();
        diff.min(FULL_CIRCLE as i32 - diff) as u16
    }

    // Smallest representable positive angle (1 raw unit = 1 / SCALE degrees).
    pub const MIN_POSITIVE: Self = Self(1);
}

impl fmt::Display for AngleU16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = format!("{:.2}", self.to_degrees());

        if s.contains('.') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }

        write!(f, "{s}")
    }
}

impl FromStr for AngleU16 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let deg: f64 = s.parse().map_err(|_| "must be a number".to_string())?;
        Ok(AngleU16::from_degrees(deg))
    }
}
