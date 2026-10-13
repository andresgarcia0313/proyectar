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

/// AAC first: every Miracast sink decodes it and it needs a twentieth of LPCM's bandwidth.
/// `None` when the sink takes no audio or none we can send.
pub fn choose(sink: &str) -> Option<AudioCodec> {
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
    if has("AAC", 0) {
        Some(AudioCodec::Aac)
    } else if has("LPCM", 1) {
        Some(AudioCodec::Lpcm)
    } else {
        None
    }
}
