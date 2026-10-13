//! Logs every state change of the Wi-Fi Direct device with NetworkManager's reason code:
//! the thread to pull when a link fails (the TV did not answer, it left, DHCP failed...).

use crate::registro;
use futures_util::StreamExt;
use serde_json::json;
use zbus::zvariant::OwnedObjectPath;

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
trait Dispositivo {
    #[zbus(signal, name = "StateChanged")]
    fn cambio(&self, new_state: u32, old_state: u32, reason: u32) -> zbus::Result<()>;
}

pub async fn vigilar(conn: zbus::Connection, ruta: OwnedObjectPath) {
    let Ok(constructor) = DispositivoProxy::builder(&conn).path(ruta) else { return };
    let Ok(dispositivo) = constructor.build().await else { return };
    let Ok(mut cambios) = dispositivo.receive_cambio().await else { return };
    while let Some(senal) = cambios.next().await {
        if let Ok(a) = senal.args() {
            let (de, a_estado) = (nombre(a.old_state), nombre(a.new_state));
            registro::anotar("p2p_estado", json!({"de": de, "a": a_estado, "motivo": a.reason}));
        }
    }
}

/// `NMDeviceState` in words.
fn nombre(estado: u32) -> &'static str {
    match estado {
        30 => "desconectado",
        40 => "preparando",
        50 => "configurando",
        60 => "pide-secretos",
        70 => "obteniendo-ip",
        80 => "verificando-ip",
        90 => "secundarias",
        100 => "activo",
        110 => "desactivando",
        120 => "fallo",
        _ => "otro",
    }
}
