use crate::prelude::*;

// ── Amethyst / Violet Palette ──────────────────────────────────────────────────
// '50':  #f5f3ff  (0.961, 0.953, 1.000)
// '100': #ede9fe  (0.929, 0.914, 0.996)
// '200': #ddd6fe  (0.867, 0.839, 0.996)
// '300': #c4b5fd  (0.769, 0.710, 0.992)
// '400': #a78bfa  (0.655, 0.545, 0.980)
// '500': #8b5cf6  (0.545, 0.361, 0.965)
// '600': #7c3aed  (0.486, 0.227, 0.929)
// '700': #6d28d9  (0.427, 0.157, 0.851)
// '800': #5b21b6  (0.357, 0.129, 0.714)
// '900': #4c1d95  (0.298, 0.114, 0.584)
// '950': #2e1065  (0.180, 0.063, 0.396)

// Backgrounds
pub const COLOR_BG_BASE: Color = Color::srgb(0.051, 0.047, 0.086); // #0d0c16 — root / content
pub const COLOR_BG_SURFACE: Color = Color::srgb(0.051, 0.047, 0.086); // #0d0c16 — header / sidebar
pub const COLOR_PANEL_BG: Color = Color::srgba(0.15, 0.15, 0.29, 0.95);
pub const COLOR_HEADER_BG: Color = Color::srgba(0.10, 0.10, 0.16, 1.0);

// Labels
pub const COLOR_TITLE: Color = Color::srgba(0.75, 0.80, 1.0, 0.55);
pub const COLOR_LABEL: Color = Color::srgba(0.90, 0.92, 1.0, 0.95);
pub const COLOR_LABEL_SECONDARY: Color = Color::srgba(0.65, 0.70, 0.90, 0.70);
pub const COLOR_LABEL_TERTIARY: Color = Color::srgba(0.55, 0.60, 0.80, 0.50);
pub const COLOR_LABEL_DISABLED: Color = Color::srgba(0.40, 0.42, 0.55, 0.35);

// State buttons (manager panels)
pub const COLOR_ACTIVE: Color = Color::srgb(0.31, 0.59, 0.40); // green — active state
pub const COLOR_HOVER: Color = Color::srgb(0.22, 0.22, 0.32); // dark blue-gray — hover
pub const COLOR_INACTIVE: Color = Color::srgb(0.13, 0.13, 0.20); // very dark blue — inactive
pub const COLOR_DISABLED: Color = Color::srgb(0.18, 0.18, 0.18); // dark gray — disabled

// Pause / run toggle button
pub const COLOR_RUNNING: Color = Color::srgba(0.15, 0.15, 0.29, 0.95);
// 600
pub const COLOR_RUNNING_HOVER: Color = Color::srgba(0.20, 0.20, 0.35, 0.95); // 500
pub const COLOR_PAUSED: Color = Color::srgb(0.655, 0.545, 0.980); // 400
pub const COLOR_PAUSED_HOVER: Color = Color::srgb(0.769, 0.710, 0.992); // 300

// Menu sidebar buttons
pub const COLOR_MENU_ACTIVE: Color = Color::srgba(0.15, 0.15, 0.29, 0.95); // 400 @ 25%
pub const COLOR_MENU_HOVER: Color = Color::srgba(0.655, 0.545, 0.980, 0.12); // 400 @ 12%
pub const COLOR_MENU_NORMAL: Color = Color::srgba(0.0, 0.0, 0.0, 0.0); // transparent

// Input fields
pub const COLOR_INPUT_BG: Color = Color::srgba(0.08, 0.08, 0.14, 1.0);
pub const COLOR_INPUT_BORDER: Color = Color::srgba(0.28, 0.32, 0.52, 0.70);
pub const COLOR_INPUT_TEXT: Color = Color::srgba(0.90, 0.92, 1.0, 0.95);
pub const COLOR_INPUT_BG_DISABLED: Color = Color::srgba(0.06, 0.06, 0.10, 0.40);

// Disabled panel / header
pub const COLOR_PANEL_BG_DISABLED: Color = Color::srgba(0.06, 0.06, 0.10, 0.50);
pub const COLOR_HEADER_BG_DISABLED: Color = Color::srgba(0.10, 0.10, 0.16, 0.50);

// Action buttons (Add, Submit, etc.)
pub const COLOR_BUTTON_BG: Color = Color::srgba(0.18, 0.22, 0.42, 1.0);
pub const COLOR_BUTTON_HOVER: Color = Color::srgba(0.25, 0.30, 0.55, 1.0);
pub const COLOR_BUTTON_TEXT: Color = Color::srgba(1.0, 1.0, 1.0, 0.9);

// Hint / code reference box
pub const COLOR_HINT_BG: Color = Color::srgba(0.10, 0.12, 0.20, 0.80);
pub const COLOR_SYNTAX_WITH: Color = Color::srgba(0.60, 0.85, 0.60, 0.90); // green — With<>
pub const COLOR_SYNTAX_WITHOUT: Color = Color::srgba(0.85, 0.55, 0.55, 0.90); // muted red — Without<>

// Destructive (close / delete actions) — Apple systemRed dark mode, muted
#[allow(dead_code)]
pub const COLOR_DESTRUCTIVE: Color = Color::srgba(0.78, 0.18, 0.15, 1.0); // #C72E26 — base
pub const COLOR_DESTRUCTIVE_HOVER: Color = Color::srgba(1.0, 0.271, 0.227, 0.85); // #FF453A @ 85%

// Separator
pub const COLOR_SEPARATOR: Color = Color::srgba(0.28, 0.32, 0.52, 0.45);

// Scrollbar
pub const COLOR_SCROLLBAR_TRACK: Color = Color::srgba(0.08, 0.08, 0.14, 0.80);
pub const COLOR_SCROLLBAR_THUMB: Color = Color::srgba(0.40, 0.45, 0.70, 0.80);

// Entity row selection
pub const COLOR_ROW_SELECTED: Color = Color::srgba(0.545, 0.361, 0.965, 0.25); // violet 500 @ 25%
pub const COLOR_ROW_HOVER: Color = Color::srgba(0.545, 0.361, 0.965, 0.10); // violet 500 @ 10%

// Overlay (disconnected / blocking screens)
pub const COLOR_OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
pub const COLOR_OVERLAY_TEXT: Color = Color::srgba(0.75, 0.75, 0.75, 1.0);
