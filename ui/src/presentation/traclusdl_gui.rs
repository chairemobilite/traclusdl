/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// traclusdl_gui.rs — GUI rendering for sections and widgets
// This file is mainly AI generated (Claude.ai)
// It's made to provide a minimal working GUI for users

use std::fmt::Display;
use std::str::FromStr;

use eframe::egui;
use eframe::egui::{RichText, ScrollArea, TextEdit, Vec2};
use rfd::FileDialog;

use super::style::*;
use crate::logic::traclusdl_app::TraclusDLApp;
use traclusdl_core::ui::args::ExecutionMode;
use traclusdl_core::ui::args_config::get_param_configs;
use traclusdl_core::utils::data_types::angle_u16::AngleU16;

// ─────────────────────────────────────────────
// App Update (main render loop)
// ─────────────────────────────────────────────

impl eframe::App for TraclusDLApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events();
        render_error_popup(ctx, self);

        // Request a repaint every frame while a task is running so the
        // progress display stays live without user interaction
        if self.runner.is_running() {
            ctx.request_repaint();
        }

        // Apply dark theme overrides
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = COLOR_BACKGROUND;
        visuals.window_fill = COLOR_SECTION_BG;
        visuals.extreme_bg_color = COLOR_OUTPUT_BG;
        visuals.faint_bg_color = COLOR_SECTION_BG;
        visuals.widgets.inactive.fg_stroke.color = COLOR_TEXT;
        visuals.widgets.hovered.fg_stroke.color = COLOR_TEXT;
        visuals.widgets.active.fg_stroke.color = COLOR_TEXT;
        visuals.widgets.noninteractive.fg_stroke.color = COLOR_TEXT;
        visuals.widgets.open.fg_stroke.color = COLOR_TEXT;
        ctx.set_visuals(visuals);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(SECTION_SPACING);

            // All sections are placed inside a centered column of exactly CONTAINER_WIDTH.
            // We derive the left offset so every section starts at the same x coordinate.
            let panel_width = ui.available_width();
            let left_offset = ((panel_width - CONTAINER_WIDTH) / 2.0).max(0.0);

            let mut render = |ui: &mut egui::Ui| {
                render_file_section(ui, self);
                ui.add_space(SECTION_SPACING);

                render_parameters_section(ui, self);
                ui.add_space(SECTION_SPACING);

                render_computing_mode_section(ui, self);
                ui.add_space(SECTION_SPACING);

                ui.add(egui::Separator::default().horizontal().spacing(6.0));
                ui.add_space(4.0);

                render_output_section(ui, self);
                ui.add_space(SECTION_SPACING);

                render_action_bar(ui, self);
            };

            // Place a child UI of exactly CONTAINER_WIDTH at the computed left offset.
            // This guarantees that every section rendered inside shares the same x bounds.
            ui.horizontal(|ui| {
                ui.add_space(left_offset);
                ui.vertical(|ui| {
                    ui.set_width(CONTAINER_WIDTH);
                    render(ui);
                });
            });
        });
    }
}

// ─────────────────────────────────────────────
// Shared: container frame used by every section
// ─────────────────────────────────────────────

// Frame overhead = border (1 px each side) + inner margin (each side).
const FRAME_OVERHEAD: f32 = (INNER_MARGIN + 1.0) * 2.0;
const INNER_WIDTH: f32 = CONTAINER_WIDTH - FRAME_OVERHEAD;

fn container_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(COLOR_SECTION_BG)
        .stroke(egui::Stroke::new(1.0_f32, COLOR_BORDER))
        .rounding(CONTAINER_ROUNDING)
        .inner_margin(egui::Margin::same(INNER_MARGIN))
}

// ─────────────────────────────────────────────
// Section: File Input
// ─────────────────────────────────────────────

