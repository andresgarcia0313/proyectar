//! Health while streaming, for the event log: the Wi-Fi Direct radio as the driver sees it
//! (signal, PHY rate, retries) and this process' CPU use. Retries climbing or the rate falling
//! explain stutter and delay before the TV complains.

use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Default, Serialize, PartialEq)]
pub struct Radio {
    pub interfaz: String,
    pub senal_dbm: Option<i32>,
    pub tx_mbps: Option<f32>,
    pub reintentos: Option<u64>,
    pub fallidos: Option<u64>,
}

/// Reads `iw dev p2p-… station dump` (no root needed); `None` when there is no group.
pub async fn radio() -> Option<Radio> {
    tokio::task::spawn_blocking(|| {
        let interfaz = std::fs::read_dir("/sys/class/net")
            .ok()?
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .find(|n| n.starts_with("p2p-"))?;
        let salida =
            std::process::Command::new("iw").args(["dev", &interfaz, "station", "dump"]).output().ok()?;
        Some(Radio { interfaz, ..leer(&String::from_utf8_lossy(&salida.stdout)) })
    })
    .await
    .ok()
    .flatten()
}

fn leer(texto: &str) -> Radio {
    let valor = |clave: &str| {
        texto.lines().find_map(|l| l.trim().strip_prefix(clave)).and_then(|v| v.split_whitespace().next())
    };
    Radio {
        interfaz: String::new(),
        senal_dbm: valor("signal:").and_then(|v| v.parse().ok()),
        tx_mbps: valor("tx bitrate:").and_then(|v| v.parse().ok()),
        reintentos: valor("tx retries:").and_then(|v| v.parse().ok()),
        fallidos: valor("tx failed:").and_then(|v| v.parse().ok()),
    }
}

/// CPU used by this process since the previous call, in % of one core.
pub struct Cpu(Instant, u64);

impl Cpu {
    pub fn nuevo() -> Self {
        Self(Instant::now(), ticks())
    }

    pub fn medir(&mut self) -> f32 {
        let (ahora, t) = (Instant::now(), ticks());
        let uso = (t - self.1.min(t)) as f32 / 100.0 / ahora.duration_since(self.0).as_secs_f32().max(0.001);
        *self = Self(ahora, t);
        uso * 100.0
    }
}

/// utime + stime of this process in clock ticks (100 per second on Linux).
fn ticks() -> u64 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let resto = stat.rsplit_once(')').map(|(_, r)| r).unwrap_or_default();
    let campos: Vec<&str> = resto.split_whitespace().collect();
    campos.get(11).zip(campos.get(12)).map_or(0, |(u, s)| u.parse().unwrap_or(0) + s.parse().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    #[test]
    fn lee_la_salida_de_iw() {
        let texto = "Station 9a:06\n\tsignal:  \t-48 [-50, -51] dBm\n\ttx bitrate:\t433.3 MBit/s VHT-MCS 9\n\
                     \ttx retries:\t12\n\ttx failed:\t0\n";
        let r = super::leer(texto);
        assert_eq!(
            (r.senal_dbm, r.tx_mbps, r.reintentos, r.fallidos),
            (Some(-48), Some(433.3), Some(12), Some(0))
        );
    }

    #[test]
    fn mide_cpu_propia() {
        let mut cpu = super::Cpu::nuevo();
        let t = std::time::Instant::now();
        while t.elapsed().as_millis() < 300 {}
        assert!(cpu.medir() > 10.0);
    }
}
