//! `wfd_video_formats`: what the sink can decode and the single mode the source picks.
//! Format: `native preferred profile level CEA VESA HH latency min-slice slice-enc
//! frame-rate-ctl max-hres max-vres`, with one codec entry or several separated by commas.

use crate::Error;

/// CEA modes the source can produce, best first: (bit, width, height, fps).
const CEA: [(u32, u32, u32, u32); 6] = [
    (8, 1920, 1080, 60),
    (7, 1920, 1080, 30),
    (6, 1280, 720, 60),
    (5, 1280, 720, 30),
    (1, 720, 480, 60),
    (0, 640, 480, 60),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoMode {
    pub cea_bit: u32,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    /// 0x01 Constrained Baseline, 0x02 Constrained High.
    pub profile: u8,
    /// Highest level bit the sink accepts: 0x01 3.1 ... 0x10 4.2.
    pub level: u8,
}

impl VideoMode {
    /// The `wfd_video_formats` value for M4: exactly one mode, nothing else enabled.
    pub fn to_param(&self) -> String {
        format!(
            "00 00 {:02x} {:02x} {:08x} 00000000 00000000 00 0000 0000 00 none none",
            self.profile,
            self.level,
            1u32 << self.cea_bit
        )
    }

    /// H.264 level string GStreamer caps expect for `level`.
    pub fn h264_level(&self) -> &'static str {
        match self.level {
            0x10.. => "4.2",
            0x08.. => "4.1",
            0x04.. => "4",
            0x02.. => "3.2",
            _ => "3.1",
        }
    }
}

/// Picks the best mode both sides support, never above `max_height` or `max_fps`.
/// Constrained Baseline when the sink lists it: the mandatory profile, the one every
/// decoder handles and the cheapest to decode, so the TV adds the least delay.
pub fn choose(sink: &str, max_height: u32, max_fps: u32) -> Result<VideoMode, Error> {
    let mut best: Option<VideoMode> = None;
    for entry in codec_entries(sink)? {
        let field = |i: usize| entry.get(i).copied().unwrap_or("0");
        let hex = |i: usize| u32::from_str_radix(field(i), 16).unwrap_or(0);
        let (profile, level, cea) = (hex(0) as u8, hex(1) as u8, hex(2));
        let Some(&(bit, w, h, fps)) =
            CEA.iter().find(|(bit, _, h, fps)| cea & (1 << bit) != 0 && *h <= max_height && *fps <= max_fps)
        else {
            continue;
        };
        let profile = if profile & 0x01 != 0 { 0x01 } else { 0x02 };
        let level = top_bit(level);
        let mode = VideoMode { cea_bit: bit, width: w, height: h, fps, profile, level };
        let better = best.is_none_or(|b| h * fps > b.height * b.fps);
        if better {
            best = Some(mode);
        }
    }
    best.ok_or(Error::NoCommonFormat("video"))
}

/// Splits the value into codec entries of 11 fields, skipping `native` and `preferred`.
fn codec_entries(value: &str) -> Result<Vec<Vec<&str>>, Error> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return Err(Error::NoCommonFormat("video"));
    }
    let mut parts = value.split(',');
    let first: Vec<&str> = parts.next().unwrap_or("").split_whitespace().collect();
    let mut entries = vec![first.get(2..).unwrap_or(&[]).to_vec()];
    entries.extend(parts.map(|p| p.split_whitespace().collect::<Vec<_>>()));
    Ok(entries.into_iter().filter(|e| e.len() >= 3).collect())
}

fn top_bit(level: u8) -> u8 {
    if level == 0 {
        0x01
    } else {
        0x80 >> level.leading_zeros()
    }
}
