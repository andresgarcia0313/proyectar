//! Direct capture through KWin's `zkde_screencast_unstable_v1`: no dialog, no saved permission
//! that could hand back the wrong source, and the virtual screen of "Extender" carries the TV's
//! name and the profile's size. The PipeWire node lives on the session daemon (no descriptor).

use super::estado::{esperar_nodo, texto, Estado, Salida};
use super::protocolo::orden::kde_output_order_v1::KdeOutputOrderV1;
use super::protocolo::screencast::zkde_screencast_stream_unstable_v1::ZkdeScreencastStreamUnstableV1 as Flujo;
use super::protocolo::screencast::zkde_screencast_unstable_v1::{Pointer, ZkdeScreencastUnstableV1};
use crate::fallo::{Fallo, Resultado};
use crate::modo::Fuente;
use crate::registro;
use serde_json::json;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::{Connection, EventQueue, Proxy};

/// A live KWin stream. Dropping it ends the stream and removes a virtual screen.
pub struct Kwin {
    pub nodo: u32,
    pub tamano: Option<(i32, i32)>,
    /// Connector of the mirrored screen (`eDP-1`) or name of the virtual one.
    pub salida: String,
    flujo: Flujo,
    conexion: Connection,
    _cola: EventQueue<Estado>,
}

impl Drop for Kwin {
    fn drop(&mut self) {
        self.flujo.close();
        let _ = self.conexion.flush();
    }
}

/// Asks KWin off the async threads and logs how it went. `None` sends the caller to the portal.
pub async fn directa(fuente: Fuente, nombre: &str, tamano: (u32, u32)) -> Resultado<Option<Kwin>> {
    let (inicio, dueno) = (std::time::Instant::now(), nombre.to_owned());
    let pedido = tokio::task::spawn_blocking(move || pedir(fuente, &dueno, tamano))
        .await
        .map_err(|e| Fallo::Captura(e.to_string()))?;
    let segundos = inicio.elapsed().as_secs_f32();
    let datos = match &pedido {
        Ok(Some(k)) => json!({"nodo": k.nodo, "tamano": k.tamano, "nombre": k.salida, "segundos": segundos}),
        Ok(None) => return Ok(None),
        Err(e) => json!({"error": e, "segundos": segundos}),
    };
    let mut linea = json!({"metodo": "kwin", "fuente": fuente, "pedido": tamano});
    if let (Some(linea), serde_json::Value::Object(datos)) = (linea.as_object_mut(), datos) {
        linea.extend(datos);
    }
    registro::anotar("captura", linea);
    Ok(pedido.ok().flatten())
}

/// `Ok(None)` when this is not KWin, KWin keeps the protocol from this program (its `.desktop`
/// lacks `X-KDE-Wayland-Interfaces`) or the source is a window: the portal handles those.
pub fn pedir(fuente: Fuente, nombre: &str, (ancho, alto): (u32, u32)) -> Result<Option<Kwin>, String> {
    if fuente == Fuente::Ventana {
        return Ok(None);
    }
    let Ok(conexion) = Connection::connect_to_env() else { return Ok(None) };
    let (globales, mut cola) = registry_queue_init::<Estado>(&conexion).map_err(texto)?;
    let qh = cola.handle();
    let Ok(kwin) = globales.bind::<ZkdeScreencastUnstableV1, _, _>(&qh, 1..=6, ()) else { return Ok(None) };
    if fuente == Fuente::Extender && kwin.version() < 2 {
        return Ok(None);
    }
    let mut estado = Estado::default();
    globales.contents().with_list(|lista| {
        for global in lista.iter().filter(|g| g.interface == WlOutput::interface().name) {
            let version = global.version.min(4);
            let objeto = globales.registry().bind::<WlOutput, _, _>(global.name, version, &qh, ());
            estado.salidas.push(Salida { objeto, nombre: None, modo: None });
        }
    });
    let _orden = globales.bind::<KdeOutputOrderV1, _, _>(&qh, 1..=1, ()).ok();
    cola.roundtrip(&mut estado).map_err(texto)?;

    let (puntero, (w, h), n) = (Pointer::Embedded as u32, (ancho as i32, alto as i32), nombre.to_owned());
    let (flujo, tamano, salida) = match fuente {
        Fuente::Extender if kwin.version() >= 4 => {
            let flujo =
                kwin.stream_virtual_output_with_description(n.clone(), n, w, h, 1.0, puntero, &qh, ());
            (flujo, Some((w, h)), nombre.to_owned())
        }
        Fuente::Extender => {
            (kwin.stream_virtual_output(n, w, h, 1.0, puntero, &qh, ()), Some((w, h)), nombre.to_owned())
        }
        _ => {
            let principal = estado.principal().ok_or("KWin no anunció ninguna pantalla")?;
            let flujo = kwin.stream_output(&principal.objeto, puntero, &qh, ());
            (flujo, principal.modo, principal.nombre.clone().unwrap_or_default())
        }
    };
    let nodo = esperar_nodo(&mut cola, &mut estado).inspect_err(|_| flujo.close())?;
    Ok(Some(Kwin { nodo, tamano, salida, flujo, conexion, _cola: cola }))
}
