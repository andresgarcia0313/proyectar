//! What the user picks before or during a projection, as in Windows' Win+K panel.

pub use crate::perfil::Perfil;
use serde::{Deserialize, Serialize};

/// Windows' "change mode": trade picture quality for delay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Latencia {
    /// Lowest delay: 720p60, short GOP, almost no buffering.
    Juegos,
    /// Sharp text for documents and code: 1080p30.
    #[default]
    Trabajo,
    /// Smooth motion and the best picture: 1080p60, more bitrate and buffering.
    Videos,
}

/// Windows' Win+P choices that make sense for a wireless display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Fuente {
    /// The same picture as a screen of the laptop.
    #[default]
    Duplicar,
    /// A new virtual screen that extends the desktop onto the TV.
    Extender,
    /// A single window.
    Ventana,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opciones {
    pub latencia: Latencia,
    pub fuente: Fuente,
    /// Send the laptop's sound to the TV. When off, silence is sent: Samsung TVs drop a
    /// session that carries no audio track.
    pub audio: bool,
}

impl Default for Opciones {
    fn default() -> Self {
        Self { latencia: Latencia::default(), fuente: Fuente::default(), audio: true }
    }
}
