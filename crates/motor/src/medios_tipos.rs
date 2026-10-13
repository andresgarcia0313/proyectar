//! Where the picture comes from and the encoder fallback chain. The chain covers the
//! hardware of most computers; encoders this one lacks are skipped.

use std::os::fd::OwnedFd;
use wfd::VideoMode;

pub enum Origen {
    /// PipeWire node to read; through the portal's descriptor when it gave one (KWin does not).
    PipeWire { fd: Option<OwnedFd>, nodo: u32 },
    /// Live test pattern, for tests.
    Prueba,
}

/// Encoders, best first: GPUs (lowest delay, no CPU), then software.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codificador {
    /// Intel (Gen 9 and later) low-power VA-API engine.
    Gpu,
    /// VA-API on AMD or older Intel.
    GpuVa,
    Nvidia,
    /// Raspberry Pi and other ARM boards.
    V4l2,
    OpenH264,
    X264,
}

const ORDEN: [Codificador; 6] = [
    Codificador::Gpu,
    Codificador::GpuVa,
    Codificador::Nvidia,
    Codificador::V4l2,
    Codificador::OpenH264,
    Codificador::X264,
];

impl Codificador {
    /// The best encoder this computer has.
    pub fn primero() -> Self {
        ORDEN.into_iter().find(|c| c.disponible()).unwrap_or(Self::X264)
    }

    /// The next encoder in the chain that this computer has, if any.
    pub fn siguiente(self) -> Option<Self> {
        let resto = ORDEN.iter().skip_while(|c| **c != self).skip(1);
        resto.copied().find(|c| c.disponible())
    }

    pub fn disponible(self) -> bool {
        gstreamer::init().is_ok() && gstreamer::ElementFactory::find(self.nombre()).is_some()
    }

    pub fn es_gpu(self) -> bool {
        !matches!(self, Self::OpenH264 | Self::X264)
    }

    pub fn nombre(self) -> &'static str {
        match self {
            Self::Gpu => "vah264lpenc",
            Self::GpuVa => "vah264enc",
            Self::Nvidia => "nvh264enc",
            Self::V4l2 => "v4l2h264enc",
            Self::OpenH264 => "openh264enc",
            Self::X264 => "x264enc",
        }
    }
}

/// The capture element: the PipeWire stream, or a moving test picture for the tests.
pub(crate) fn fuente(origen: &Origen, modo: &VideoMode) -> String {
    match origen {
        // Without media.type WirePlumber cannot link KWin's direct stream ("target not found").
        Origen::PipeWire { nodo, .. } => format!(
            "pipewiresrc name=fuente path={nodo} do-timestamp=true always-copy=true keepalive-time={} \
             max-buffers=64 resend-last=true stream-properties=\"props,media.type=Video\"",
            1000 / modo.fps.max(1)
        ),
        Origen::Prueba => format!(
            "videotestsrc name=fuente is-live=true pattern=pinwheel ! \
             video/x-raw,width={},height={},framerate={}/1",
            modo.width, modo.height, modo.fps
        ),
    }
}
