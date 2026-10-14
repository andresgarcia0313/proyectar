//! What a session's numbers were when the log has no `resumen_sesion` (0.4.0 and older): rebuilt
//! from `estadisticas` (flat or under `video`), the reconnections and `latencia_medida`, with
//! the same keys as the summary so the report prints both alike.

use serde_json::{json, Map, Value};

pub fn reconstruir(eventos: &[Value]) -> Value {
    let stats: Vec<&Value> = eventos.iter().filter(|e| e["evento"] == "estadisticas").collect();
    let serie = |f: &dyn Fn(&Value) -> Option<f64>| stats.iter().filter_map(|e| f(e)).collect::<Vec<f64>>();
    let video = |k: &'static str| move |e: &Value| e["video"][k].as_f64().or_else(|| e[k].as_f64());
    let media = |v: Vec<f64>| (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64);
    let extremo = |v: Vec<f64>, max: bool| v.into_iter().reduce(if max { f64::max } else { f64::min });
    let reconectando = |e: &&Value| e["estado"].as_str().is_some_and(|s| s.starts_with("Reconectando"));
    let latencia = eventos.iter().rev().filter(|e| e["evento"] == "latencia_medida");
    let mut m = Map::new();
    for (clave, valor) in [
        ("fps_media", media(serie(&video("fps")))),
        ("kbps_media", media(serie(&video("kbps")))),
        ("retardo_equipo_media_ms", media(serie(&video("retardo_ms")))),
        ("perdida_max_pct", extremo(serie(&video("perdida_pct")), true)),
        ("jitter_media_ms", media(serie(&video("jitter_ms")))),
        ("senal_min_dbm", extremo(serie(&|e| e["radio"]["senal_dbm"].as_f64()), false)),
        ("cpu_media_pct", media(serie(&|e| e["cpu_pct"].as_f64()))),
        ("reconexiones", Some(eventos.iter().filter(reconectando).count() as f64)),
        ("latencia_medida_ms", latencia.filter_map(|e| e["ms"].as_f64()).next()),
    ] {
        if let Some(v) = valor {
            m.insert(clave.into(), json!(v));
        }
    }
    Value::Object(m)
}

/// `2026-10-13 16:28:03.477` as seconds on a continuous scale (days from the civil calendar).
pub fn segundos(hora: &str) -> Option<f64> {
    let (fecha, reloj) = hora.split_once(' ')?;
    let f: Vec<i64> = fecha.split('-').map(str::parse).collect::<Result<_, _>>().ok()?;
    let r: Vec<f64> = reloj.split(':').map(str::parse).collect::<Result<_, _>>().ok()?;
    let (&[a, m, d], &[h, min, s]) = (f.as_slice(), r.as_slice()) else { return None };
    let (a, m) = if m <= 2 { (a - 1, m + 9) } else { (a, m - 3) };
    let dias = 365 * a + a / 4 - a / 100 + a / 400 + (153 * m + 2) / 5 + d;
    Some(dias as f64 * 86_400.0 + h * 3600.0 + min * 60.0 + s)
}

#[cfg(test)]
mod tests {
    use super::{reconstruir, segundos};
    use serde_json::json;

    #[test]
    fn las_horas_cruzan_la_medianoche_y_los_meses() {
        let d = segundos("2026-03-01 00:00:10.500").unwrap() - segundos("2026-02-28 23:59:59.000").unwrap();
        assert!((d - 11.5).abs() < 1e-6, "{d}");
        assert_eq!(segundos("hora rota"), None);
    }

    #[test]
    fn toma_la_ultima_latencia_medida_y_omite_lo_que_no_hay() {
        let eventos = [
            json!({"evento": "latencia_medida", "ms": 180}),
            json!({"evento": "latencia_medida", "ms": 120}),
        ];
        assert_eq!(reconstruir(&eventos), json!({"reconexiones": 0.0, "latencia_medida_ms": 120.0}));
    }
}
