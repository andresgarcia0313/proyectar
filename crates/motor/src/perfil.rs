//! The encoder and pipeline settings behind each latency mode.

use crate::modo::Latencia;

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
    /// Uncompressed audio when the TV offers it: the TV waits less to keep voice and picture together.
    pub lpcm: bool,
    /// Periodic intra refresh instead of key frames: every frame weighs about the same, so the
    /// decoder buffer can shrink to a few frames (software encoder, scaled on the GPU).
    pub refresco: bool,
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
                lpcm: true,
                refresco: true,
            },
            Latencia::Trabajo => Perfil {
                max_height: 1080,
                max_fps: 30,
                kbps: 10_000,
                cola: 2,
                rapidez: 4,
                cpb_ms: 1000,
                mux_ms: 30,
                lpcm: false,
                refresco: false,
            },
            Latencia::Videos => Perfil {
                max_height: 1080,
                max_fps: 60,
                kbps: 14_000,
                cola: 4,
                rapidez: 4,
                cpb_ms: 2000,
                mux_ms: 40,
                lpcm: false,
                refresco: false,
            },
        }
    }
}