fn render_file_section(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    container_frame().show(ui, |ui| {
        ui.set_min_width(INNER_WIDTH);

        // Labels row
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("Input file name").color(COLOR_LABEL));
            ui.add_space(FILE_INPUT_WIDTH - 90.0 + SPACE_BETWEEN_FIELD * 2.0);
            ui.label(RichText::new("Number DL").color(COLOR_LABEL));
            ui.add_space(NUM_DL_WIDTH - 62.0 + SPACE_BETWEEN_FIELD * 2.0);
            ui.label(RichText::new("% correlation").color(COLOR_LABEL));
        });

        ui.add_space(WIDGET_SPACING);

        // Fields row — all read-only; values come from current_vm()
        ui.horizontal(|ui| {
            ui.add(
                TextEdit::singleline(&mut app.current_vm().input_name)
                    .desired_width(FILE_INPUT_WIDTH)
                    .clip_text(true)
                    .interactive(false)
                    .text_color(COLOR_TEXT),
            );
            ui.add_space(SPACE_BETWEEN_FIELD);

            let mut num_str = app.current_vm().num_dl.to_string();
            ui.add(
                TextEdit::singleline(&mut num_str)
                    .desired_width(NUM_DL_WIDTH)
                    .interactive(false)
                    .text_color(COLOR_TEXT),
            );
            ui.add_space(SPACE_BETWEEN_FIELD);

            let mut pct_str = format!("{:.1}%", app.current_vm().percent_correlation * 100.0);
            ui.add(
                TextEdit::singleline(&mut pct_str)
                    .desired_width(PERCENT_CORR_WIDTH)
                    .interactive(false)
                    .text_color(COLOR_TEXT),
            );
            ui.add_space(SPACE_BETWEEN_FIELD);

            let browse_response = ui.add_enabled(
                !app.runner.is_running(),
                egui::Button::new("Browse File")
                    .min_size([BROWSE_BTN_WIDTH, BROWSE_BTN_HEIGHT].into()),
            );
            if browse_response.clicked()
                && let Some(path) = FileDialog::new()
                    .add_filter("Text file", &["txt", "csv", "tsv"])
                    .pick_file()
            {
                app.on_browse_done(path);
            }
        });
    });
}

// ─────────────────────────────────────────────
// Section: Parameters
// ─────────────────────────────────────────────

fn render_parameters_section(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    // Fetch config once — labels and ranges come from param_config
    let cfg = get_param_configs();

    container_frame().show(ui, |ui| {
        ui.set_min_width(INNER_WIDTH);
        ui.set_max_width(INNER_WIDTH);

        let add_col_width: f32 = 44.0;
        let sep_width: f32 = 6.0;
        let param_col_width = INNER_WIDTH - add_col_width - sep_width;

        ui.horizontal(|ui| {
            // ── Left: headers + parameter rows ───────────────────────────
            ui.vertical(|ui| {
                ui.set_width(param_col_width);

                // Header row — labels come from param_config
                ui.horizontal(|ui| {
                    ui.add_space(26.0); // aligns with "#N " prefix below
                    for label in &[
                        cfg.segment_size.label,
                        cfg.max_angle.label,
                        cfg.max_dist.label,
                        cfg.min_density.label,
                    ] {
                        ui.add_sized(
                            [PARAM_FIELD_WIDTH, 16.0],
                            egui::Label::new(
                                RichText::new(*label).small().color(COLOR_LABEL).strong(),
                            ),
                        );
                        ui.add_space(WIDGET_SPACING);
                    }
                });

                ui.separator();

                // One row per ViewModel in the list
                let mut to_remove: Option<usize> = None;
                let vm_count = app.vm.len();

                for (idx, vm) in app.vm.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("#{}", idx + 1)).color(COLOR_TEXT));
                        ui.add_space(4.0);

                        // segment_size
                        commit_on_focus_loss(
                            ui,
                            &mut vm.args_buffer.segment_size,
                            &mut vm.args_selected.segment_size,
                            PARAM_FIELD_WIDTH,
                            cfg.segment_size.min,
                            cfg.segment_size.max,
                        );
                        ui.add_space(WIDGET_SPACING);

                        // max_angle
                        commit_on_focus_loss(
                            ui,
                            &mut vm.args_buffer.max_angle,
                            &mut vm.args_selected.max_angle,
                            PARAM_FIELD_WIDTH,
                            AngleU16::from_degrees(cfg.max_angle.min),
                            AngleU16::from_degrees(cfg.max_angle.max),
                        );
                        ui.add_space(WIDGET_SPACING);

                        // max_dist
                        commit_on_focus_loss(
                            ui,
                            &mut vm.args_buffer.max_dist,
                            &mut vm.args_selected.max_dist,
                            PARAM_FIELD_WIDTH,
                            cfg.max_dist.min,
                            cfg.max_dist.max,
                        );
                        ui.add_space(WIDGET_SPACING);

                        // min_density
                        commit_on_focus_loss(
                            ui,
                            &mut vm.args_buffer.min_density,
                            &mut vm.args_selected.min_density,
                            PARAM_FIELD_WIDTH,
                            cfg.min_density.min,
                            cfg.min_density.max,
                        );
                        ui.add_space(WIDGET_SPACING);

                        if vm_count > 1 && ui.small_button(" - ").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                }

                if let Some(idx) = to_remove {
                    app.vm.remove(idx);
                    // Keep current_selected_vm in bounds after removal
                    if app.current_selected_vm >= app.vm.len() {
                        app.current_selected_vm = app.vm.len().saturating_sub(1);
                    }
                }
            });

            ui.separator();

            // ── Right: + button ───────────────────────────────────────────
            ui.allocate_ui(Vec2::new(add_col_width, ui.available_height()), |ui| {
                ui.centered_and_justified(|ui| {
                    if ui.add_sized([36.0, 36.0], egui::Button::new("+")).clicked() {
                        app.on_plus_vm();
                    }
                });
            });
        });
    });
}

