//! Audio branch: the TV's sound (or silence) as AAC-LC or LPCM into the muxer. The sound is
//! the monitor of the TV's own output (`salida_audio`), read in small quanta and stamped with
//! the pipeline clock like the video, so voice and picture stay together.

use crate::medios_cadena::{cola, PID_AUDIO};
use crate::salida_audio_cmd::SUMIDERO;
use wfd::AudioCodec;

/// 256 frames at 48 kHz (5 ms); PipeWire raises it to its minimum quantum if that is larger.
const LATENCIA: &str = "256/48000";

pub fn audio(codec: Option<AudioCodec>, sonido_tv: bool) -> String {
    let origen = if sonido_tv {
        format!(
            "pipewiresrc name=sonido target-object={SUMIDERO} do-timestamp=true provide-clock=false \
             stream-properties=\"props,stream.capture.sink=true,node.latency={LATENCIA}\""
        )
    } else {
        "audiotestsrc name=sonido wave=silence is-live=true".to_owned()
    };
    let cierre = match codec {
        Some(AudioCodec::Lpcm) => {
            "capsfilter caps=audio/x-raw,format=S16BE ! capssetter join=false replace=true caps=\"audio/x-lpcm,width=16,\
             rate=48000,channels=2,dynamic_range=0,emphasis=false,mute=false\""
        }
        // With no negotiated codec silence still goes as AAC: video-only drops Samsung sessions.
        _ => "avenc_aac bitrate=128000 ! aacparse ! audio/mpeg,mpegversion=4,stream-format=adts",
    };
    format!(
        "{origen} ! audioconvert ! audioresample ! audio/x-raw,rate=48000,channels=2 ! \
         {cierre} ! {} ! mux.sink_{PID_AUDIO}",
        cola(8, false)
    )
}

#[cfg(test)]
mod tests {
    use super::audio;
    use wfd::AudioCodec;

    #[test]
    fn con_sonido_lee_el_monitor_del_sumidero_del_tv_con_latencia_baja() {
        let rama = audio(None, true);
        assert!(rama.contains("target-object=proyectar_tv do-timestamp=true provide-clock=false"));
        assert!(rama.contains("stream.capture.sink=true,node.latency=256/48000"));
    }

    #[test]
    fn sin_sonido_manda_silencio() {
        assert!(audio(Some(AudioCodec::Lpcm), false).starts_with("audiotestsrc name=sonido wave=silence"));
    }
}
