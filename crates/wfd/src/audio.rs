//! `wfd_audio_codecs`: `LPCM 00000003 00, AAC 00000001 00` (codec, modes bitmap, latency).
//! The source sends 48 kHz stereo, which is mode bit 0 for AAC and bit 1 for LPCM.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioCodec {
    Aac,
    Lpcm,
}

impl AudioCodec {
    pub fn to_param(self) -> &'static str {
        match self {
            AudioCodec::Aac => "AAC 00000001 00",
            AudioCodec::Lpcm => "LPCM 00000002 00",
        }
    }
}

/// AAC first (every Miracast sink decodes it, a twentieth of LPCM's bandwidth) unless `lpcm`
/// asks for uncompressed audio and the sink lists LPCM 48 kHz stereo.
/// A sink that takes audio but lists neither (only AC3, only 44.1 kHz, unreadable) still
/// gets LPCM 48 kHz stereo, which the spec makes mandatory. `None` only for `none`.
pub fn choose(sink: &str, lpcm: bool) -> Option<AudioCodec> {
    let sink = sink.trim();
    if sink.is_empty() || sink.eq_ignore_ascii_case("none") {
        return None;
    }
    let offers: Vec<(&str, u32)> = sink
        .split(',')
        .filter_map(|entry| {
            let mut f = entry.split_whitespace();
            let name = f.next()?;
            let modes = u32::from_str_radix(f.next()?, 16).ok()?;
            Some((name, modes))
        })
        .collect();
    let has = |name: &str, bit: u32| {
        offers.iter().any(|(n, m)| n.eq_ignore_ascii_case(name) && m & (1 << bit) != 0)
    };
    Some(if has("AAC", 0) && !(lpcm && has("LPCM", 1)) { AudioCodec::Aac } else { AudioCodec::Lpcm })
}
