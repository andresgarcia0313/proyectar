//! Wi-Fi Display (Miracast, spec v2.1 R1 features) source side, with no I/O policy of its own: RTSP messages,
//! capability negotiation (M1-M7), keep-alive and the information elements for discovery.

pub mod audio;
mod caps;
mod events;
pub mod ie;
mod inbox;
mod link;
pub mod modes;
pub mod params;
mod read;
pub mod rtsp;
pub mod session;
pub mod sink_info;
mod start;
mod triggers;
pub mod video;

pub use audio::AudioCodec;
pub use modes::Table;
pub use session::{Event, Negotiated, Session, Wanted};
pub use sink_info::SinkInfo;
pub use video::VideoMode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("conexión con la pantalla: {0}")]
    Io(#[from] std::io::Error),
    #[error("mensaje RTSP mal formado: {0}")]
    Malformed(String),
    #[error("la pantalla respondió {code} a {step}")]
    Refused { step: &'static str, code: u16 },
    #[error("la pantalla cerró la conexión durante {0}")]
    Closed(&'static str),
    #[error("la pantalla no respondió a tiempo en {0}")]
    Timeout(&'static str),
    #[error("no hay formato de {0} en común con la pantalla")]
    NoCommonFormat(&'static str),
}

impl Error {
    /// Worth a new attempt (link hiccup, slow sink) rather than a dead end (incompatible sink).
    pub fn is_transient(&self) -> bool {
        !matches!(self, Error::NoCommonFormat(_) | Error::Malformed(_))
    }
}
