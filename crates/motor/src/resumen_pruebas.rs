use super::*;
use crate::monitor_carga::Carga;
use crate::{monitor::Radio, Opciones};

pub(crate) fn segundo(fps: f32, retardo_ms: f32, jitter_ms: f32) -> Estadisticas {
    Estadisticas {
        fps,
        kbps: 9_000,
        retardo_ms,
        jitter_ms,
        perdida_pct: 0.5,
        rtt_ms: 4.0,
        ..Default::default()
    }
}

pub(crate) fn radio(senal: i32, reintentos: u64, ahorro: bool) -> Radio {
    let ahorro = Some(ahorro);
    Radio {
        senal_dbm: Some(senal),
        tx_mbps: Some(400.0),
        reintentos: Some(reintentos),
        ahorro,
        ..Default::default()
    }
}

#[test]
fn arranque_sin_contar_y_perdida_solo_con_informes() {
    let mut a = Acumulador::nuevo("TV");
    a.segundo(&Estadisticas { fps: 3.0, ..Default::default() });
    (0..5).for_each(|_| a.segundo(&segundo(30.0, 40.0, 1.0)));
    assert_eq!(
        (a.transmision_s, a.m.fps.largo(), a.m.fps.min()),
        (6, 3, Some(30.0)),
        "los 3 primeros s son arranque"
    );
    assert_eq!(a.m.perdida.largo(), 5, "todo en cero es que aún no llegó un informe RTCP");
    assert_eq!((a.m.retardo.largo(), a.m.rtt.largo()), (5, 5));
}

#[test]
fn los_contadores_de_cada_tuberia_se_suman() {
    let mut a = Acumulador::nuevo("TV");
    for (descartes, reintentos) in [(7, [10, 25]), (2, [3, 4])] {
        a.transmitiendo(&Opciones::default(), "x264enc", 30, "aac".into(), "kwin");
        a.segundo(&Estadisticas { descartes, audio_medido_s: 4, ..Default::default() });
        reintentos.iter().for_each(|n| a.muestra(Some(&radio(-60, *n, false)), 10.0, Carga::default()));
    }
    let r = a.resumen("fin");
    assert_eq!((r.descartes, r.audio_medido_s, r.reintentos_delta), (9, 8, Some(16)));
    assert_eq!((r.codificadores, r.ahorro_apagado), (vec!["x264enc"], Some(true)));
}

#[test]
fn una_caida_cuenta_una_vez_y_se_mide_su_recuperacion() {
    let mut a = Acumulador::nuevo("TV");
    a.caida();
    a.caida();
    a.transmitiendo(&Opciones::default(), "x264enc", 30, "aac".into(), "portal");
    let r = a.resumen("fin");
    assert_eq!((r.reconexiones, r.recuperadas, r.captura), (1, 1, Some("portal")));
    assert!(r.recuperacion_media_s.unwrap() < 1.0);
}
