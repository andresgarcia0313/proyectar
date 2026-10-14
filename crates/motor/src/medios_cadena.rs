//! Pipeline text (source, encoder, MPEG-TS, RTP, appsink for `envio`): pure strings, so the layout can be unit tested.

use crate::medios::{Codificador, Origen};
use crate::medios_audio::audio;
use crate::medios_codificador::codificador;
use crate::medios_tipos::fuente;
use crate::modo::Perfil;
use wfd::{AudioCodec, VideoMode};

pub const PID_PMT: u32 = 0x1000;
pub const PID_VIDEO: u32 = 0x1011;
pub const PID_AUDIO: u32 = 0x1100;

/// A queue of `max` frames; the leaky one (only before the encoder) is named `fuga` so its
/// drops can be counted (`medios_descartes`).
pub(crate) fn cola(max: u32, fuga: bool) -> String {
    let fuga = if fuga { " leaky=downstream name=fuga" } else { "" };
    format!("queue max-size-buffers={max} max-size-time=0 max-size-bytes=0{fuga}")
}

pub fn descripcion(
    origen: &Origen,
    modo: &VideoMode,
    codec: Option<AudioCodec>,
    perfil: Perfil,
    cod: Codificador,
    sonido_tv: bool,
) -> String {
    let h264 = if modo.profile == 2 { "high" } else { "constrained-baseline" };
    format!(
        "{} ! {} ! videorate name=tasa skip-to-first=true ! video/x-raw,framerate={}/1 ! {} ! \
         h264parse name=analizador config-interval=-1 ! \
         video/x-h264,stream-format=byte-stream,alignment=au,profile={h264} ! \
         {} ! mux.sink_{PID_VIDEO} \
         mpegtsmux name=mux alignment=7 latency={} pat-interval=9000 pmt-interval=9000 \
         pcr-interval=3600 \
         ! rtpmp2tpay pt=33 mtu=1328 perfect-rtptime=false ssrc=1 ! \
         appsink name=salida emit-signals=true sync=false async=false max-buffers=0 \
         enable-last-sample=false {}",
        fuente(origen, modo),
        cola(perfil.cola, true),
        modo.fps,
        codificador(cod, modo, perfil),
        cola(4, false),
        u64::from(perfil.mux_ms) * 1_000_000,
        audio(codec, sonido_tv),
    )
}
