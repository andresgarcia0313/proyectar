//! Verdicts for the manual tests 1 to 4 from a session summary; 5 to 8 in `resumen_veredictos_uso`,
//! 9 to 11 in `resumen_veredictos_red`.
//! What the computer cannot see (the TV's screen, its sound, its menus) stays `sin_datos`.

use crate::modo::Latencia;
use crate::resumen_tipos::{hasta, Resultado, Resultado::*, Resumen, Veredicto};

/// Seconds streaming before any measure is judged: a session cut after 5 s says nothing.
pub const MIN_S: u32 = 20;
/// Capture to network inside this laptop, 95th percentile, ms. Windows' own sender stays near
/// 30-50 ms; past 60 the pointer trails the hand, past 100 anyone notices.
const RETARDO_P95_MS: (f32, f32) = (60.0, 100.0);
/// Loss the TV reports, session mean, %: under 1 the TV conceals it, past 3 it shows as blocks.
pub const PERDIDA_PCT: (f32, f32) = (1.0, 3.0);
/// A single report this lossy is a visible burst even when the mean is low.
const PERDIDA_PICO_PCT: f32 = 10.0;
/// Full frames the TV asked for, per minute: each one is a picture it lost and rebuilt.
const IDR_POR_MIN: (f32, f32) = (1.0, 4.0);
/// Frames dropped before the encoder, % of those due: 1 % at 30 fps is a hitch every 3 s.
const DESCARTE_PCT: (f32, f32) = (1.0, 5.0);

pub fn veredicto(prueba: u8, nombre: &'static str, resultado: Resultado, detalle: String) -> Veredicto {
    Veredicto { prueba, nombre, resultado, detalle }
}

/// A number the Colombian way: one decimal, decimal comma.
pub fn n(v: f32) -> String {
    format!("{v:.1}").replace('.', ",")
}

pub fn veredictos(r: &Resumen) -> Vec<Veredicto> {
    let (uso, red) =
        (crate::resumen_veredictos_uso::veredictos(r), crate::resumen_veredictos_red::veredictos(r));
    [ajustes(), retardo(r), latencia(r), imagen(r)].into_iter().chain(uso).chain(red).collect()
}

fn ajustes() -> Veredicto {
    let detalle = "los menús del televisor (modo juego, nitidez) no se pueden leer desde el equipo".into();
    veredicto(1, "ajustes_tv", SinDatos, detalle)
}

fn retardo(r: &Resumen) -> Veredicto {
    let Some(p95) = r.retardo_equipo_p95_ms.filter(|_| r.transmision_s >= MIN_S) else {
        return veredicto(2, "retardo_equipo", SinDatos, format!("hacen falta {MIN_S} s transmitiendo"));
    };
    let rtt = r.rtt_media_ms.map_or("sin eco RTCP del TV".into(), |v| format!("ida y vuelta {} ms", n(v)));
    let detalle =
        format!("p95 {} ms de la captura a la red (bien hasta {} ms); {rtt}", n(p95), RETARDO_P95_MS.0);
    veredicto(2, "retardo_equipo", hasta(p95, RETARDO_P95_MS.0, RETARDO_P95_MS.1), detalle)
}

/// Glass to glass with a stopwatch, by mode: Juegos aims at Windows' game mode (~100 ms);
/// Videos keeps a large buffer on purpose, so it is allowed far more.
fn latencia(r: &Resumen) -> Veredicto {
    let Some(ms) = r.latencia_medida_ms else {
        let detalle = "sin cámara no se mide de pantalla a pantalla; anota el cronómetro en la app".into();
        return veredicto(3, "latencia_total", SinDatos, detalle);
    };
    let (bien, aviso) = match r.modo.unwrap_or_default() {
        Latencia::Juegos => (150.0, 250.0),
        Latencia::Trabajo => (250.0, 400.0),
        Latencia::Videos => (500.0, 1000.0),
    };
    veredicto(
        3,
        "latencia_total",
        hasta(ms, bien, aviso),
        format!("{} ms medidos (bien hasta {bien} ms)", n(ms)),
    )
}

fn imagen(r: &Resumen) -> Veredicto {
    if r.transmision_s < MIN_S {
        return veredicto(4, "imagen", SinDatos, format!("hacen falta {MIN_S} s transmitiendo"));
    }
    let minutos = r.transmision_s as f32 / 60.0;
    let idr = r.idr_pedidos as f32 / minutos;
    let debidos = (r.fps_objetivo as f32 * r.transmision_s as f32).max(1.0);
    let descarte = r.descartes as f32 * 100.0 / debidos;
    let mut peor =
        hasta(idr, IDR_POR_MIN.0, IDR_POR_MIN.1).max(hasta(descarte, DESCARTE_PCT.0, DESCARTE_PCT.1));
    let perdida = match (r.perdida_media_pct, r.perdida_max_pct) {
        (Some(media), Some(pico)) => {
            peor = peor.max(hasta(media, PERDIDA_PCT.0, PERDIDA_PCT.1));
            peor = peor.max(if pico > PERDIDA_PICO_PCT { Aviso } else { Bien });
            format!("pérdida media {} % (pico {} %)", n(media), n(pico))
        }
        _ => "sin informes RTCP del TV".into(),
    };
    let detalle = format!(
        "{perdida}, {} cuadros completos pedidos por minuto, {} % de cuadros descartados; la imagen del TV no se ve desde aquí",
        n(idr),
        n(descarte)
    );
    veredicto(4, "imagen", peor, detalle)
}

#[cfg(test)]
#[path = "resumen_veredictos_pruebas.rs"]
mod pruebas;
