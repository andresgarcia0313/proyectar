//! Audio branch: the TV's sound (or silence) as AAC-LC or LPCM into the muxer.

use crate::medios_cadena::{cola, PID_AUDIO};
use wfd::AudioCodec;

pub fn audio(codec: Option<AudioCodec>, sonido_tv: bool) -> String {
    let origen = if sonido_tv {
        "pipewiresrc name=sonido do-timestamp=true \
         stream-properties=\"props,stream.capture.sink=true\""
    } else {
        "audiotestsrc name=sonido wave=silence is-live=true"
    };
    let cierre = match codec {
        Some(AudioCodec::Lpcm) => {
            "audio/x-raw,format=S16BE ! capssetter replace=true caps=audio/x-lpcm,width=16,\
             rate=48000,channels=2,dynamic_range=0,emphasis=false,mute=false"
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
