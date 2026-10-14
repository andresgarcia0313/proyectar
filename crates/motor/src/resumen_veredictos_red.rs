//! Verdicts for the manual tests 9 to 11: ten stable minutes, the link at distance, reconnection.

use crate::resumen_tipos::{desde, hasta, Resultado::*, Resumen, Veredicto};
use crate::resumen_veredictos::{n, veredicto, PERDIDA_PCT};
use crate::resumen_veredictos_uso::fps;

/// The stability test is ten minutes streaming.
const ESTABLE_S: u32 = 600;
/// Weakest signal of the Wi-Fi Direct link, dBm: at -70 the radio keeps its top rates; past -80
/// retries pile up and the picture breaks.
const SENAL_DBM: (f32, f32) = (-70.0, -80.0);
/// Round trip of the link, ms: Wi-Fi Direct with nothing queued answers in 2-10 ms; past 50
/// packets wait in the radio (power save, interference, another network on the channel).
const RTT_MS: (f32, f32) = (20.0, 50.0);
/// From a fall to the picture back, s: the TV's own reconnection takes about 10 s; past 45 the
/// person has already reached for the remote.
const RECUPERACION_S: (f32, f32) = (20.0, 45.0);

pub fn veredictos(r: &Resumen) -> [Veredicto; 3] {
    [estabilidad(r), enlace(r), reconexion(r)]
}

fn estabilidad(r: &Resumen) -> Veredicto {
    if r.transmision_s < ESTABLE_S {
        let detalle =
            format!("transmitió {} s; la prueba pide {} min seguidos", r.transmision_s, ESTABLE_S / 60);
        return veredicto(9, "estabilidad", SinDatos, detalle);
    }
    let (mut peor, cuadros) = fps(r).unwrap_or((SinDatos, "sin cuadros medidos".into()));
    peor = peor.max(match r.reconexiones {
        0 => Bien,
        1 => Aviso,
        _ => Mal,
    });
    if let Some(p) = r.perdida_media_pct {
        peor = peor.max(hasta(p, PERDIDA_PCT.0, PERDIDA_PCT.1));
    }
    let detalle = format!("{} min, {cuadros}, {} caídas", r.transmision_s / 60, r.reconexiones);
    veredicto(9, "estabilidad", peor, detalle)
}

fn enlace(r: &Resumen) -> Veredicto {
    if r.senal_min_dbm.is_none() && r.rtt_media_ms.is_none() {
        return veredicto(10, "enlace", SinDatos, "el radio no informó señal ni hubo eco RTCP".into());
    }
    let mut partes = vec![];
    let mut peor = Bien;
    if let (Some(min), Some(media)) = (r.senal_min_dbm, r.senal_media_dbm) {
        peor = peor.max(desde(min, SENAL_DBM.0, SENAL_DBM.1));
        partes.push(format!(
            "señal {} dBm en promedio, {} la peor (bien desde {})",
            n(media),
            n(min),
            SENAL_DBM.0
        ));
    }
    if let Some(rtt) = r.rtt_media_ms {
        peor = peor.max(hasta(rtt, RTT_MS.0, RTT_MS.1));
        partes.push(format!("ida y vuelta {} ms", n(rtt)));
    }
    if let Some(tx) = r.tx_mbps_media {
        partes.push(format!("radio a {} Mbit/s", n(tx)));
    }
    if r.ahorro_apagado == Some(false) {
        peor = peor.max(Aviso);
        partes.push("el ahorro de energía del radio seguía activo (suma hasta 100 ms)".into());
    }
    veredicto(10, "enlace", peor, partes.join("; "))
}

fn reconexion(r: &Resumen) -> Veredicto {
    if r.reconexiones == 0 {
        return veredicto(11, "reconexion", SinDatos, "no hubo caídas que recuperar".into());
    }
    // A fall still pending when the user stopped is not a failure to recover; giving up is.
    let sin_volver = if r.motivo == "fallo" { r.reconexiones.saturating_sub(r.recuperadas) } else { 0 };
    let (resultado, detalle) = match r.recuperacion_max_s {
        _ if sin_volver > 0 => {
            (Mal, format!("{sin_volver} de {} caídas no volvieron a proyectar", r.reconexiones))
        }
        Some(max) => {
            let media = r.recuperacion_media_s.unwrap_or(max);
            (
                hasta(max, RECUPERACION_S.0, RECUPERACION_S.1),
                format!("{} caídas, volvió en {} s (peor {} s)", r.reconexiones, n(media), n(max)),
            )
        }
        None => (SinDatos, "no se midió la recuperación".into()),
    };
    veredicto(11, "reconexion", resultado, detalle)
}
