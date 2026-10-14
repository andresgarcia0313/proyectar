//! "Medir latencia": the delay the user read off a photo of the laptop and the TV goes to
//! the event log, next to what the projection was doing at that moment.

use crate::proyeccion_object::qobject;
use cxx_qt_lib::QString;
use serde_json::json;

/// Longest delay that still makes sense for a photo of two clocks (ms).
const TOPE_MS: f64 = 10_000.0;

/// Reads "120", "120 ms" or "85,5" as milliseconds; anything else is `None`.
pub(crate) fn interpretar(texto: &str) -> Option<f64> {
    let limpio = texto.trim().trim_end_matches("ms").trim().replace(',', ".");
    let ms: f64 = limpio.parse().ok()?;
    (ms.is_finite() && (0.0..=TOPE_MS).contains(&ms)).then_some(ms)
}

impl qobject::Proyeccion {
    /// Logs the measured delay; false when the text is not a delay in ms.
    pub fn anotar_latencia(&self, texto: &QString) -> bool {
        let Some(ms) = interpretar(&texto.to_string()) else {
            return false;
        };
        motor::registro::anotar(
            "latencia_medida",
            json!({
                "ms": ms,
                "opciones": self.opciones(),
                "retardo_interno_ms": self.retardo,
                "fps": self.fps,
                "kbps": self.kbps,
            }),
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::interpretar;

    #[test]
    fn acepta_milisegundos_con_o_sin_unidad() {
        assert_eq!(interpretar("120"), Some(120.0));
        assert_eq!(interpretar(" 120 ms "), Some(120.0));
        assert_eq!(interpretar("85,5"), Some(85.5));
        assert_eq!(interpretar("0"), Some(0.0));
    }

    #[test]
    fn rechaza_lo_que_no_es_un_retardo() {
        for malo in ["", "ms", "rápido", "-40", "NaN", "inf", "10001"] {
            assert_eq!(interpretar(malo), None, "{malo:?}");
        }
    }
}
