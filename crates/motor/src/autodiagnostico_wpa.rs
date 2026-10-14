//! The name this computer announces over Wi-Fi Direct, as wpa_supplicant holds it after the
//! dispatcher script set it. An empty name shows on some TVs as "0" in the allow prompt.

use crate::autodiagnostico::{check, correr, Check};

/// wpa_supplicant only answers root on most systems; then the name stays unknown.
pub fn nombre_wifi_direct() -> Check {
    const WPA: &str = "fi.w1.wpa_supplicant1";
    let leer = |ruta: &str, interfaz: &str, prop: &str| {
        correr("busctl", &["get-property", WPA, ruta, interfaz, prop])
    };
    match leer("/fi/w1/wpa_supplicant1", WPA, "Interfaces") {
        Some((true, texto)) => {
            let rutas = texto.split('"').skip(1).step_by(2).map(str::to_owned).collect::<Vec<_>>();
            let nombre = rutas.iter().find_map(|r| {
                let (_, t) = leer(r, &format!("{WPA}.Interface.P2PDevice"), "P2PDeviceConfig")?;
                nombre_en(&t)
            });
            match nombre {
                Some(n) => check("nombre_wifi_direct", Some(!n.is_empty()), format!("anuncia \"{n}\"")),
                None => check("nombre_wifi_direct", None, "wpa_supplicant no dio la configuración P2P"),
            }
        }
        Some((false, t)) if t.contains("denied") => {
            check("nombre_wifi_direct", None, "wpa_supplicant solo deja leerlo como administrador")
        }
        _ => check("nombre_wifi_direct", None, "no se pudo consultar wpa_supplicant"),
    }
}

/// `"DeviceName" s "lenovo"` inside busctl's `a{sv}` output.
pub fn nombre_en(texto: &str) -> Option<String> {
    let resto = texto.split("\"DeviceName\" s \"").nth(1)?;
    Some(resto.split('"').next()?.to_owned())
}
