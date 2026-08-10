pub mod core {
    pub use tui_generator_core::*;
}

#[cfg(feature = "ratatui")]
pub mod ratatui {
    pub use tui_generator_ratatui::*;
}

#[cfg(feature = "clap")]
pub mod clap {
    pub use ::clap::*;
}

pub use tui_generator_core::*;

#[cfg(feature = "ratatui")]
pub use tui_generator_ratatui::RatatuiRenderer;

#[cfg(feature = "clap")]
pub use tui_generator_macros::Tui;
