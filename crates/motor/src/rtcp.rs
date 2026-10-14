//! RTCP reports from the TV (RFC 3550, section 6.4): how much of our stream it lost and how
//! uneven the arrivals were. A compound packet chains SR, RR, SDES...; only report blocks count.

/// The SSRC `rtpmp2tpay` stamps on our packets (`ssrc=1` in the pipeline text).
pub const SSRC: u32 = 1;

const SR: u8 = 200;
const RR: u8 = 201;
const BLOQUE: usize = 24;
/// MP2T over RTP runs a 90 kHz clock: jitter comes in those ticks.
const TICS_POR_MS: f32 = 90.0;

/// What the receiver says about our stream.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Informe {
    /// Share lost since the previous report, in percent.
    pub perdida_pct: f32,
    /// Packets lost since the start (negative when duplicates arrived).
    pub perdidos: i32,
    /// Interarrival jitter, in ms.
    pub jitter_ms: f32,
}

/// The report about `ssrc` in a (possibly compound) packet; the first block when none names it.
pub fn leer(datos: &[u8], ssrc: u32) -> Option<Informe> {
    let (mut resto, mut primero) = (datos, None);
    while resto.len() >= 4 {
        let largo = (usize::from(u16::from_be_bytes([resto[2], resto[3]])) + 1) * 4;
        if resto[0] >> 6 != 2 || largo > resto.len() {
            break;
        }
        let (paquete, cuenta) = (&resto[..largo], usize::from(resto[0] & 0x1f));
        let inicio = match resto[1] {
            SR => Some(28),
            RR => Some(8),
            _ => None,
        };
        for b in inicio.into_iter().flat_map(|i| (0..cuenta).map(move |n| i + n * BLOQUE)) {
            let Some(b) = paquete.get(b..b + BLOQUE) else { break };
            let informe = bloque(b);
            if u32::from_be_bytes([b[0], b[1], b[2], b[3]]) == ssrc {
                return Some(informe);
            }
            primero.get_or_insert(informe);
        }
        resto = &resto[largo..];
    }
    primero
}

fn bloque(b: &[u8]) -> Informe {
    Informe {
        perdida_pct: f32::from(b[4]) * 100.0 / 256.0,
        // 24-bit signed: shift into the top of an i32 and back to keep the sign.
        perdidos: i32::from_be_bytes([b[5], b[6], b[7], 0]) >> 8,
        jitter_ms: u32::from_be_bytes([b[12], b[13], b[14], b[15]]) as f32 / TICS_POR_MS,
    }
}

#[cfg(test)]
#[path = "rtcp_pruebas.rs"]
mod pruebas;
