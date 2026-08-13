/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// input_od_line.rs — parsed origin-destination line from input file

use super::point::Point;

#[derive(Debug)]
pub struct InputODLine {
    pub name: String,
    pub line_id: usize,
    pub weight: u32,
    pub start: Point,
    pub end: Point,
}
