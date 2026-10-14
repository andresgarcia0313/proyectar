use super::{armar, leer};
use serde_json::{json, Value};

fn ev(hora: &str, evento: &str, datos: Value) -> Value {
    let mut e = json!({"hora": format!("2026-10-13 {hora}.000"), "evento": evento});
    e.as_object_mut().unwrap().extend(datos.as_object().unwrap().clone());
    e
}

/// A 0.4.0 log: no `resumen_sesion`, flat `estadisticas`, one failure and a reconnection.
fn sesion_vieja() -> Vec<Value> {
    vec![
        ev("16:51:18", "inicio", json!({})),
        ev("16:51:34", "captura", json!({"fuente": "Duplicar"})),
        ev("16:51:34", "invitar", json!({"pantalla": "[TV] Sala"})),
        ev("16:51:49", "transmitiendo", json!({"opciones": {"fuente": "Duplicar", "latencia": "Trabajo"}})),
        ev("16:51:59", "estadisticas", json!({"fps": 30.0, "kbps": 9000})),
        ev(
            "16:52:09",
            "estadisticas",
            json!({"video": {"fps": 20.0, "kbps": 7000}, "radio": {"senal_dbm": -61}}),
        ),
        ev("16:52:10", "fallo", json!({"fase": "EnlaceCaido"})),
        ev("16:52:10", "estado", json!({"estado": "Reconectando { intento: 1 }"})),
        ev("16:56:03", "estado", json!({"estado": "Inactivo"})),
    ]
}

#[test]
fn sin_resumen_reconstruye_las_metricas_de_estadisticas_y_fallos() {
    let mut eventos = sesion_vieja();
    let sin_tv = ev("16:52:20", "fallo", json!({"fase": "SinRespuestaTv"}));
    eventos.splice(8..8, [sin_tv.clone(), sin_tv]);
    let t = armar(&eventos, 5);
    assert!(t.contains("Informe de Proyectar: 1 de 1 sesiones"), "{t}");
    assert!(t.contains("== Sesión 1: [TV] Sala =="), "{t}");
    assert!(t.contains("Modo: Duplicar, Trabajo | Duración: 4 min 29 s"), "{t}");
    assert!(t.contains("FPS medio 25 - tasa media 8000 kb/s - señal mínima -61 dBm - reconexiones 1"), "{t}");
    assert!(t.contains("Fallos (3): EnlaceCaido, SinRespuestaTv x2\n"), "{t}");
    assert!(t.contains("no trae resumen_sesion"), "{t}");
    assert!(t.contains("No hay autodiagnóstico registrado"), "{t}");
}

#[test]
fn con_resumen_muestra_la_tabla_de_veredictos_aunque_falten_campos() {
    let mut eventos = sesion_vieja();
    let veredictos = json!([{"prueba": "9 estabilidad", "resultado": "bien", "detalle": "10 min sin cortes"},
                            {"prueba": 4, "resultado": "sin_datos"}]);
    let resumen = json!({"pantalla": "[TV] Sala", "modo": "Juego", "duracion_s": 612.4,
                         "fps_media": 59.94, "rtt_media_ms": null, "veredictos": veredictos});
    eventos.insert(8, ev("16:56:03", "resumen_sesion", resumen));
    let t = armar(&eventos, 5);
    assert!(t.contains("1 de 1 sesiones"), "el resumen cierra la sesión abierta: {t}");
    assert!(t.contains("Juego | Duración: 10 min 12 s"), "{t}");
    assert!(t.contains("Métricas: FPS medio 59,9\n"), "solo lo que trae el resumen: {t}");
    assert!(t
        .lines()
        .any(|l| l.starts_with("9 estabilidad") && l.contains("bien") && l.ends_with("10 min sin cortes")));
    assert!(t.lines().any(|l| l.starts_with('4') && l.contains("sin_datos") && l.ends_with('-')), "{t}");
}

#[test]
fn un_resumen_tras_inactivo_se_une_a_la_ultima_sesion() {
    let mut eventos = sesion_vieja();
    eventos.push(ev("16:56:04", "resumen_sesion", json!({"veredictos": []})));
    let t = armar(&eventos, 5);
    assert!(t.contains("1 de 1 sesiones") && t.contains("[TV] Sala"), "{t}");
}

#[test]
fn muestra_solo_las_ultimas_n_sesiones_y_el_ultimo_autodiagnostico() {
    let mut eventos = [sesion_vieja(), sesion_vieja(), sesion_vieja()].concat();
    let checks = |ok: Value| json!({"checks": [{"nombre": "dispatcher", "ok": ok, "detalle": "falta"}]});
    eventos.push(ev("17:00:00", "autodiagnostico", checks(json!(true))));
    eventos.push(ev("17:01:00", "autodiagnostico", checks(Value::Null)));
    let t = armar(&eventos, 2);
    assert!(t.contains("2 de 3 sesiones") && t.contains("Sesión 3") && !t.contains("Sesión 1:"), "{t}");
    assert!(t.contains("Autodiagnóstico (2026-10-13 17:01:00.000)"), "{t}");
    assert!(t.lines().any(|l| l.starts_with("dispatcher") && l.contains("sin dato")), "{t}");
}

#[test]
fn lee_primero_el_rotado_y_salta_lineas_rotas() {
    let dir = std::env::temp_dir().join(format!("proyectar-informe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (viejo, actual) = (dir.join("e.1.jsonl"), dir.join("e.jsonl"));
    std::fs::write(&viejo, "{\"evento\":\"a\"}\n{\"evento\":\n").unwrap();
    std::fs::write(&actual, "{\"evento\":\"b\"}\n").unwrap();
    let eventos = leer(&[viejo, actual, dir.join("no-existe.jsonl")]);
    let nombres: Vec<&str> = eventos.iter().filter_map(|e| e["evento"].as_str()).collect();
    assert_eq!(nombres, ["a", "b"]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn un_registro_sin_sesiones_lo_dice() {
    let t = armar(&[ev("10:00:00", "inicio", json!({}))], 5);
    assert!(t.contains("0 de 0 sesiones") && t.contains("No hay sesiones"), "{t}");
}
