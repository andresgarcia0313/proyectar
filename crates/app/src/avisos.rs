//! Turns what the engine reports into what the interface shows: Spanish text (es-CO),
//! a state code and the merged list of displays. Pure functions, no Qt.

use motor::{Estado, Pantalla};
use serde_json::{json, Value};

/// Codes QML switches on; keep in step with `Proyectar/PasosConexion.qml` and `MainWindow.qml`.
pub fn codigo(estado: &Estado) -> i32 {
    match estado {
        Estado::Inactivo => 0,
        Estado::Buscando => 1,
        Estado::Conectando { .. } => 2,
        Estado::EsperandoPantalla { .. } => 3,
        Estado::Proyectando { .. } => 4,
        Estado::Reconectando { .. } => 5,
        Estado::Negociando { .. } => 6,
    }
}

pub fn texto(estado: &Estado) -> String {
    match estado {
        Estado::Inactivo => "Elige una pantalla para proyectar".into(),
        Estado::Buscando => "Buscando…".into(),
        Estado::Conectando { nombre } => format!("Conectando con {nombre}…"),
        Estado::EsperandoPantalla { .. } => "Esperando que el televisor abra el video…".into(),
        Estado::Negociando { nombre } => format!("Preparando el video con {nombre}…"),
        Estado::Proyectando { nombre } => format!("Proyectando en {nombre}"),
        Estado::Reconectando { intento, motivo, .. } => format!("Reconectando (intento {intento}): {motivo}"),
    }
}

pub fn nombre(estado: &Estado) -> String {
    match estado {
        Estado::Inactivo | Estado::Buscando => String::new(),
        Estado::Conectando { nombre }
        | Estado::EsperandoPantalla { nombre }
        | Estado::Negociando { nombre }
        | Estado::Proyectando { nombre }
        | Estado::Reconectando { nombre, .. } => nombre.clone(),
    }
}

/// Recent displays first (newest first, even when out of range), then the rest by signal.
pub fn lista_json(cercanas: &[Pantalla], recientes: &[Pantalla]) -> String {
    let senal = |mac: &str| cercanas.iter().find(|p| p.mac == mac).map(|p| p.senal);
    let mut filas: Vec<Value> = recientes.iter().map(|p| fila(p, senal(&p.mac), true)).collect();
    let mut resto: Vec<&Pantalla> =
        cercanas.iter().filter(|p| !recientes.iter().any(|r| r.mac == p.mac)).collect();
    resto.sort_by(|a, b| b.senal.cmp(&a.senal));
    filas.extend(resto.into_iter().map(|p| fila(p, Some(p.senal), false)));
    Value::Array(filas).to_string()
}

fn fila(p: &Pantalla, senal: Option<u8>, reciente: bool) -> Value {
    json!({"nombre": p.nombre, "mac": p.mac, "senal": senal.unwrap_or(0),
           "cercana": senal.is_some(), "reciente": reciente})
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn pantalla(nombre: &str, mac: &str, senal: u8) -> Pantalla {
        Pantalla { nombre: nombre.into(), mac: mac.into(), senal, ruta: String::new() }
    }

    #[test]
    fn textos_del_estado_en_espanol() {
        let tv = || "Sala".to_string();
        assert_eq!(texto(&Estado::Buscando), "Buscando…");
        assert_eq!(texto(&Estado::Conectando { nombre: tv() }), "Conectando con Sala…");
        assert_eq!(
            texto(&Estado::EsperandoPantalla { nombre: tv() }),
            "Esperando que el televisor abra el video…"
        );
        assert_eq!(texto(&Estado::Proyectando { nombre: tv() }), "Proyectando en Sala");
        assert_eq!(
            texto(&Estado::Reconectando { nombre: tv(), intento: 2, motivo: "Se perdió el enlace".into() }),
            "Reconectando (intento 2): Se perdió el enlace"
        );
    }

    #[test]
    fn recientes_arriba_y_el_resto_por_senal() {
        let cercanas = [pantalla("A", "a", 40), pantalla("B", "b", 90), pantalla("C", "c", 60)];
        let recientes = [pantalla("C", "c", 0), pantalla("Z", "z", 0)];
        let lista: Vec<Value> = serde_json::from_str(&lista_json(&cercanas, &recientes)).unwrap();
        let macs: Vec<&str> = lista.iter().map(|f| f["mac"].as_str().unwrap()).collect();
        assert_eq!(macs, ["c", "z", "b", "a"]);
        assert_eq!(lista[0]["senal"], 60);
        assert_eq!(lista[1]["cercana"], false);
        assert_eq!(lista[2]["reciente"], false);
    }

    #[test]
    fn nombre_vacio_sin_proyeccion() {
        assert_eq!(nombre(&Estado::Buscando), "");
        assert_eq!(codigo(&Estado::Proyectando { nombre: "x".into() }), 4);
    }
}
