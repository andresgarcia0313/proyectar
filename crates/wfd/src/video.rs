//! `wfd_video_formats`: what the sink can decode and the single mode the source picks.
//! Format: `native preferred profile level CEA VESA HH latency min-slice slice-enc
//! frame-rate-ctl max-hres max-vres`, with one codec entry or several separated by commas.

use crate::modes::{level_needed, modes, Table};
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoMode {
    /// Bit of the mode in its table: CEA 0-31, VESA 32-63 (bit + 32), HH 64-95 (bit + 64).
    /// The name stays from when only CEA was supported.
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
    /// 640x480p60 Constrained Baseline 3.1: every sink must decode it (spec 3.1.1).
    pub const FALLBACK: VideoMode =
        VideoMode { cea_bit: 0, width: 640, height: 480, fps: 60, profile: 0x01, level: 0x01 };

    pub fn table(&self) -> Table {
        match self.cea_bit {
            0..32 => Table::Cea,
            32..64 => Table::Vesa,
            _ => Table::Hh,
        }
    }

    /// The `wfd_video_formats` value for M4: exactly one mode, nothing else enabled.
    pub fn to_param(&self) -> String {
        let bit = 1u32 << (self.cea_bit - self.table().offset());
        let in_table = |t: Table| if self.table() == t { bit } else { 0 };
        format!(
            "00 00 {:02x} {:02x} {:08x} {:08x} {:08x} 00 0000 0000 00 none none",
            self.profile,
            self.level,
            in_table(Table::Cea),
            in_table(Table::Vesa),
            in_table(Table::Hh)
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

/// Picks the best mode both sides support, never above `max_height` or `max_fps` nor
/// above the H.264 level the sink declares for that entry. Smoothness first (height x fps),
/// then width. Constrained Baseline unless the sink lists only Constrained High: the
/// mandatory profile, the cheapest to decode, so the TV adds the least delay.
/// `none` means the sink takes no video; an unreadable list falls back to 640x480p60.
pub fn choose(sink: &str, max_height: u32, max_fps: u32) -> Result<VideoMode, Error> {
    if sink.trim().eq_ignore_ascii_case("none") {
        return Err(Error::NoCommonFormat("video"));
    }
    let mut best: Option<VideoMode> = None;
    for entry in codec_entries(sink) {
        let hex = |i: usize| entry.get(i).and_then(|f| u32::from_str_radix(f, 16).ok()).unwrap_or(0);
        let profile = if hex(0) & 0x03 == 0x02 { 0x02 } else { 0x01 };
        let level = top_bit(hex(1) as u8 & 0x1f);
        for table in [Table::Cea, Table::Vesa, Table::Hh] {
            let bits = hex(table.field());
            for (bit, width, height, fps) in modes(table) {
                let fits = bits & (1 << bit) != 0 && height <= max_height && fps <= max_fps;
                if !fits || level_needed(width, height, fps).is_none_or(|need| need > level) {
                    continue;
                }
                let mode = VideoMode { cea_bit: bit + table.offset(), width, height, fps, profile, level };
                if best.is_none_or(|b| (height * fps, width) > (b.height * b.fps, b.width)) {
                    best = Some(mode);
                }
            }
        }
    }
    Ok(best.unwrap_or(VideoMode::FALLBACK))
}

/// Codec entries as (profile, level, CEA, VESA, HH, ...): the first one carries `native`
/// and `preferred` in front, unless a lax sink left them out (11 fields instead of 13).
fn codec_entries(value: &str) -> Vec<Vec<&str>> {
    let mut parts = value.split(',');
    let first: Vec<&str> = parts.next().unwrap_or("").split_whitespace().collect();
    let skip = if first.len() == 11 { 0 } else { 2 };
    let mut entries = vec![first.get(skip..).unwrap_or(&[]).to_vec()];
    entries.extend(parts.map(|p| p.split_whitespace().collect::<Vec<_>>()));
    entries.into_iter().filter(|e| e.len() >= 3).collect()
}

/// Highest level bit set, so `0x1f` (3.1 to 4.2) reads as 4.2; nothing set reads as 3.1.
fn top_bit(level: u8) -> u8 {
    if level == 0 {
        0x01
    } else {
        0x80 >> level.leading_zeros()
    }
}
