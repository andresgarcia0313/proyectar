//! How the report reads: Spanish labels, decimal comma and a plain-text verdict table.

use crate::informe_sesion::Sesion;
use serde_json::Value;
use std::fmt::Write;

const METRICAS: [(&str, &str, &str); 18] = [
    ("fps_media", "FPS medio", ""),
    ("fps_p5", "FPS p5", ""),
    ("kbps_media", "tasa media", " kb/s"),
    ("retardo_equipo_media_ms", "retardo del equipo", " ms"),
    ("retardo_equipo_p95_ms", "retardo p95", " ms"),
    ("perdida_media_pct", "pérdida media", " %"),
    ("perdida_max_pct", "pérdida máxima", " %"),
    ("jitter_media_ms", "jitter medio", " ms"),
    ("rtt_media_ms", "RTT medio", " ms"),
    ("senal_min_dbm", "señal mínima", " dBm"),
    ("tx_mbps_media", "tasa del radio", " Mb/s"),
    ("reconexiones", "reconexiones", ""),
    ("recuperacion_media_s", "recuperación media", " s"),
    ("idr_pedidos", "IDR pedidos", ""),
    ("descartes", "cuadros descartados", ""),
    ("cpu_media_pct", "CPU media", " %"),
    ("carga_media", "carga media", ""),
    ("latencia_medida_ms", "latencia medida", " ms"),
];

pub fn numero(v: f64) -> String {
    let texto = if v.fract() == 0.0 { format!("{v:.0}") } else { format!("{v:.1}") };
    texto.replace('.', ",")
}

fn duracion(s: f64) -> String {
    let s = s.max(0.0).round() as u64;
    if s >= 60 {
        format!("{} min {} s", s / 60, s % 60)
    } else {
        format!("{s} s")
    }
}

pub fn texto(v: &Value) -> String {
    v.as_str().map(str::to_owned).unwrap_or_else(|| if v.is_null() { "-".into() } else { v.to_string() })
}

pub fn sesion(numero_sesion: usize, s: &Sesion) -> String {
    let mut t = format!("\n== Sesión {numero_sesion}: {} ==\n", s.pantalla());
    let dur = s.duracion_s().map_or("desconocida".into(), duracion);
    let _ = writeln!(t, "Inicio: {} | Modo: {} | Duración: {dur}", s.inicio(), s.modo());
    let m = s.metricas();
    let datos: Vec<String> = METRICAS
        .iter()
        .filter_map(|(clave, nombre, unidad)| {
            Some(format!("{nombre} {}{unidad}", numero(m[*clave].as_f64()?)))
        })
        .collect();
    let datos = if datos.is_empty() { "sin estadísticas".into() } else { datos.join(" - ") };
    let _ = writeln!(t, "Métricas: {datos}");
    let mut fallos: Vec<(&str, usize)> = Vec::new(); // phase and count, in order of appearance
    for fase in s.eventos.iter().filter(|e| e["evento"] == "fallo").filter_map(|e| e["fase"].as_str()) {
        match fallos.iter_mut().find(|(f, _)| *f == fase) {
            Some((_, n)) => *n += 1,
            None => fallos.push((fase, 1)),
        }
    }
    if !fallos.is_empty() {
        let lista: Vec<String> =
            fallos.iter().map(|(f, n)| if *n > 1 { format!("{f} x{n}") } else { (*f).to_owned() }).collect();
        let total: usize = fallos.iter().map(|(_, n)| n).sum();
        let _ = writeln!(t, "Fallos ({total}): {}", lista.join(", "));
    }
    let veredictos = s.resumen.as_ref().and_then(|r| r["veredictos"].as_array());
    match veredictos {
        Some(lista) if !lista.is_empty() => {
            let _ = writeln!(t, "{:<28} {:<10} Detalle", "Prueba", "Resultado");
            for v in lista {
                let _ = writeln!(
                    t,
                    "{:<28} {:<10} {}",
                    texto(&v["prueba"]),
                    texto(&v["resultado"]),
                    texto(&v["detalle"])
                );
            }
        }
        _ => t.push_str("Veredictos: el registro no trae resumen_sesion; las métricas se reconstruyeron.\n"),
    }
    t
}
