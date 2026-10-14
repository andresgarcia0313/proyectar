use crate::modo::Fuente;
use crate::monitor_carga::Carga;
use crate::resumen::pruebas::{radio, segundo};
use crate::resumen::Acumulador;
use crate::resumen_tipos::Resultado::{self, *};
use crate::{Estadisticas, Opciones};

/// Ten healthy minutes of Duplicar at 30 fps with one fall recovered at once.
fn sana(acumulado: &mut Acumulador) {
    let opciones = Opciones { fuente: Fuente::Duplicar, ..Default::default() };
    acumulado.transmitiendo(&opciones, "vah264lpenc", 30, "aac".into(), "kwin");
    for s in 0..610 {
        let mut e = segundo(30.0, 35.0, 2.0);
        (e.audio_medido_s, e.audio_senal_s, e.descartes) = (s, s / 2, 3);
        acumulado.segundo(&e);
        if s % 10 == 0 {
            acumulado.muestra(Some(&radio(-55, 100 + u64::from(s), false)), 20.0, Carga::default());
        }
    }
    acumulado.audio_restaurada = Some(true);
}

fn resultado(r: &crate::resumen_tipos::Resumen, prueba: u8) -> Resultado {
    r.veredictos.iter().find(|v| v.prueba == prueba).unwrap().resultado
}

#[test]
fn sesion_sana_de_diez_minutos() {
    let mut a = Acumulador::nuevo("TV");
    a.caida();
    sana(&mut a);
    let r = a.resumen("fin");
    assert_eq!(
        (r.fps_media, r.retardo_equipo_p95_ms, r.senal_min_dbm),
        (Some(30.0), Some(35.0), Some(-55.0))
    );
    let todos: Vec<_> = (1..=11).map(|p| resultado(&r, p)).collect();
    let esperado = [SinDatos, Bien, SinDatos, Bien, Bien, Bien, SinDatos, Bien, Aviso, Bien, Bien];
    assert_eq!(todos, esperado, "{:#?}", r.veredictos);
    assert!(r.veredictos[8].detalle.contains("1 caídas"), "una caída en 10 min es aviso");
}

#[test]
fn sesion_mala_y_sesion_corta() {
    let mut a = Acumulador::nuevo("TV");
    sana(&mut a);
    for _ in 0..100 {
        a.segundo(&Estadisticas {
            fps: 12.0,
            retardo_ms: 180.0,
            perdida_pct: 9.0,
            jitter_ms: 30.0,
            ..Default::default()
        });
    }
    a.muestra(Some(&radio(-84, 5_000, true)), 90.0, Carga::default());
    (a.idr, a.audio_restaurada, a.salida_virtual) = (200, Some(false), Some(false));
    a.caida();
    let r = a.resumen("fallo");
    for prueba in [2, 4, 6, 10, 11] {
        assert_eq!(resultado(&r, prueba), Mal, "prueba {prueba}: {:#?}", r.veredictos);
    }
    assert_eq!(resultado(&r, 7), SinDatos, "Duplicar no juzga la pantalla virtual");
    let corta = Acumulador::nuevo("TV").resumen("fin");
    assert!(corta.veredictos.iter().all(|v| v.resultado == SinDatos), "{:#?}", corta.veredictos);
    assert_eq!(corta.veredictos.len(), 11);
}

#[test]
fn la_latencia_medida_se_lee_desde_el_inicio_de_la_sesion() {
    let log = r#"{"hora":"2026-10-13 20:00:00.000","evento":"latencia_medida","ms":300.0}
{"hora":"2026-10-13 21:00:01.000","evento":"latencia_medida","ms":120.5}
{"hora":"2026-10-13 21:00:02.000","evento":"estadisticas"}"#;
    use crate::resumen_informe::ultima_latencia;
    assert_eq!(ultima_latencia(log, "2026-10-13 21:00:00.000"), Some(120.5));
    assert_eq!(ultima_latencia(log, "2026-10-13 22:00:00.000"), None);
}
