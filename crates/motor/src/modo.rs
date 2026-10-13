//! What the user picks before or during a projection, as in Windows' Win+K panel.

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

/// Encoder and pipeline settings for a latency mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Perfil {
    pub max_height: u32,
    pub max_fps: u32,
    pub kbps: u32,
    /// Frames queued before the encoder; older ones are dropped, never delayed.
    pub cola: u32,
    /// GPU encoder speed (VA-API target usage): 7 is fastest, 4 balances speed and detail.
    pub rapidez: u32,
    /// Decoder buffer (H.264 CPB/VBV) in ms of bitrate. The TV fills it before showing a frame,
    /// so it is the largest delay we control. Below ~500 ms the Intel driver skips frames on busy
    /// content (measured), so Juegos stops there.
    pub cpb_ms: u32,
    /// How long the MPEG-TS muxer waits to interleave audio and video, in ms.
    pub mux_ms: u32,
}

impl Latencia {
    pub fn perfil(self) -> Perfil {
        match self {
            Latencia::Juegos => Perfil {
                max_height: 720,
                max_fps: 60,
                kbps: 8_000,
                cola: 1,
                rapidez: 7,
                cpb_ms: 500,
                mux_ms: 20,
            },
            Latencia::Trabajo => Perfil {
                max_height: 1080,
                max_fps: 30,
                kbps: 10_000,
                cola: 2,
                rapidez: 4,
                cpb_ms: 1000,
                mux_ms: 30,
            },
            Latencia::Videos => Perfil {
                max_height: 1080,
                max_fps: 60,
                kbps: 14_000,
                cola: 4,
                rapidez: 4,
                cpb_ms: 2000,
                mux_ms: 40,
            },
        }
    }
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
