//! The byte surgery behind `medios_lpcm`: relabels mpegtsmux's DVD LPCM as Miracast LPCM in the
//! PMT and in each audio PES, in place, on the 188-byte packets leaving the muxer.

use crate::medios_cadena::{PID_AUDIO, PID_PMT};
use crate::medios_lpcm::crc32;

/// LPCM audio descriptor: 48 kHz, no emphasis, stereo.
pub const DESCRIPTOR: [u8; 4] = [0x83, 2, (2 << 5) | (3 << 1), (1 << 5) | 0xF];
const TS: usize = 188;

/// Rewrites, in place, the PMT and the start of each audio PES in these TS packets.
pub fn corregir(datos: &mut [u8]) {
    for p in datos.chunks_exact_mut(TS) {
        if p[0] != 0x47 || p[1] & 0x40 == 0 {
            continue;
        }
        let pid = (u32::from(p[1] & 0x1F) << 8) | u32::from(p[2]);
        let inicio = 4 + if p[3] & 0x20 != 0 { 1 + usize::from(p[4]) } else { 0 };
        match pid {
            _ if inicio >= TS => {}
            PID_PMT => pmt(p, inicio),
            PID_AUDIO => pes(&mut p[inicio..]),
            _ => {}
        }
    }
}

/// Stream 0xFD becomes 0xBD; its extension field (only valid on 0xFD) becomes stuffing.
fn pes(c: &mut [u8]) {
    if c.len() < 9 || c[..4] != [0, 0, 1, 0xFD] {
        return;
    }
    c[3] = 0xBD;
    let (fin, campos) = (
        9 + usize::from(c[8]),
        match c[7] >> 6 {
            2 => 5,
            3 => 10,
            _ => 0,
        },
    );
    if c[7] & 1 != 0 && c[7] & 0x3E == 0 && fin <= c.len() && 9 + campos <= fin {
        c[7] &= !1;
        c[9 + campos..fin].fill(0xFF);
    }
}

/// Type 0x8B becomes 0x83 and gains the LPCM descriptor; the section grows by 4 and is re-signed.
/// mpegtsmux pads the PMT packet in front (adaptation field), so the room comes from there.
fn pmt(p: &mut [u8], inicio: usize) {
    let Some((s, largo, i, info)) = entrada_lpcm(&p[inicio..]) else { return };
    let mut base = inicio;
    if base + s + 3 + largo + 4 > TS {
        if p[3] & 0x20 == 0 || p[4] < 5 || p[5] != 0 {
            return; // no stuffing to give up
        }
        p.copy_within(base..TS, base - 4);
        p[4] -= 4;
        base -= 4;
        p[TS - 4..].fill(0xFF);
    }
    let c = &mut p[base..];
    let (donde, fin) = (i + 5 + info, s + 3 + largo - 4); // descriptor goes here; CRC starts there
    c.copy_within(donde..fin, donde + 4);
    c[donde..donde + 4].copy_from_slice(&DESCRIPTOR);
    c[i] = 0x83;
    let poner = |c: &mut [u8], at: usize, v: usize| {
        c[at] = (c[at] & 0xF0) | (v >> 8) as u8;
        c[at + 1] = v as u8;
    };
    poner(c, i + 3, info + 4);
    poner(c, s + 1, largo + 4);
    let crc = crc32(&c[s..fin + 4]);
    c[fin + 4..fin + 8].copy_from_slice(&crc.to_be_bytes());
}

/// Where the section starts, its length, and the audio entry still labelled DVD LPCM (0x8B).
fn entrada_lpcm(c: &[u8]) -> Option<(usize, usize, usize, usize)> {
    let s = 1 + usize::from(*c.first()?);
    if s + 12 > c.len() || c[s] != 0x02 {
        return None;
    }
    let largo = (usize::from(c[s + 1] & 0xF) << 8) | usize::from(c[s + 2]);
    let fin = (s + 3 + largo).checked_sub(4).filter(|f| *f <= c.len())?;
    let mut i = s + 12 + ((usize::from(c[s + 10] & 0xF) << 8) | usize::from(c[s + 11]));
    while i + 5 <= fin {
        let pid = (u32::from(c[i + 1] & 0x1F) << 8) | u32::from(c[i + 2]);
        let info = (usize::from(c[i + 3] & 0xF) << 8) | usize::from(c[i + 4]);
        if pid == PID_AUDIO && c[i] == 0x8B {
            return Some((s, largo, i, info));
        }
        i += 5 + info;
    }
    None
}
