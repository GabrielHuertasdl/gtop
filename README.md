# gtop

Real-time Linux process manager with TUI and GUI interfaces, sharing a common core.

```
╔══ GTOP ══╗  v0.1.0  CPU ████░░░░  24%   RAM ███░░░░░  38%   Load 1.2 0.9 0.8
```

## Features

- **TUI** (`gtop-tui`): ratatui-based terminal UI with 4 color themes, per-core CPU grid, per-process sparklines, adaptive compact/full layout
- **GUI** (`gtop-gui`): egui/eframe desktop UI with card-based system strip, inline progress bars, real-time updates
- **Shared core** (`gtop-core`): procfs-based metrics (CPU, RAM, swap, disk, net, temp, battery, GPU), process tree, sort/filter, signal dispatch
- Process tree view, multi-select batch actions (kill/term/pause/resume/renice)
- Search by name or PID, kernel thread toggle
- Per-process CPU history sparklines (`▁▂▃▄▅▆▇█`)

## Installation

### From source

```bash
git clone https://github.com/gabriel-huertas/gtop
cd gtop
cargo build --release
```

Binaries land in `target/release/gtop-tui` and `target/release/gtop-gui`.

### AUR (Arch Linux)

```bash
yay -S gtop-tui gtop-gui
# or
git clone https://aur.archlinux.org/gtop-tui.git && cd gtop-tui && makepkg -si
```

## Usage

```
gtop-tui [OPTIONS]

Options:
  -i, --interval <MILLIS>    Refresh interval (default: 1000)
  -t, --tree                 Start in tree view
  -k, --kernel               Show kernel threads on startup
      --theme <NAME>         Theme: default | catppuccin-mocha | gruvbox | nord
      --no-color             Monochrome output
      --ascii                ASCII-only characters
      --completions <SHELL>  Print shell completions (bash/zsh/fish) and exit
      --man                  Print man page and exit
```

```
gtop-gui [OPTIONS]

Options:
  -i, --interval <MILLIS>    Refresh interval (default: 1000)
```

## TUI Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Cursor down |
| `k` / `↑` | Cursor up |
| `Ctrl-D` / `PgDn` | Page down |
| `Ctrl-U` / `PgUp` | Page up |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |
| `Space` | Toggle select |
| `a` / `A` | Select all / Deselect all |
| `Enter` / `d` | Toggle detail panel |
| `X` | Kill selected (SIGKILL) |
| `t` | Terminate selected (SIGTERM) |
| `p` / `r` | Pause / Resume |
| `/` | Search |
| `T` | Toggle tree/list view |
| `i` | Toggle kernel threads |
| `I` | Toggle sort direction |
| `1`–`8` | Sort by column (PID/USER/CPU/MEM/RSS/NI/S/NAME) |
| `F5` | Force refresh |
| `q` / `Ctrl-C` | Quit |

## Themes

- `default` — dark teal accent (`#00d4aa`)
- `catppuccin-mocha` — Catppuccin Mocha palette
- `gruvbox` — Gruvbox Dark palette
- `nord` — Nord palette

```bash
gtop-tui --theme catppuccin-mocha
```

## Shell Completions

```bash
# Bash
gtop-tui --completions bash >> ~/.bash_completion

# Zsh
gtop-tui --completions zsh > ~/.zfunc/_gtop-tui

# Fish
gtop-tui --completions fish > ~/.config/fish/completions/gtop-tui.fish
```

## Workspace Structure

```
gtop/
├── crates/
│   ├── gtop-core/     # shared metrics, process, state, IPC
│   ├── gtop-tui/      # ratatui terminal UI
│   └── gtop-gui/      # egui desktop UI
├── packaging/
│   ├── aur/           # AUR PKGBUILD + .desktop files
│   └── appimage/      # AppImage build scripts
└── assets/
    └── icons/         # SVG icon
```

## Requirements

- Linux kernel 3.2+ (procfs)
- Rust 1.78+
- For GPU metrics: NVIDIA driver with `/proc/driver/nvidia` or AMD with `/sys/class/drm`

## License

MIT
