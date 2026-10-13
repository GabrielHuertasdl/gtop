use std::time::Duration;

use eframe::egui::{self, Context};

use gtop_core::ipc::AppEvent;
use gtop_core::process::manager::ProcessManager;
use gtop_core::state::app_state::{ActionKind, AppState, PendingAction};
use gtop_core::state::{handle_event, SideEffect};

use crate::bridge::SharedState;
use crate::panels::{
    confirm_modal, detail_panel, performance_panel::PerfView, process_panel, performance_panel,
};
use crate::theme::GuiTheme;

#[derive(PartialEq, Clone, Copy)]
enum Tab { Processes, Performance }

pub struct GuiApp {
    shared:      SharedState,
    snapshot:    AppState,
    theme:       GuiTheme,
    active_tab:  Tab,
    show_detail: bool,
    perf_view:   PerfView,
}

impl GuiApp {
    pub fn new(cc: &eframe::CreationContext<'_>, shared: SharedState) -> Self {
        let theme = GuiTheme::default();
        theme.apply_visuals(&cc.egui_ctx);

        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(0.0, 0.0);
        cc.egui_ctx.set_style(style);

        let snapshot = shared
            .read()
            .map(|s| s.clone())
            .unwrap_or_else(|_| AppState::new(1));

        GuiApp {
            shared,
            snapshot,
            theme,
            active_tab: Tab::Processes,
            show_detail: false,
            perf_view: PerfView::default(),
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Pull state from background thread
        if let Ok(st) = self.shared.try_read() {
            self.snapshot = st.clone();
        }

        let mut events: Vec<AppEvent> = Vec::new();

        // ── Tab bar (top, WTM style) ──────────────────────────────────────────
        egui::TopBottomPanel::top("tab_bar")
            .min_height(36.0)
            .frame(egui::Frame::new().fill(self.theme.surface).inner_margin(egui::Margin::symmetric(0, 0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    tab_btn(ui, "Processes",   self.active_tab == Tab::Processes,   &self.theme, || self.active_tab = Tab::Processes);
                    tab_btn(ui, "Performance", self.active_tab == Tab::Performance, &self.theme, || self.active_tab = Tab::Performance);

                    // Right-aligned: detail toggle (only on Processes tab)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        if self.active_tab == Tab::Processes {
                            if ui.small_button(if self.show_detail { "◀ Detail" } else { "Detail ▶" }).clicked() {
                                self.show_detail = !self.show_detail;
                                events.push(AppEvent::ToggleDetail);
                            }
                        }
                    });
                });
            });

        // ── Detail sidebar (Processes tab only) ───────────────────────────────
        if self.active_tab == Tab::Processes && self.show_detail {
            if let Some(pid) = self.snapshot.focused_pid {
                let proc = self.snapshot.tree.by_pid.get(&pid).cloned();
                egui::SidePanel::right("detail_panel")
                    .default_width(260.0)
                    .width_range(200.0..=380.0)
                    .frame(egui::Frame::new()
                        .fill(self.theme.surface)
                        .inner_margin(egui::Margin::symmetric(12, 8)))
                    .show(ctx, |ui| {
                        detail_panel::render(ui, proc.as_ref(), &self.theme);
                    });
            }
        }

        // ── Central panel ─────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(self.theme.bg).inner_margin(egui::Margin::symmetric(0, 0)))
            .show(ctx, |ui| {
                match self.active_tab {
                    Tab::Processes => {
                        process_panel::render(ui, &self.snapshot, &self.theme, &mut events);
                    }
                    Tab::Performance => {
                        performance_panel::render(ui, &self.snapshot, &self.theme, &mut self.perf_view);
                    }
                }
            });

        // ── Confirmation modal ────────────────────────────────────────────────
        if let Some(ref action) = self.snapshot.pending_action.clone() {
            confirm_modal::render(ctx, action, &self.theme, &mut events);
        }

        // ── Apply events ──────────────────────────────────────────────────────
        if !events.is_empty() {
            if let Ok(mut st) = self.shared.write() {
                for event in events {
                    if let Some(effect) = handle_event(&mut st, event) {
                        match effect {
                            SideEffect::ExecuteAction(action) => {
                                execute_action(action);
                                st.selected_pids.clear();
                            }
                            SideEffect::Quit => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    }
                }
                self.snapshot = st.clone();
            }
        }

        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

// ── Tab button — active has bottom accent bar ─────────────────────────────────

fn tab_btn(ui: &mut Ui, label: &str, active: bool, theme: &GuiTheme, mut on_click: impl FnMut()) {
    let (resp, painter) = ui.allocate_painter(
        egui::vec2(120.0, 36.0),
        egui::Sense::click(),
    );

    let bg = if active { theme.bg } else { theme.surface };
    painter.rect_filled(resp.rect, egui::CornerRadius::ZERO, bg);

    if active {
        // Bottom blue bar
        let bar = egui::Rect::from_min_size(
            egui::pos2(resp.rect.min.x, resp.rect.max.y - 2.0),
            egui::vec2(resp.rect.width(), 2.0),
        );
        painter.rect_filled(bar, egui::CornerRadius::ZERO, theme.accent);
    }

    if resp.hovered() && !active {
        painter.rect_filled(resp.rect, egui::CornerRadius::ZERO,
            egui::Color32::from_white_alpha(8));
    }

    let text_color = if active { theme.text } else { theme.text_dim };
    painter.text(
        resp.rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        text_color,
    );

    if resp.clicked() { on_click(); }
}

use egui::Ui;

fn execute_action(action: PendingAction) {
    for (pid, name) in action.pids.iter().zip(action.names.iter()) {
        let pid = *pid;
        if !ProcessManager::verify_process(pid, name) {
            tracing::warn!("Process {} (PID {}) gone", name, pid);
            continue;
        }
        let result = match &action.kind {
            ActionKind::Kill      => ProcessManager::kill(pid),
            ActionKind::Terminate => ProcessManager::terminate(pid),
            ActionKind::Pause     => ProcessManager::pause(pid),
            ActionKind::Resume    => ProcessManager::resume(pid),
            ActionKind::Renice(v) => ProcessManager::renice(pid, *v),
            ActionKind::Signal(s) => ProcessManager::send_signal(pid, *s),
        };
        if let Err(e) = result {
            tracing::error!("Action on PID {} failed: {}", pid, e);
        }
    }
}
