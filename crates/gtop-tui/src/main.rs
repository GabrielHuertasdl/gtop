mod app;
mod layout;
mod theme;
mod widgets;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use theme::Theme;

#[derive(Parser, Debug)]
#[command(name = "gtop-tui", about = "Real-time Linux process manager TUI", version)]
pub struct Args {
    /// Refresh interval in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    pub interval: u64,

    /// Start in tree view mode
    #[arg(short, long)]
    pub tree: bool,

    /// Show kernel threads on startup
    #[arg(short, long)]
    pub kernel: bool,

    /// Color theme: default, catppuccin-mocha, gruvbox, nord
    #[arg(long, default_value = "default")]
    pub theme: String,

    /// Disable all colors (monochrome output)
    #[arg(long)]
    pub no_color: bool,

    /// Use ASCII-only characters (no Unicode block/braille chars)
    #[arg(long)]
    pub ascii: bool,

    /// Print shell completions and exit (bash/zsh/fish/elvish/powershell)
    #[arg(long, value_name = "SHELL")]
    pub completions: Option<String>,

    /// Print man page to stdout and exit
    #[arg(long)]
    pub man: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle non-interactive output modes
    if args.man {
        print_man()?;
        return Ok(());
    }

    if let Some(ref shell) = args.completions {
        print_completions(shell)?;
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    // Build theme from CLI args
    let theme = Theme::from_name(&args.theme, args.ascii, args.no_color);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Restore terminal on panic
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            std::io::stdout(),
            LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
        orig_hook(info);
    }));

    let result = app::App::with_theme(theme, args.tree, args.kernel)?.run(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn print_man() -> Result<()> {
    // Simple man page output without clap_mangen dependency
    print!(r#".TH GTOP-TUI 1 "2026" "gtop 0.1.0" "User Commands"
.SH NAME
gtop-tui \- real-time Linux process manager (TUI)
.SH SYNOPSIS
.B gtop-tui
[\fIOPTIONS\fR]
.SH DESCRIPTION
gtop-tui is a real-time Linux process manager with a terminal user interface.
It monitors CPU, RAM, swap, disk, network, temperature, battery, and GPU.
.SH OPTIONS
.TP
\fB\-i\fR, \fB\-\-interval\fR \fIMILLIS\fR
Refresh interval in milliseconds (default: 1000)
.TP
\fB\-t\fR, \fB\-\-tree\fR
Start in process tree view
.TP
\fB\-k\fR, \fB\-\-kernel\fR
Show kernel threads on startup
.TP
\fB\-\-theme\fR \fINAME\fR
Color theme: default, catppuccin-mocha, gruvbox, nord
.TP
\fB\-\-no-color\fR
Disable all colors
.TP
\fB\-\-ascii\fR
Use ASCII characters only (no Unicode blocks)
.SH KEYBINDINGS
.TP
\fBj/k\fR or \fB↑/↓\fR
Move cursor
.TP
\fBSpace\fR
Toggle process selection
.TP
\fBX\fR
Kill selected process (SIGKILL)
.TP
\fBt\fR
Terminate selected process (SIGTERM)
.TP
\fBp\fR / \fBr\fR
Pause / Resume (SIGSTOP / SIGCONT)
.TP
\fB/\fR
Search
.TP
\fBT\fR
Toggle tree view
.TP
\fBEnter\fR / \fBd\fR
Toggle detail panel
.TP
\fB1-8\fR
Sort by column
.TP
\fBI\fR
Toggle sort direction
.TP
\fBi\fR
Toggle kernel threads
.TP
\fBq\fR / \fBCtrl-C\fR
Quit
.SH AUTHOR
Gabriel Huertas <gabriel.huertasdl@gmail.com>
"#);
    Ok(())
}

fn print_completions(shell: &str) -> Result<()> {
    // Minimal completion hints (full clap_complete integration would need build dep)
    let script = match shell.to_lowercase().as_str() {
        "bash" => r#"# gtop-tui bash completion
_gtop_tui() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local opts="--interval --tree --kernel --theme --no-color --ascii --completions --man --help --version"
    case "$cur" in
        --theme) COMPREPLY=($(compgen -W "default catppuccin-mocha gruvbox nord" -- "${COMP_WORDS[COMP_CWORD+1]}")); return ;;
        --completions) COMPREPLY=($(compgen -W "bash zsh fish" -- "${COMP_WORDS[COMP_CWORD+1]}")); return ;;
    esac
    COMPREPLY=($(compgen -W "$opts" -- "$cur"))
}
complete -F _gtop_tui gtop-tui
"#,
        "zsh" => r#"#compdef gtop-tui
_gtop_tui() {
    _arguments \
        '(-i --interval)'{-i,--interval}'[refresh interval ms]:ms:' \
        '(-t --tree)'{-t,--tree}'[tree view]' \
        '(-k --kernel)'{-k,--kernel}'[show kernel threads]' \
        '--theme[color theme]:theme:(default catppuccin-mocha gruvbox nord)' \
        '--no-color[disable colors]' \
        '--ascii[ascii only]' \
        '--completions[shell completions]:shell:(bash zsh fish)' \
        '--man[print man page]'
}
_gtop_tui
"#,
        "fish" => r#"# gtop-tui fish completion
complete -c gtop-tui -l interval -s i -d 'refresh interval ms'
complete -c gtop-tui -l tree    -s t -d 'tree view'
complete -c gtop-tui -l kernel  -s k -d 'show kernel threads'
complete -c gtop-tui -l theme        -d 'color theme' -a 'default catppuccin-mocha gruvbox nord'
complete -c gtop-tui -l no-color     -d 'disable colors'
complete -c gtop-tui -l ascii        -d 'ascii only'
"#,
        _ => {
            eprintln!("Unknown shell: {}. Supported: bash, zsh, fish", shell);
            return Ok(());
        }
    };
    print!("{}", script);
    Ok(())
}
