use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub dim_fg: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub header_fg: Color,
    pub border: Color,
    pub accent: Color,
    pub logo_fg: Color,
    pub logo_bg: Color,

    pub cpu_low: Color,
    pub cpu_med: Color,
    pub cpu_high: Color,
    pub cpu_crit: Color,

    pub mem_color: Color,
    pub swap_color: Color,

    pub net_rx: Color,
    pub net_tx: Color,

    pub status_run: Color,
    pub status_sleep: Color,
    pub status_stop: Color,
    pub status_zombie: Color,
    pub status_disk: Color,

    pub selected_bg: Color,
    pub search_active: Color,

    pub confirm_danger_fg: Color,

    pub ascii: bool,
    pub no_color: bool,
}

impl Default for Theme {
    fn default() -> Self {
        default_theme()
    }
}

impl Theme {
    pub fn from_name(name: &str, ascii: bool, no_color: bool) -> Self {
        let mut t = match name {
            "catppuccin" | "catppuccin-mocha" | "mocha" => catppuccin_mocha(),
            "gruvbox" | "gruvbox-dark" => gruvbox_dark(),
            "nord" => nord(),
            _ => default_theme(),
        };
        t.ascii = ascii;
        t.no_color = no_color;
        if no_color {
            // flatten to grayscale
            t.cpu_low = Color::White;
            t.cpu_med = Color::White;
            t.cpu_high = Color::White;
            t.cpu_crit = Color::White;
            t.mem_color = Color::White;
            t.swap_color = Color::White;
            t.net_rx = Color::White;
            t.net_tx = Color::White;
            t.header_fg = Color::White;
            t.accent = Color::White;
            t.logo_fg = Color::Reset;
            t.logo_bg = Color::Reset;
            t.status_run = Color::White;
            t.status_sleep = Color::White;
            t.status_stop = Color::White;
            t.status_zombie = Color::White;
            t.status_disk = Color::White;
            t.selected_bg = Color::DarkGray;
            t.search_active = Color::White;
            t.border = Color::DarkGray;
            t.dim_fg = Color::DarkGray;
        }
        t
    }

    pub fn cpu_color(&self, pct: f32) -> Color {
        if pct < 40.0 {
            self.cpu_low
        } else if pct < 70.0 {
            self.cpu_med
        } else if pct < 90.0 {
            self.cpu_high
        } else {
            self.cpu_crit
        }
    }

