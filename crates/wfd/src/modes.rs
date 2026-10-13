//! Resolution tables of `wfd_video_formats` (WFD spec v2.1, tables 5-21 to 5-23): which bit
//! of the CEA, VESA and HH bitmaps means which picture. Interlaced modes are left out,
//! the source only produces progressive video.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Table {
    Cea,
    Vesa,
    Hh,
}

impl Table {
    /// Field of a codec entry holding this bitmap, counted after `native` and `preferred`.
    pub fn field(self) -> usize {
        match self {
            Table::Cea => 2,
            Table::Vesa => 3,
            Table::Hh => 4,
        }
    }

    /// Offset of this table in `VideoMode::cea_bit`, which keeps its name for compatibility.
    pub fn offset(self) -> u32 {
        match self {
            Table::Cea => 0,
            Table::Vesa => 32,
            Table::Hh => 64,
        }
    }
}

/// (bit, width, height, fps) for the progressive CEA modes.
const CEA: [(u32, u32, u32, u32); 13] = [
    (0, 640, 480, 60),
    (1, 720, 480, 60),
    (3, 720, 576, 50),
    (5, 1280, 720, 30),
    (6, 1280, 720, 60),
    (7, 1920, 1080, 30),
    (8, 1920, 1080, 60),
    (10, 1280, 720, 25),
    (11, 1280, 720, 50),
    (12, 1920, 1080, 25),
    (13, 1920, 1080, 50),
    (15, 1280, 720, 24),
    (16, 1920, 1080, 24),
];

/// VESA and HH list each size twice: bit 2n at 30 fps, bit 2n+1 at 60 fps.
const VESA: [(u32, u32); 15] = [
    (800, 600),
    (1024, 768),
    (1152, 864),
    (1280, 768),
    (1280, 800),
    (1360, 768),
    (1366, 768),
    (1280, 1024),
    (1400, 1050),
    (1440, 900),
    (1600, 900),
    (1600, 1200),
    (1680, 1024),
    (1680, 1050),
    (1920, 1200),
];
const HH: [(u32, u32); 6] = [(800, 480), (854, 480), (864, 480), (640, 360), (960, 540), (848, 480)];

/// Every mode of `table` as (bit, width, height, fps).
pub fn modes(table: Table) -> Vec<(u32, u32, u32, u32)> {
    let paired = |sizes: &[(u32, u32)]| {
        let each =
            |(i, &(w, h)): (usize, &(u32, u32))| [(2 * i as u32, w, h, 30), (2 * i as u32 + 1, w, h, 60)];
        sizes.iter().enumerate().flat_map(each).collect()
    };
    match table {
        Table::Cea => CEA.to_vec(),
        Table::Vesa => paired(&VESA),
        Table::Hh => paired(&HH),
    }
}

/// Lowest WFD level bit (0x01 = 3.1 ... 0x10 = 4.2) whose H.264 limits carry the mode,
/// from the macroblocks per frame and per second of the standard; `None` beyond 4.2.
pub fn level_needed(width: u32, height: u32, fps: u32) -> Option<u8> {
    let frame = width.div_ceil(16) * height.div_ceil(16);
    let rate = frame * fps;
    const LIMITS: [(u8, u32, u32); 5] = [
        (0x01, 3600, 108_000),
        (0x02, 5120, 216_000),
        (0x04, 8192, 245_760),
        (0x08, 8192, 245_760),
        (0x10, 8704, 522_240),
    ];
    LIMITS.iter().find(|(_, fs, mbps)| frame <= *fs && rate <= *mbps).map(|(bit, ..)| *bit)
}
