use egui::{Color32, Visuals};

pub struct GuiTheme {
    pub bg:       Color32,
    pub surface:  Color32,
    pub surface2: Color32,
    pub border:   Color32,

    pub text:        Color32,
    pub text_dim:    Color32,
    pub text_strong: Color32,

    pub accent: Color32,

    pub cpu_low:  Color32,
    pub cpu_med:  Color32,
    pub cpu_high: Color32,
    pub cpu_crit: Color32,

    pub mem_color:  Color32,
    pub swap_color: Color32,
    pub net_rx: Color32,
    pub net_tx: Color32,

    pub temp_ok:   Color32,
    pub temp_warm: Color32,
    pub temp_hot:  Color32,

    pub bat_ok:   Color32,
    pub bat_low:  Color32,
    pub bat_crit: Color32,

    pub danger:      Color32,
    pub kernel_fg:   Color32,
    pub selected_bg: Color32,
    pub cursor_bg:   Color32,

    pub status_running:  Color32,
    pub status_sleeping: Color32,
}

impl Default for GuiTheme {
    fn default() -> Self {
        GuiTheme {
            // Windows 11 Task Manager dark
            bg:       Color32::from_rgb(0x20, 0x20, 0x20),
            surface:  Color32::from_rgb(0x2b, 0x2b, 0x2b),
            surface2: Color32::from_rgb(0x33, 0x33, 0x33),
            border:   Color32::from_rgb(0x3d, 0x3d, 0x3d),

            text:        Color32::from_rgb(0xff, 0xff, 0xff),
            text_dim:    Color32::from_rgb(0xab, 0xab, 0xab),
            text_strong: Color32::from_rgb(0xff, 0xff, 0xff),

            accent: Color32::from_rgb(0x00, 0x78, 0xd4), // #0078d4 Windows blue

            cpu_low:  Color32::from_rgb(0x6c, 0xcb, 0x5f), // green
            cpu_med:  Color32::from_rgb(0xf0, 0xbe, 0x54), // amber
            cpu_high: Color32::from_rgb(0xe0, 0x7b, 0x39), // orange
            cpu_crit: Color32::from_rgb(0xf4, 0x48, 0x47), // red

            mem_color:  Color32::from_rgb(0x9b, 0x59, 0xb6), // purple (WTM memory color)
            swap_color: Color32::from_rgb(0xc5, 0x86, 0xc0),
            net_rx: Color32::from_rgb(0x6c, 0xcb, 0x5f),
            net_tx: Color32::from_rgb(0xf4, 0x48, 0x47),

            temp_ok:   Color32::from_rgb(0x6c, 0xcb, 0x5f),
            temp_warm: Color32::from_rgb(0xf0, 0xbe, 0x54),
            temp_hot:  Color32::from_rgb(0xf4, 0x48, 0x47),

            bat_ok:   Color32::from_rgb(0x6c, 0xcb, 0x5f),
            bat_low:  Color32::from_rgb(0xf0, 0xbe, 0x54),
            bat_crit: Color32::from_rgb(0xf4, 0x48, 0x47),

            danger:      Color32::from_rgb(0xf4, 0x48, 0x47),
            kernel_fg:   Color32::from_rgb(0x55, 0x55, 0x55),
            selected_bg: Color32::from_rgba_premultiplied(0x00, 0x78, 0xd4, 0x40),
            cursor_bg:   Color32::from_rgba_premultiplied(0x00, 0x78, 0xd4, 0x28),

            status_running:  Color32::from_rgb(0x6c, 0xcb, 0x5f),
            status_sleeping: Color32::from_rgb(0x6e, 0x6e, 0x6e),
        }
    }
}

impl GuiTheme {
    pub fn cpu_color(&self, pct: f32) -> Color32 {
        if pct < 40.0      { self.cpu_low }
        else if pct < 70.0 { self.cpu_med }
        else if pct < 90.0 { self.cpu_high }
        else               { self.cpu_crit }
    }

    pub fn temp_color(&self, celsius: f32) -> Color32 {
        if celsius > 85.0      { self.temp_hot }
        else if celsius > 70.0 { self.temp_warm }
        else                   { self.temp_ok }
    }

    pub fn bat_color(&self, pct: u32) -> Color32 {
        if pct < 20      { self.bat_crit }
        else if pct < 50 { self.bat_low }
        else             { self.bat_ok }
    }

    pub fn apply_visuals(&self, ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        visuals.window_fill    = self.bg;
        visuals.panel_fill     = self.bg;
        visuals.extreme_bg_color = self.surface;

        // Widget fills
        visuals.widgets.noninteractive.bg_fill = self.surface;
        visuals.widgets.inactive.bg_fill       = self.surface2;
        visuals.widgets.hovered.bg_fill        = self.surface2;

        // Text color
        visuals.widgets.noninteractive.fg_stroke.color = self.text;

        // Borders
        visuals.window_stroke.color = self.border;
        visuals.widgets.noninteractive.bg_stroke.color = self.border;

        // Selection / accent
        visuals.hyperlink_color      = self.accent;
        visuals.selection.bg_fill    = self.selected_bg;
        visuals.selection.stroke.color = self.accent;

        // Flat corners — Windows Task Manager has no rounded corners
        let zero = egui::CornerRadius::ZERO;
        visuals.menu_corner_radius = zero;
        for w in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            w.corner_radius = zero;
        }

        ctx.set_visuals(visuals);
    }
}
