use nix::sys::signal::Signal;

use crate::process::filter::{FilterConfig, SortColumn};
use crate::process::model::Pid;

#[derive(Debug, Clone)]
pub enum AppEvent {
    // Navigation
    CursorUp,
    CursorDown,
    PageUp,
    PageDown,
    CursorTop,
    CursorBottom,
    SetCursor(usize),

    // Selection
    ToggleSelect,
    SelectAll,
    DeselectAll,
    OpenDetail,
    CloseDetail,

    // Process actions (these produce a PendingAction, waiting for confirm)
    RequestKill,
    RequestTerminate,
    RequestPause,
    RequestResume,
    RequestRenice,
    RequestSignal(Signal),

    // Confirmation dialog
    Confirm,
    Cancel,
    SetReniceValue(i32),

    // View
    ToggleViewMode,
    ToggleKernelThreads,
    ToggleDetail,
    SetSort(SortColumn),
    ToggleSortDirection,

    // Search / filter
    SetSearch(String),
    ClearSearch,
    SetFilter(FilterConfig),
    ClearFilter,

    // Meta
    Quit,
    Refresh,
}
