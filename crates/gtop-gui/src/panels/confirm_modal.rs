use egui::{Align2, Context, RichText};

use gtop_core::ipc::AppEvent;
use gtop_core::state::app_state::PendingAction;

use crate::theme::GuiTheme;

pub fn render(ctx: &Context, action: &PendingAction, theme: &GuiTheme, events: &mut Vec<AppEvent>) {
    let is_danger = action.kind.is_destructive();

    egui::Window::new("⚠ Confirm Action")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .min_width(320.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);

                let color = if is_danger { theme.danger } else { egui::Color32::YELLOW };

                ui.label(
                    RichText::new(format!("Action: {}", action.kind.label()))
                        .color(color)
                        .strong(),
                );

                ui.add_space(4.0);

                let target_str = if action.pids.len() == 1 {
                    format!("{} (PID {})", action.names[0], action.pids[0])
                } else {
                    format!("{} processes", action.pids.len())
                };

                ui.label(RichText::new(format!("Target: {}", target_str)).strong());

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    let confirm_btn = egui::Button::new(
                        RichText::new("  Confirm  ").color(egui::Color32::WHITE),
                    )
                    .fill(if is_danger { theme.danger } else { egui::Color32::DARK_GREEN });

                    if ui.add(confirm_btn).clicked() {
                        events.push(AppEvent::Confirm);
                    }

                    ui.add_space(12.0);

                    if ui.button("  Cancel  ").clicked() {
                        events.push(AppEvent::Cancel);
                    }
                });

                ui.add_space(8.0);
            });
        });
}