// ─────────────────────────────────────────────
// Commit helpers
//
// `buf` is a &mut String bound directly to the TextEdit.
// The user can type anything — empty string, partial number, minus sign — freely.
// Only on focus loss or Enter is the text parsed, clamped, and written to `value`.
// If parsing fails the buffer is reset to the last valid committed value.
// ─────────────────────────────────────────────

fn commit_on_focus_loss<T>(
    ui: &mut egui::Ui,
    buf: &mut String, // raw text bound to TextEdit — may be empty or partial
    value: &mut T,    // committed value — only updated on focus loss / Enter
    width: f32,
    min: T,
    max: T,
) where
    T: FromStr + Display + PartialOrd + Copy,
{
    let response = ui.add(
        TextEdit::singleline(buf)
            .desired_width(width)
            .clip_text(true)
            .text_color(COLOR_TEXT),
    );
    // buf is updated in place by TextEdit — no need to sync manually

    let commit = response.lost_focus()
        || (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));

    if commit {
        match buf.trim().parse::<T>() {
            Ok(v) => {
                let mut v = v;
                if v < min {
                    v = min;
                }
                if v > max {
                    v = max;
                }
                *value = v;
                *buf = value.to_string(); // normalise buffer to clamped value
            }
            Err(_) => {
                *buf = value.to_string(); // restore buffer to last valid committed value
            }
        }
    }
}

// ─────────────────────────────────────────────
// Section: Computing Mode
// ─────────────────────────────────────────────

fn render_computing_mode_section(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    container_frame().show(ui, |ui| {
        ui.set_min_width(CONTAINER_WIDTH - 5.0); // compensate for frame horizontal padding

        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.radio_value(
                &mut app.current_vm().args_selected.mode,
                ExecutionMode::Serial,
                RichText::new("Serial computing").color(COLOR_TEXT),
            );
            ui.add_space(40.0);
            let parallel_label = format!("Parallel computing ({} CPU detected)", app.detected_cpus);
            ui.radio_value(
                &mut app.current_vm().args_selected.mode,
                ExecutionMode::ParallelRayon,
                RichText::new(parallel_label).color(COLOR_TEXT),
            );
        });
    });
}

// ─────────────────────────────────────────────
// Section: Outputs
// ─────────────────────────────────────────────

fn render_output_section(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    ui.label(RichText::new("OUTPUTS").color(COLOR_LABEL).strong());
    ui.add_space(4.0);

    egui::Frame::none()
        .fill(COLOR_OUTPUT_BG)
        .stroke(egui::Stroke::new(1.0_f32, COLOR_BORDER))
        .rounding(CONTAINER_ROUNDING)
        .inner_margin(egui::Margin::same(INNER_MARGIN))
        .show(ui, |ui| {
            ui.set_min_width(INNER_WIDTH);

            ScrollArea::vertical()
                .max_height(OUTPUT_BOX_HEIGHT)
                .min_scrolled_height(OUTPUT_BOX_HEIGHT)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Subtract scrollbar width (~12 px) so text doesn't clip under it
                    ui.set_min_width(INNER_WIDTH - 12.0);
                    let output = &app.current_vm().output;
                    if output.is_empty() {
                        ui.label(
                            RichText::new("No output yet.")
                                .color(egui::Color32::GRAY)
                                .italics(),
                        );
                    } else {
                        ui.label(RichText::new(output.as_str()).color(COLOR_TEXT));
                    }
                });
        });
}

