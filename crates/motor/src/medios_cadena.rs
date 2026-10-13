//! Text description of the GStreamer pipeline: source, encoder, MPEG-TS muxer, RTP, UDP.
//! Pure string building so the layout the Samsung TV is picky about can be unit tested.

use crate::medios::{Codificador, Origen};
use crate::medios_audio::audio;
use crate::modo::Perfil;
use std::net::SocketAddr;
use wfd::{AudioCodec, VideoMode};

pub const PID_PMT: u32 = 0x1000;
pub const PID_VIDEO: u32 = 0x1011;
pub const PID_AUDIO: u32 = 0x1100;

pub(crate) fn cola(max: u32, fuga: bool) -> String {
    let fuga = if fuga { " leaky=downstream" } else { "" };
    format!("queue max-size-buffers={max} max-size-time=0 max-size-bytes=0{fuga}")
}

fn fuente(origen: &Origen, modo: &VideoMode) -> String {
    match origen {
        // Without media.type WirePlumber cannot link KWin's direct stream ("target not found").
        Origen::PipeWire { nodo, .. } => format!(
            "pipewiresrc name=fuente path={nodo} do-timestamp=true always-copy=true keepalive-time=33 \
             max-buffers=64 resend-last=true stream-properties=\"props,media.type=Video\""
        ),
        Origen::Prueba => format!(
            "videotestsrc name=fuente is-live=true pattern=pinwheel ! \
             video/x-raw,width={},height={},framerate={}/1",
            modo.width, modo.height, modo.fps
        ),
    }
}

fn codificador(cod: Codificador, modo: &VideoMode, perfil: Perfil) -> String {
    let (fps, alto, kbps) = (modo.fps, modo.profile == 2, perfil.kbps);
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
            kbps * 2,
            perfil.rapidez
        ),
        Codificador::Nvidia => format!(
            "{sw} ! nvh264enc name=cod preset=low-latency-hq rc-mode=cbr bitrate={kbps} \
             vbv-buffer-size={} gop-size={fps} bframes=0 zerolatency=true aud=true",
            kbps * 2
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
             vbv-buf-capacity=2000 key-int-max={fps} bframes=0 ref=1 cabac={alto} dct8x8={alto} \
             aud=true byte-stream=true option-string=scenecut=0"
        ),
    }
}

pub fn descripcion(
    origen: &Origen,
    destino: SocketAddr,
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
         mpegtsmux name=mux alignment=7 latency=40000000 pat-interval=9000 pmt-interval=9000 \
         pcr-interval=3600 \
         ! rtpmp2tpay pt=33 mtu=1328 perfect-rtptime=false ssrc=1 ! \
         udpsink name=salida host={} port={} sync=false async=false qos-dscp=46 buffer-size=1048576 {}",
        fuente(origen, modo),
        cola(perfil.cola, true),
        modo.fps,
        codificador(cod, modo, perfil),
        cola(4, false),
        destino.ip(),
        destino.port(),
        audio(codec, sonido_tv),
    )
}
