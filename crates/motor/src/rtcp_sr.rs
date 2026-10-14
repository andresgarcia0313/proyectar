//! Our own RTCP sender reports (RFC 3550, 6.4.1): once the TV gets one, its receiver reports
//! echo it (LSR) with the time it held it (DLSR), and the round trip of the link comes out.

use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SR: u8 = 200;
/// MP2T over RTP runs a 90 kHz clock.
const HZ: f64 = 90_000.0;
/// Seconds from the NTP era (1900) to the Unix one (1970).
const DESDE_1900: u64 = 2_208_988_800;
/// A round trip beyond this is a stale or foreign echo, not the link.
const RTT_MAX: Duration = Duration::from_secs(5);

/// What the sending thread has put on the wire, for the next sender report.
#[derive(Default)]
pub struct Cuenta {
    paquetes: AtomicU32,
    octetos: AtomicU32,
    /// When the last packet left and the RTP timestamp `rtpmp2tpay` gave it.
    ultimo: Mutex<Option<(Instant, u32)>>,
}

impl Cuenta {
    /// Called for every RTP packet sent; its payload is everything after the 12-byte header.
    pub fn anotar(&self, rtp: &[u8], ahora: Instant) {
        let Some(ts) = rtp.get(4..8) else { return };
        self.paquetes.fetch_add(1, Relaxed);
        self.octetos.fetch_add(rtp.len().saturating_sub(12) as u32, Relaxed);
        let ts = u32::from_be_bytes([ts[0], ts[1], ts[2], ts[3]]);
        *self.ultimo.lock().unwrap_or_else(|e| e.into_inner()) = Some((ahora, ts));
    }

    /// The report for `ssrc` now; `None` until a packet left. The RTP time is the last packet's,
    /// moved forward at 90 kHz, so it matches the timeline the TV is decoding.
    pub fn informe(&self, ssrc: u32, ahora: Instant, ntp: u64) -> Option<[u8; 28]> {
        let (cuando, ts) = (*self.ultimo.lock().unwrap_or_else(|e| e.into_inner()))?;
        let avance = (ahora.saturating_duration_since(cuando).as_secs_f64() * HZ) as u32;
        Some(sr(ssrc, ntp, ts.wrapping_add(avance), self.paquetes.load(Relaxed), self.octetos.load(Relaxed)))
    }
}

/// A sender report with no report blocks (we receive nothing to report on).
pub fn sr(ssrc: u32, ntp: u64, rtp: u32, paquetes: u32, octetos: u32) -> [u8; 28] {
    let mut p = [0u8; 28];
    p[..4].copy_from_slice(&[0x80, SR, 0, 6]); // version 2, no blocks, 7 words minus one
    p[4..8].copy_from_slice(&ssrc.to_be_bytes());
    p[8..16].copy_from_slice(&ntp.to_be_bytes());
    p[16..20].copy_from_slice(&rtp.to_be_bytes());
    p[20..24].copy_from_slice(&paquetes.to_be_bytes());
    p[24..28].copy_from_slice(&octetos.to_be_bytes());
    p
}

/// Wall-clock time as a 64-bit NTP timestamp (seconds since 1900, 32.32 fixed point).
pub fn ntp(t: SystemTime) -> u64 {
    let d = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let fraccion = (u64::from(d.subsec_nanos()) << 32) / 1_000_000_000;
    ((d.as_secs() + DESDE_1900) << 32) | fraccion
}

/// The middle 32 bits of an NTP timestamp, the unit LSR and arrival times are compared in.
pub fn medio(ntp: u64) -> u32 {
    (ntp >> 16) as u32
}

/// Round trip in ms from a receiver report that arrived at `llegada` (middle NTP bits):
/// arrival - LSR - DLSR, all in 1/65536 s. `None` before the TV echoed any report.
pub fn rtt_ms(lsr: u32, dlsr: u32, llegada: u32) -> Option<f32> {
    if lsr == 0 {
        return None;
    }
    let ida_y_vuelta = llegada.wrapping_sub(lsr).wrapping_sub(dlsr);
    let ms = f64::from(ida_y_vuelta) * 1000.0 / 65_536.0;
    (ms <= RTT_MAX.as_millis() as f64).then_some(ms as f32)
}

#[cfg(test)]
#[path = "rtcp_sr_pruebas.rs"]
mod pruebas;