// ─────────────────────────────────────────────
// Section: Action Bar
// ─────────────────────────────────────────────

fn render_action_bar(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    ui.set_min_width(CONTAINER_WIDTH);

    if app.runner.is_running() {
        render_action_bar_running(ui, app);
    } else {
        render_action_bar_idle(ui, app);
    }
}

// ── Idle state: Start Computation + Create Output ────────────────────────────

fn render_action_bar_idle(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    ui.horizontal(|ui| {
        let start_response = ui.add_sized(
            [ACTION_BTN_WIDTH, ACTION_BTN_HEIGHT],
            egui::Button::new("Start\nComputation"),
        );
        if start_response.clicked() {
            app.on_start_computation();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let create_response = ui.add_sized(
                [ACTION_BTN_WIDTH, ACTION_BTN_HEIGHT],
                egui::Button::new("Create output"),
            );
            if create_response.clicked() {
                app.on_generate_outputs();
            }
        });
    });
}

// ── Running state: progress bar + time info + Stop button ────────────────────

fn render_action_bar_running(ui: &mut egui::Ui, app: &mut TraclusDLApp) {
    let vm = app.current_vm();
    let (r, g, b) = vm.computation_type.color();
    let color_progress = egui::Color32::from_rgb(r, g, b);

    let progress: f32 = if vm.total_to_compute > 0 {
        vm.num_computed as f32 / vm.total_to_compute as f32
    } else {
        0.0
    };

    let elapsed_secs = vm.start_time_computation.elapsed().as_secs_f64();
    let elapsed_str = format_duration(elapsed_secs);

    let eta_str = if vm.estimated_time_total > 0.0 {
        format_duration(vm.estimated_time_total)
    } else {
        "Estimating...".to_string()
    };

    // Row 1: progress bar spanning full width minus Stop button
    ui.horizontal(|ui| {
        // Stop button flush right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_sized([80.0, ACTION_BTN_HEIGHT], egui::Button::new("Stop"))
                .clicked()
            {
                app.on_stop_computation();
            }

            // Progress bar fills remaining space to the left of Stop
            let bar_width = ui.available_width() - 8.0;
            ui.add(
                egui::ProgressBar::new(progress)
                    .desired_width(bar_width)
                    .show_percentage()
                    .fill(color_progress),
            );
        });
    });

    ui.add_space(4.0);

    // Row 2: time labels
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.label(RichText::new("Elapsed: ").color(COLOR_LABEL));
        ui.label(RichText::new(&elapsed_str).color(COLOR_TEXT));
        ui.add_space(24.0);
        ui.label(RichText::new("Total Estimate: ").color(COLOR_LABEL));
        ui.label(RichText::new(&eta_str).color(COLOR_TEXT));
    });
}

// ─────────────────────────────────────────────
// Duration formatting
//
// < 60 s  →  "42s"
// < 1 h   →  "3m 07s"
// >= 1 h  →  "1h 03m"
// ─────────────────────────────────────────────

fn format_duration(secs: f64) -> String {
    let total = secs as u64;
    if total < 60 {
        format!("{}s", total)
    } else if total < 3600 {
        format!("{}m {:02}s", total / 60, total % 60)
    } else {
        format!("{}h {:02}m", total / 3600, (total % 3600) / 60)
    }
}

// ─────────────────────────────────────────────
// Error Popup
// ─────────────────────────────────────────────

fn render_error_popup(ctx: &egui::Context, app: &mut TraclusDLApp) {
    let error_msg = match &app.current_vm().error_popup {
        Some(msg) => msg.clone(),
        None => return,
    };

    // Dim and block the entire UI behind the popup
    egui::Area::new(egui::Id::new("error_modal_backdrop"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::PanelResizeLine)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(120));
            // Consume all pointer input so nothing behind is clickable
            ui.allocate_rect(screen, egui::Sense::click_and_drag());
        });

    egui::Window::new("Error")
        .collapsible(false)
        .resizable(false)
        .fixed_size([500.0, 300.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(&error_msg).color(COLOR_LABEL));
            ui.add_space(12.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.button("  OK  ").clicked() {
                    app.current_vm().error_popup = None;
                }
            });
        });
}
