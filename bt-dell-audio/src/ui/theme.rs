#![allow(dead_code)]

pub mod symbols {
    pub const CHECK: &str = "✓";
    pub const WARN: &str = "⚠";
    pub const ERROR: &str = "✖";
    pub const BULLET: &str = "•";
    pub const ARROW: &str = "→";
}

pub mod borders {
    pub const TOP: &str = "╔═══════════════════════════════════════════════════════════════════╗";
    pub const BOTTOM: &str = "╚═══════════════════════════════════════════════════════════════════╝";
    pub const DIVIDER: &str = "───────────────────────────────────────────────────────────────────";
    pub const DOUBLE_DIVIDER: &str = "═══════════════════════════════════════════════════════════════════";
}
