//! How busy the whole computer is while streaming: the 1-minute load and the CPU pressure
//! (PSI, share of the last 10 s some task waited for a CPU). Pressure explains stutter better
//! than the load: a load of 8 on 12 threads may still leave the encoder room, pressure does not.

use serde::Serialize;

#[derive(Debug, Default, Clone, Copy, Serialize, PartialEq)]
pub struct Carga {
    pub carga_1m: Option<f32>,
    pub psi_cpu_pct: Option<f32>,
}

/// Reads `/proc/loadavg` and `/proc/pressure/cpu`; a missing file leaves its value empty.
pub fn medir() -> Carga {
    let leer = |ruta: &str| std::fs::read_to_string(ruta).unwrap_or_default();
    Carga { carga_1m: carga(&leer("/proc/loadavg")), psi_cpu_pct: presion(&leer("/proc/pressure/cpu")) }
}

/// First field of `/proc/loadavg`.
pub fn carga(texto: &str) -> Option<f32> {
    texto.split_whitespace().next()?.parse().ok()
}

/// `avg10` of the `some` line of a PSI file, in percent.
pub fn presion(texto: &str) -> Option<f32> {
    let linea = texto.lines().find(|l| l.starts_with("some "))?;
    linea.split_whitespace().find_map(|c| c.strip_prefix("avg10="))?.parse().ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn lee_carga_y_presion() {
        assert_eq!(super::carga("14.32 10.48 10.16 8/4480 79157\n"), Some(14.32));
        let psi = "some avg10=55.86 avg60=44.64 avg300=36.50 total=1\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
        assert_eq!(super::presion(psi), Some(55.86));
        assert_eq!((super::carga(""), super::presion("")), (None, None));
    }

    #[test]
    fn mide_este_equipo() {
        assert!(super::medir().carga_1m.is_some(), "Linux siempre tiene /proc/loadavg");
    }
}
