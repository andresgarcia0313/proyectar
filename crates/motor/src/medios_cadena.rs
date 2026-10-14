//! Pipeline text (source, encoder, MPEG-TS, RTP, appsink for `envio`): pure strings, so the layout can be unit tested.

use crate::medios::{Codificador, Origen};
use crate::medios_audio::audio;
use crate::medios_tipos::fuente;
use crate::modo::Perfil;
use wfd::{AudioCodec, VideoMode};

pub const PID_PMT: u32 = 0x1000;
pub const PID_VIDEO: u32 = 0x1011;
pub const PID_AUDIO: u32 = 0x1100;

pub(crate) fn cola(max: u32, fuga: bool) -> String {
    let fuga = if fuga { " leaky=downstream" } else { "" };
    format!("queue max-size-buffers={max} max-size-time=0 max-size-bytes=0{fuga}")
}

fn codificador(cod: Codificador, modo: &VideoMode, perfil: Perfil) -> String {
    let (fps, alto, kbps, cpb) =
        (modo.fps, modo.profile == 2, perfil.kbps, perfil.kbps * perfil.cpb_ms / 1000);
    let (w, h) = (modo.width, modo.height);
    let sw = format!("videoconvert name=pre ! videoscale ! video/x-raw,format=I420,width={w},height={h}");
    match cod {
        // Capture, scaling, color conversion and encoding all stay on the GPU.
        Codificador::Gpu | Codificador::GpuVa => format!(
            "vapostproc name=pre ! video/x-raw(memory:VAMemory),format=NV12,width={w},height={h} ! \
             {} name=cod rate-control=cbr bitrate={kbps} cpb-size={} target-usage={} \
             key-int-max={fps} b-frames=0 ref-frames=1 num-slices=1 cabac={alto} dct8x8={alto} \
             aud=true trellis=false mbbrc=disabled",
            cod.nombre(),
            cpb,
            perfil.rapidez
        ),
        Codificador::Nvidia => format!(
            "{sw} ! nvh264enc name=cod preset=low-latency-hq rc-mode=cbr bitrate={kbps} \
             vbv-buffer-size={cpb} gop-size={fps} bframes=0 zerolatency=true aud=true"
        ),
        Codificador::V4l2 => format!(
            "{sw} ! v4l2h264enc name=cod extra-controls=\"controls,video_bitrate={},\
             h264_i_frame_period={fps},repeat_sequence_header=1\"",
            kbps * 1000
        ),
        Codificador::OpenH264 => format!(
            "{sw} ! openh264enc name=cod usage-type=screen complexity=low rate-control=bitrate \
             bitrate={} gop-size={fps} enable-frame-skip=false multi-thread=2",
            kbps * 1000
        ),
        Codificador::X264 => format!(
            "{sw} ! x264enc name=cod tune=zerolatency speed-preset=ultrafast bitrate={kbps} \
             vbv-buf-capacity={} key-int-max={fps} bframes=0 ref=1 cabac={alto} dct8x8={alto} \
             aud=true byte-stream=true option-string=scenecut=0",
            perfil.cpb_ms
        ),
    }
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
