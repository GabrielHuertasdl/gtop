use egui::{Color32, Grid, RichText, ScrollArea, Ui};

use gtop_core::ipc::AppEvent;
use gtop_core::process::filter::SortColumn;
use gtop_core::process::{ProcessInfo, ProcessStatus};
use gtop_core::state::app_state::AppState;
use gtop_core::util::format_bytes;

use crate::theme::GuiTheme;

pub fn render(
    ui: &mut Ui,
    state: &AppState,
    theme: &GuiTheme,
    events: &mut Vec<AppEvent>,
) {
    // ── Search bar (WTM style — top of pane) ─────────────────────────────────
    egui::Frame::new()
        .fill(theme.surface)
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(13.0));
                let mut search = state.search.clone();
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut search)
                            .desired_width(240.0)
                            .hint_text("Search")
                            .frame(false),
                    )
                    .changed()
                {
                    events.push(AppEvent::SetSearch(search));
                }
                if !state.search.is_empty() {
                    if ui.small_button("✕").clicked() {
                        events.push(AppEvent::ClearSearch);
                    }
                }
            });
        });

    ui.separator();

    // ── Table ─────────────────────────────────────────────────────────────────
    let available_h = ui.available_height() - 36.0; // reserve bottom bar
    ScrollArea::vertical()
        .id_salt("proc_scroll")
        .max_height(available_h)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Grid::new("proc_grid")
                .num_columns(5)
                .striped(true)
                .min_col_width(0.0)
                .spacing([0.0, 0.0])
                .show(ui, |ui| {
                    // Header row
                    hdr(ui, theme, "Name",    SortColumn::Name,     state, events, 220.0);
                    hdr(ui, theme, "Status",  SortColumn::Status,   state, events, 90.0);
                    hdr(ui, theme, "CPU",     SortColumn::CpuPct,   state, events, 65.0);
                    hdr(ui, theme, "Memory",  SortColumn::MemBytes, state, events, 90.0);
                    hdr(ui, theme, "PID",     SortColumn::Pid,      state, events, 60.0);
                    ui.end_row();

                    // Data rows
                    for (i, proc) in state.processes.iter().enumerate() {
                        proc_row(ui, proc, theme, events, i, i == state.cursor_index);
                        ui.end_row();
                    }
                });
        });

    ui.separator();

    // ── Bottom bar — WTM style: count left, End Task right ────────────────────
    egui::Frame::new()
        .fill(theme.surface)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} processes", state.processes.len()))
                        .color(theme.text_dim)
                        .size(11.0),
                );
                if !state.selected_pids.is_empty() {
                    ui.separator();
                    ui.label(
                        RichText::new(format!("{} selected", state.selected_pids.len()))
                            .color(theme.accent)
                            .size(11.0),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let end_task_btn = egui::Button::new(
                        RichText::new("End Task").size(12.0).color(Color32::WHITE),
                    )
                    .fill(theme.accent)
                    .corner_radius(egui::CornerRadius::ZERO);

                    if ui.add(end_task_btn).clicked() {
                        events.push(AppEvent::RequestKill);
                    }
                });
            });
        });
}

// ── Header cell ───────────────────────────────────────────────────────────────

fn hdr(
    ui: &mut Ui,
    theme: &GuiTheme,
    label: &str,
    col: SortColumn,
    state: &AppState,
    events: &mut Vec<AppEvent>,
    w: f32,
) {
    let active = state.sort_column == col;
    let suffix = if active {
        if state.sort_ascending { " ▲" } else { " ▼" }
    } else { "" };

    let text = RichText::new(format!("{}{}", label, suffix))
        .size(11.0)
        .color(if active { theme.accent } else { theme.text_dim })
        .strong();

    let resp = ui.add_sized(
        [w, 24.0],
        egui::Label::new(text).sense(egui::Sense::click()),
    );
    if resp.clicked() {
        events.push(AppEvent::SetSort(col));
    }
}

// ── Data row ─────────────────────────────────────────────────────────────────

fn proc_row(
    ui: &mut Ui,
    proc: &ProcessInfo,
    theme: &GuiTheme,
    events: &mut Vec<AppEvent>,
    index: usize,
    is_cursor: bool,
) {
    let fg = if proc.is_kernel { theme.kernel_fg } else { theme.text };
    let cpu_color = theme.cpu_color(proc.cpu_pct);

    // Name cell (leftmost, widest)
    let name_resp = ui.add_sized(
        [220.0, 20.0],
        egui::Label::new(
            RichText::new(truncate(&proc.name, 28))
                .color(if is_cursor { Color32::WHITE } else { fg })
                .size(12.0),
        )
        .sense(egui::Sense::click()),
    );
    if is_cursor {
        ui.painter().rect_filled(
            name_resp.rect,
            egui::CornerRadius::ZERO,
            theme.cursor_bg,
        );
    }
    if name_resp.clicked() {
        events.push(AppEvent::SetCursor(index));
    }
    name_resp.context_menu(|ui| {
        ui.label(RichText::new(format!("{} (PID {})", proc.name, proc.pid)).strong());
        ui.separator();
        if ui.button("End Task (SIGKILL)").clicked()   { events.push(AppEvent::RequestKill);      ui.close_menu(); }
        if ui.button("End Process (SIGTERM)").clicked() { events.push(AppEvent::RequestTerminate); ui.close_menu(); }
        if ui.button("Suspend").clicked()               { events.push(AppEvent::RequestPause);     ui.close_menu(); }
        if ui.button("Resume").clicked()                { events.push(AppEvent::RequestResume);    ui.close_menu(); }
        ui.separator();
        if ui.button("Properties").clicked()            { events.push(AppEvent::OpenDetail);       ui.close_menu(); }
    });

    // Status with colored dot
    let (dot, status_text, status_color) = status_info(proc, theme);
    ui.add_sized(
        [90.0, 20.0],
        egui::Label::new(
            RichText::new(format!("{} {}", dot, status_text))
                .color(status_color)
                .size(11.0),
        ),
    );

    // CPU%
    ui.add_sized(
        [65.0, 20.0],
        egui::Label::new(
            RichText::new(format!("{:5.1}%", proc.cpu_pct))
                .color(cpu_color)
                .monospace()
                .size(11.0),
        ),
    );

    // Memory
    ui.add_sized(
        [90.0, 20.0],
        egui::Label::new(
            RichText::new(format!("{:>8}", format_bytes(proc.mem_bytes)))
                .color(theme.text_dim)
                .monospace()
                .size(11.0),
        ),
    );

    // PID
    ui.add_sized(
        [60.0, 20.0],
        egui::Label::new(
            RichText::new(proc.pid.to_string())
                .color(theme.text_dim)
                .monospace()
                .size(11.0),
        ),
    );
}

fn status_info(proc: &ProcessInfo, theme: &GuiTheme) -> (&'static str, &'static str, Color32) {
    match proc.status {
        ProcessStatus::Running   => ("●", "Running",   theme.status_running),
        ProcessStatus::Sleeping  => ("○", "Sleeping",  theme.status_sleeping),
        ProcessStatus::Stopped   => ("⏸", "Suspended", Color32::YELLOW),
        ProcessStatus::Zombie    => ("☠", "Zombie",    Color32::RED),
        ProcessStatus::DiskSleep => ("⌛", "Disk wait", Color32::from_rgb(160, 60, 180)),
        _                        => ("·", "Unknown",   theme.text_dim),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}
