pub const INIT: &[u8] = &[0x1B, 0x40]; // ESC @

pub const ALIGN_LEFT: &[u8] = &[0x1B, 0x61, 0x00]; // ESC a 0
pub const ALIGN_CENTER: &[u8] = &[0x1B, 0x61, 0x01]; // ESC a 1
pub const ALIGN_RIGHT: &[u8] = &[0x1B, 0x61, 0x02]; // ESC a 2

pub const BOLD_ON: &[u8] = &[0x1B, 0x45, 0x01]; // ESC E 1
pub const BOLD_OFF: &[u8] = &[0x1B, 0x45, 0x00]; // ESC E 0

pub const UNDERLINE_ON: &[u8] = &[0x1B, 0x2D, 0x01]; // ESC - 1
pub const UNDERLINE_OFF: &[u8] = &[0x1B, 0x2D, 0x00]; // ESC - 0

pub const CUT_FULL: &[u8] = &[0x1D, 0x56, 0x00]; // GS V 0
pub const CUT_PARTIAL: &[u8] = &[0x1D, 0x56, 0x01]; // GS V 1

pub const CASH_DRAWER_PIN2: &[u8] = &[0x1B, 0x70, 0x00, 0x19, 0x78]; // ESC p 0 — pino 2
pub const CASH_DRAWER_PIN5: &[u8] = &[0x1B, 0x70, 0x01, 0x19, 0x78]; // ESC p 1 — pino 5
