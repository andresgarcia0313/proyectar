//! The encoder stage of the pipeline: one description per encoder in the fallback chain.

use crate::medios::Codificador;
use crate::modo::Perfil;
use wfd::VideoMode;

pub(crate) fn codificador(cod: Codificador, modo: &VideoMode, perfil: Perfil) -> String {
    let (fps, alto, kbps, cpb) =
        (modo.fps, modo.profile == 2, perfil.kbps, perfil.kbps * perfil.cpb_ms / 1000);
    let (w, h) = (modo.width, modo.height);
    let sw = format!("videoconvert name=pre ! videoscale ! video/x-raw,format=I420,width={w},height={h}");
    match cod {
        // Scaled on the GPU, encoded by x264 with a refresh column sweeping the picture instead of key
        // frames: every frame weighs about the same, so the TV's buffer holds only ~3 frames.
        Codificador::Refresco => format!(
            "vapostproc name=pre ! video/x-raw,format=NV12,width={w},height={h} ! x264enc name=cod \
             tune=zerolatency speed-preset=ultrafast bitrate={kbps} vbv-buf-capacity={} key-int-max={fps} \
             intra-refresh=true bframes=0 ref=1 cabac={alto} dct8x8={alto} sliced-threads=true threads=4 \
             aud=true byte-stream=true option-string=scenecut=0",
            3000 / fps.max(1)
        ),
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
