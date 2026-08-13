/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// style.rs — shared GUI sizes and colors

use eframe::egui::Color32;

// ---------- Window ----------
pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 500.0;

// ---------- Spacing ----------
pub const SECTION_SPACING: f32 = 12.0;
pub const INNER_MARGIN: f32 = 10.0;
pub const WIDGET_SPACING: f32 = 6.0;

// ---------- Container ----------
pub const CONTAINER_WIDTH: f32 = 680.0;
pub const CONTAINER_ROUNDING: f32 = 4.0;

// ---------- Input fields ----------
pub const FILE_INPUT_WIDTH: f32 = 340.0;
pub const NUM_DL_WIDTH: f32 = 80.0;
pub const PERCENT_CORR_WIDTH: f32 = 80.0;
pub const PARAM_FIELD_WIDTH: f32 = 70.0;
pub const SPACE_BETWEEN_FIELD: f32 = 8.0;

// ---------- Buttons ----------
pub const BROWSE_BTN_WIDTH: f32 = 100.0;
pub const BROWSE_BTN_HEIGHT: f32 = 24.0;
pub const ACTION_BTN_WIDTH: f32 = 130.0;
pub const ACTION_BTN_HEIGHT: f32 = 36.0;

// ---------- Output area ----------
pub const OUTPUT_BOX_HEIGHT: f32 = 100.0;

// ---------- Colors (dark theme) ----------
pub const COLOR_BACKGROUND: Color32 = Color32::from_rgb(28, 28, 30);
pub const COLOR_SECTION_BG: Color32 = Color32::from_rgb(40, 40, 44);
pub const COLOR_BORDER: Color32 = Color32::from_rgb(65, 65, 72);
pub const COLOR_LABEL: Color32 = Color32::from_rgb(210, 210, 215);
pub const COLOR_TEXT: Color32 = Color32::from_rgb(230, 230, 235);
pub const COLOR_OUTPUT_BG: Color32 = Color32::from_rgb(22, 22, 24);
