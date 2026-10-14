//! Test 7 (Extender) seen from KDE: after the capture starts, `kscreen-doctor -j` must list a
//! screen named after the TV with the size asked for, and once the capture is released it must
//! be gone. Only looks; never creates or changes a screen.

use crate::captura::{Captura, Tamano};
use crate::modo::Fuente;
use crate::registro;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// KWin adds and removes the screen a moment after the stream starts or stops.
const ESPERA: Duration = Duration::from_secs(3);
const PASO: Duration = Duration::from_millis(250);

/// After `Captura::pedir` for Extender: `Some(true)` when the screen is there with `pedido`.
pub async fn comprobar(captura: &Captura, nombre: &str, pedido: Tamano) -> Option<bool> {
    if captura.fuente != Fuente::Extender || captura.es_portal() {
        return None;
    }
    let inicio = Instant::now();
    let visto = esperar(nombre, |t| t == Some(pedido)).await;
    let ok = visto.as_ref().ok().map(|t| *t == Some(pedido));
    let (visto, detalle) = match visto {
        Ok(t) => (t, None),
        Err(e) => (None, Some(e)),
    };
    let ms = inicio.elapsed().as_millis() as u64;
    registro::anotar(
        "salida_virtual",
        json!({"nombre": nombre, "pedido": pedido, "visto": visto, "ok": ok, "ms": ms, "detalle": detalle}),
    );
    ok
}

/// Releases the capture; for a KWin Extender, checks that its screen went away.
pub async fn soltar(captura: Option<Captura>, nombre: &str) -> Option<bool> {
    let vigilar = captura.as_ref().is_some_and(|c| c.fuente == Fuente::Extender && !c.es_portal());
    Captura::soltar(captura).await;
    if !vigilar {
        return None;
    }
    let inicio = Instant::now();
    let quitada = esperar(nombre, |t| t.is_none()).await.ok().map(|t| t.is_none());
    let ms = inicio.elapsed().as_millis() as u64;
    registro::anotar("salida_virtual_quitada", json!({"nombre": nombre, "quitada": quitada, "ms": ms}));
    quitada
}

/// Polls the screens until `listo` holds for the TV's one or `ESPERA` runs out.
async fn esperar(nombre: &str, listo: impl Fn(Option<Tamano>) -> bool) -> Result<Option<Tamano>, String> {
    let inicio = Instant::now();
    loop {
        let visto = salidas().await.map(|v| buscar(&v, nombre))?;
        if listo(visto) || inicio.elapsed() > ESPERA {
            return Ok(visto);
        }
        tokio::time::sleep(PASO).await;
    }
}

async fn salidas() -> Result<Value, String> {
    let leer = || std::process::Command::new("kscreen-doctor").arg("-j").output();
    let salida = tokio::task::spawn_blocking(leer).await.map_err(|e| e.to_string())?;
    let salida = salida.map_err(|e| format!("kscreen-doctor no disponible: {e}"))?;
    serde_json::from_slice(&salida.stdout).map_err(|e| format!("kscreen-doctor -j ilegible: {e}"))
}

/// Size of the enabled screen named `nombre` (KWin may prefix virtual ones with `Virtual-`).
pub fn buscar(json: &Value, nombre: &str) -> Option<Tamano> {
    let es = |n: &str| n == nombre || n.strip_prefix("Virtual-") == Some(nombre);
    let salida = json["outputs"]
        .as_array()?
        .iter()
        .find(|o| o["enabled"].as_bool() == Some(true) && o["name"].as_str().is_some_and(es))?;
    let actual =
        salida["modes"].as_array().and_then(|m| m.iter().find(|m| m["id"] == salida["currentModeId"]));
    let tamano = actual.map_or(&salida["size"], |m| &m["size"]);
    Some((tamano["width"].as_u64()? as u32, tamano["height"].as_u64()? as u32))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn encuentra_la_pantalla_del_tv_con_su_tamano() {
        let v = json!({"outputs": [
            {"name": "eDP-1", "enabled": true, "currentModeId": "1", "modes": [{"id": "1", "size": {"width": 1920, "height": 1080}}]},
            {"name": "Virtual-[TV] Sala", "enabled": true, "currentModeId": "7",
             "modes": [{"id": "7", "size": {"width": 1280, "height": 720}}], "size": {"width": 1, "height": 1}},
            {"name": "Cocina", "enabled": false, "size": {"width": 1920, "height": 1080}},
        ]});
        assert_eq!(super::buscar(&v, "[TV] Sala"), Some((1280, 720)));
        assert_eq!(super::buscar(&v, "Cocina"), None, "apagada es como quitada");
        assert_eq!(super::buscar(&json!({}), "x"), None);
    }
}