    /// Full-intensity style for a process row cell (applies modifiers + color).
    /// is_cursor / is_selected handled by the Table widget's highlight_style, not here.
    pub fn process_name_style(&self, cpu_pct: f32, is_kernel: bool) -> Style {
        let mut s = Style::default().fg(self.fg);
        if is_kernel {
            s = s.fg(self.dim_fg).add_modifier(Modifier::DIM);
            return s;
        }
        if cpu_pct > 80.0 {
            s = s.fg(self.cpu_crit).add_modifier(Modifier::BOLD);
        } else if cpu_pct > 50.0 {
            s = s.fg(self.cpu_high).add_modifier(Modifier::BOLD);
        } else if cpu_pct < 0.1 {
            s = s.fg(self.dim_fg);
        }
        s
    }

    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.header_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.dim_fg)
    }

    pub fn selected_style(&self) -> Style {
        Style::default()
            .bg(self.selected_bg)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .bg(self.highlight_bg)
            .fg(self.highlight_fg)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn accent_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn logo_style(&self) -> Style {
        Style::default()
            .fg(self.logo_fg)
            .bg(self.logo_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// A filled block char appropriate for bars (respects ascii mode).
    pub fn block_full(&self) -> char {
        if self.ascii { '#' } else { '█' }
    }

    /// An empty block char for bar background.
    pub fn block_empty(&self) -> char {
        if self.ascii { '-' } else { '░' }
    }

    /// A half-filled block for smoother bars.
    pub fn block_half(&self) -> char {
        if self.ascii { '+' } else { '▓' }
    }
}

// ─── Palettes ────────────────────────────────────────────────────────────────

fn default_theme() -> Theme {
    Theme {
        bg: Color::Reset,
        fg: Color::White,
        dim_fg: Color::Rgb(110, 120, 140),
        highlight_bg: Color::Rgb(0, 70, 90),
        highlight_fg: Color::White,
        header_fg: Color::Rgb(0, 212, 170),
        border: Color::Rgb(75, 85, 105),
        accent: Color::Rgb(0, 212, 170),
        logo_fg: Color::Black,
        logo_bg: Color::Rgb(0, 212, 170),

        cpu_low: Color::Rgb(80, 220, 130),
        cpu_med: Color::Rgb(240, 190, 60),
        cpu_high: Color::Rgb(240, 120, 40),
        cpu_crit: Color::Rgb(220, 50, 60),

        mem_color: Color::Rgb(80, 140, 220),
        swap_color: Color::Rgb(170, 80, 200),

        net_rx: Color::Rgb(80, 220, 130),
        net_tx: Color::Rgb(220, 80, 80),

        status_run: Color::Rgb(80, 220, 130),
        status_sleep: Color::Rgb(80, 140, 220),
        status_stop: Color::Rgb(240, 190, 60),
        status_zombie: Color::Rgb(220, 50, 60),
        status_disk: Color::Rgb(170, 80, 200),

        selected_bg: Color::Rgb(30, 60, 120),
        search_active: Color::Rgb(240, 190, 60),

        confirm_danger_fg: Color::Rgb(220, 50, 60),

        ascii: false,
        no_color: false,
    }
}

fn catppuccin_mocha() -> Theme {
    Theme {
        bg: Color::Rgb(30, 30, 46),
        fg: Color::Rgb(205, 214, 244),
        dim_fg: Color::Rgb(108, 112, 134),
        highlight_bg: Color::Rgb(69, 71, 90),
        highlight_fg: Color::Rgb(205, 214, 244),
        header_fg: Color::Rgb(137, 180, 250),
        border: Color::Rgb(88, 91, 112),
        accent: Color::Rgb(116, 199, 236),
        logo_fg: Color::Rgb(30, 30, 46),
        logo_bg: Color::Rgb(116, 199, 236),

        cpu_low: Color::Rgb(166, 227, 161),
        cpu_med: Color::Rgb(249, 226, 175),
        cpu_high: Color::Rgb(250, 179, 135),
        cpu_crit: Color::Rgb(243, 139, 168),

        mem_color: Color::Rgb(137, 180, 250),
        swap_color: Color::Rgb(203, 166, 247),

        net_rx: Color::Rgb(166, 227, 161),
        net_tx: Color::Rgb(243, 139, 168),

        status_run: Color::Rgb(166, 227, 161),
        status_sleep: Color::Rgb(137, 180, 250),
        status_stop: Color::Rgb(249, 226, 175),
        status_zombie: Color::Rgb(243, 139, 168),
        status_disk: Color::Rgb(203, 166, 247),

        selected_bg: Color::Rgb(49, 50, 68),
        search_active: Color::Rgb(249, 226, 175),

        confirm_danger_fg: Color::Rgb(243, 139, 168),

        ascii: false,
        no_color: false,
    }
}

fn gruvbox_dark() -> Theme {
    Theme {
        bg: Color::Rgb(40, 40, 40),
        fg: Color::Rgb(235, 219, 178),
        dim_fg: Color::Rgb(146, 131, 116),
        highlight_bg: Color::Rgb(80, 73, 69),
        highlight_fg: Color::Rgb(235, 219, 178),
        header_fg: Color::Rgb(131, 165, 152),
        border: Color::Rgb(80, 73, 69),
        accent: Color::Rgb(142, 192, 124),
        logo_fg: Color::Rgb(40, 40, 40),
        logo_bg: Color::Rgb(142, 192, 124),

        cpu_low: Color::Rgb(184, 187, 38),
        cpu_med: Color::Rgb(250, 189, 47),
        cpu_high: Color::Rgb(254, 128, 25),
        cpu_crit: Color::Rgb(251, 73, 52),

        mem_color: Color::Rgb(131, 165, 152),
        swap_color: Color::Rgb(211, 134, 155),

        net_rx: Color::Rgb(184, 187, 38),
        net_tx: Color::Rgb(251, 73, 52),

        status_run: Color::Rgb(184, 187, 38),
        status_sleep: Color::Rgb(131, 165, 152),
        status_stop: Color::Rgb(250, 189, 47),
        status_zombie: Color::Rgb(251, 73, 52),
        status_disk: Color::Rgb(211, 134, 155),

        selected_bg: Color::Rgb(60, 56, 54),
        search_active: Color::Rgb(250, 189, 47),

        confirm_danger_fg: Color::Rgb(251, 73, 52),

        ascii: false,
        no_color: false,
    }
}

fn nord() -> Theme {
    Theme {
        bg: Color::Rgb(46, 52, 64),
        fg: Color::Rgb(216, 222, 233),
        dim_fg: Color::Rgb(76, 86, 106),
        highlight_bg: Color::Rgb(67, 76, 94),
        highlight_fg: Color::Rgb(236, 239, 244),
        header_fg: Color::Rgb(136, 192, 208),
        border: Color::Rgb(59, 66, 82),
        accent: Color::Rgb(143, 188, 187),
        logo_fg: Color::Rgb(46, 52, 64),
        logo_bg: Color::Rgb(136, 192, 208),

        cpu_low: Color::Rgb(163, 190, 140),
        cpu_med: Color::Rgb(235, 203, 139),
        cpu_high: Color::Rgb(208, 135, 112),
        cpu_crit: Color::Rgb(191, 97, 106),

        mem_color: Color::Rgb(129, 161, 193),
        swap_color: Color::Rgb(180, 142, 173),

        net_rx: Color::Rgb(163, 190, 140),
        net_tx: Color::Rgb(191, 97, 106),

        status_run: Color::Rgb(163, 190, 140),
        status_sleep: Color::Rgb(129, 161, 193),
        status_stop: Color::Rgb(235, 203, 139),
        status_zombie: Color::Rgb(191, 97, 106),
        status_disk: Color::Rgb(180, 142, 173),

        selected_bg: Color::Rgb(59, 66, 82),
        search_active: Color::Rgb(235, 203, 139),

        confirm_danger_fg: Color::Rgb(191, 97, 106),

        ascii: false,
        no_color: false,
    }
}
