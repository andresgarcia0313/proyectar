//! Startup self-check: what this computer needs to project, logged as `autodiagnostico` so
//! `proyectar --informe` can show it next to the sessions. Needs no root and never fails.

use serde_json::{json, Value};
use std::process::Command;

pub const DISPATCHER: &str = "/etc/NetworkManager/dispatcher.d/90-proyectar-nombre";
pub const CODIFICADORES: [&str; 3] = ["vah264lpenc", "x264enc", "vapostproc"];

/// `ok`: `Some(true)` met, `Some(false)` missing, `None` could not be checked (`detalle` says why).
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub nombre: String,
    pub ok: Option<bool>,
    pub detalle: String,
}

pub fn check(nombre: impl Into<String>, ok: Option<bool>, detalle: impl Into<String>) -> Check {
    Check { nombre: nombre.into(), ok, detalle: detalle.into() }
}

/// Runs every check and logs them; meant for a blocking thread, it shells out and opens sockets.
pub fn anotar() {
    let checks: Vec<Value> =
        revisar().iter().map(|c| json!({"nombre": c.nombre, "ok": c.ok, "detalle": c.detalle})).collect();
    crate::registro::anotar("autodiagnostico", json!({"checks": checks}));
}

pub fn revisar() -> Vec<Check> {
    let mut checks = vec![wifi_direct(), dispatcher(), crate::autodiagnostico_wpa::nombre_wifi_direct()];
    checks.extend(CODIFICADORES.map(codificador));
    checks.push(crate::autodiagnostico_sistema::kwin());
    checks.extend(["pactl", "pw-metadata", "iw"].map(programa));
    checks.push(crate::autodiagnostico_sistema::puerto(5001));
    checks.extend(crate::autodiagnostico_sistema::ahorro());
    checks
}

/// `(salió bien, stdout + stderr)`; `None` when the program is not there.
pub fn correr(programa: &str, args: &[&str]) -> Option<(bool, String)> {
    let salida = Command::new(programa).args(args).output().ok()?;
    let texto = String::from_utf8_lossy(&salida.stdout) + String::from_utf8_lossy(&salida.stderr);
    Some((salida.status.success(), texto.into_owned()))
}

fn wifi_direct() -> Check {
    let Some((true, texto)) = correr("nmcli", &["-t", "-f", "DEVICE,TYPE", "device"]) else {
        return check("wifi_direct", None, "no se pudo consultar NetworkManager con nmcli");
    };
    match texto.lines().find_map(|l| l.strip_suffix(":wifi-p2p")) {
        Some(d) => check("wifi_direct", Some(true), format!("dispositivo {d}")),
        None => check("wifi_direct", Some(false), "NetworkManager no tiene dispositivo Wi-Fi Direct"),
    }
}

fn dispatcher() -> Check {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(DISPATCHER) {
        Ok(m) if m.permissions().mode() & 0o111 != 0 => check("dispatcher", Some(true), DISPATCHER),
        Ok(_) => check("dispatcher", Some(false), format!("{DISPATCHER} no es ejecutable")),
        Err(_) => check("dispatcher", Some(false), "falta el script: el TV puede mostrar el nombre \"0\""),
    }
}

fn codificador(nombre: &str) -> Check {
    let hay = gstreamer::init().is_ok() && gstreamer::ElementFactory::find(nombre).is_some();
    check(nombre, Some(hay), if hay { "disponible en GStreamer" } else { "no está instalado" })
}

fn programa(nombre: &str) -> Check {
    let rutas = std::env::var("PATH").unwrap_or_default() + ":/usr/sbin:/sbin";
    let hay = rutas.split(':').any(|d| std::path::Path::new(d).join(nombre).is_file());
    check(nombre, Some(hay), if hay { "instalado" } else { "no está instalado" })
}
