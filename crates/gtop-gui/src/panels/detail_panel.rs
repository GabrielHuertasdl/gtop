use egui::{RichText, ScrollArea, Ui};

use gtop_core::process::ProcessInfo;
use gtop_core::util::format_bytes;

use crate::theme::GuiTheme;

pub fn render(ui: &mut Ui, proc: Option<&ProcessInfo>, _theme: &GuiTheme) {
    let Some(p) = proc else {
        ui.label(RichText::new("No process selected").weak());
        return;
    };

    ScrollArea::vertical().show(ui, |ui| {
        ui.heading(RichText::new(&p.name).strong());
        ui.separator();

        egui::Grid::new("detail_grid")
            .num_columns(2)
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                row(ui, "PID", &p.pid.to_string());
                row(ui, "PPID", &p.ppid.to_string());
                row(ui, "User", &p.username);
                row(ui, "Status", p.status_str());
                row(ui, "Priority", &p.priority.to_string());
                row(ui, "Nice", &p.nice.to_string());
                row(ui, "Threads", &p.threads.to_string());
                row(ui, "CPU%", &format!("{:.2}%", p.cpu_pct));
                row(ui, "Memory", &format_bytes(p.mem_bytes));
                row(ui, "Mem%", &format!("{:.2}%", p.mem_pct));
                row(ui, "I/O Read", &format_bytes(p.io_read_bytes));
                row(ui, "I/O Write", &format_bytes(p.io_write_bytes));
                row(ui, "Kernel", if p.is_kernel { "Yes" } else { "No" });
            });

        ui.separator();
        ui.label(RichText::new("Command").strong());
        ui.add(
            egui::TextEdit::multiline(&mut p.cmdline.as_str())
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .code_editor(),
        );
    });
}

fn row(ui: &mut Ui, key: &str, value: &str) {
    ui.label(RichText::new(key).strong());
    ui.label(value);
    ui.end_row();
}
