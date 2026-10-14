//! Self-check pieces that touch the session and the radio: KWin's capture protocol, the RTCP
//! port and Wi-Fi power save.

use crate::autodiagnostico::{check, correr, Check};
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::{Connection, Dispatch, QueueHandle};

const PROTOCOLO: &str = "zkde_screencast_unstable_v1";
const LANZADOR: &str = "applications/com.ingeniumcodex.proyectar.desktop";

struct Registro;

impl Dispatch<WlRegistry, GlobalListContents> for Registro {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

/// KWin only lists the protocol to apps whose launcher declares it in `X-KDE-Wayland-Interfaces`.
pub fn kwin() -> Check {
    if en_registro() == Some(true) {
        return check("kwin_screencast", Some(true), "KWin ofrece la captura directa a este proceso");
    }
    let lanzadores = [dirs::data_dir(), Some("/usr/share".into())];
    let declarado = lanzadores.into_iter().flatten().any(|d| {
        std::fs::read_to_string(d.join(LANZADOR)).is_ok_and(|t| {
            t.lines().any(|l| l.starts_with("X-KDE-Wayland-Interfaces") && l.contains(PROTOCOLO))
        })
    });
    if declarado {
        check("kwin_screencast", Some(true), "el lanzador lo declara (KWin lo da al abrirse desde él)")
    } else {
        check("kwin_screencast", Some(false), "sin protocolo ni lanzador: se usa el portal, que pide permiso")
    }
}

fn en_registro() -> Option<bool> {
    let conexion = Connection::connect_to_env().ok()?;
    let (globales, _cola) = registry_queue_init::<Registro>(&conexion).ok()?;
    Some(globales.contents().with_list(|l| l.iter().any(|g| g.interface == PROTOCOLO)))
}

/// The TV sends its RTCP reports here; someone else holding it hides loss and jitter. Loopback
/// still collides with a wildcard bind and opens nothing to the network (no firewall prompt).
pub fn puerto(numero: u16) -> Check {
    let nombre = format!("puerto_{numero}");
    match std::net::UdpSocket::bind(("127.0.0.1", numero)) {
        Ok(_) => check(nombre, Some(true), "libre"),
        Err(e) => check(nombre, Some(false), format!("ocupado: {e}")),
    }
}

/// Power save on the Wi-Fi card or the Wi-Fi Direct group delays packets up to 100 ms.
pub fn ahorro() -> Vec<Check> {
    let interfaces: Vec<String> = std::fs::read_dir("/sys/class/net")
        .map(|d| d.filter_map(|e| e.ok()?.file_name().into_string().ok()).collect())
        .unwrap_or_default();
    let mut checks: Vec<Check> = interfaces
        .iter()
        .filter(|n| n.starts_with("wlp") || n.starts_with("p2p-"))
        .map(|n| leer_ahorro(n, correr("iw", &["dev", n, "get", "power_save"]).map(|(_, t)| t)))
        .collect();
    if !interfaces.iter().any(|n| n.starts_with("p2p-")) {
        checks.push(check(
            "ahorro_p2p",
            None,
            "no hay grupo Wi-Fi Direct ahora; el dispatcher lo apaga al crearlo",
        ));
    }
    checks
}

pub fn leer_ahorro(interfaz: &str, salida: Option<String>) -> Check {
    let nombre = format!("ahorro_{interfaz}");
    match salida {
        Some(t) if t.contains(": off") => check(nombre, Some(true), "apagado"),
        Some(t) if t.contains(": on") => {
            check(nombre, Some(false), "encendido: retrasa paquetes hasta 100 ms")
        }
        _ => check(nombre, None, "iw no lo pudo leer"),
    }
}

#[cfg(test)]
#[path = "autodiagnostico_pruebas.rs"]
mod pruebas;
