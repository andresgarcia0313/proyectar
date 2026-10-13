//! A live Wi-Fi Direct link: wait until it is up, learn this laptop's address on it,
//! notice when it drops, and take it down.

use crate::control::Red;
use crate::fallo::{Fallo, Resultado};
use crate::nm::*;
use crate::registro;
use futures_util::StreamExt;
use serde_json::json;

pub struct Enlace {
    pub(crate) conn: zbus::Connection,
    pub(crate) nm: NetworkManagerProxy<'static>,
    pub(crate) active: ActiveProxy<'static>,
}

impl Enlace {
    /// Resolves once the group is formed and DHCP gave this laptop an address.
    /// Follows the state carried by each signal: a cached property can lag behind it and
    /// make the link look down when it is already up (seen with a Samsung TV).
    pub async fn listo(&self) -> Resultado<Red> {
        let mut cambios = self.active.receive_estado_cambiado().await?;
        let mut estado = self.active.state().await?;
        loop {
            registro::anotar("enlace_estado", json!({"estado": estado}));
            match estado {
                ACTIVATED => break,
                DEACTIVATED => return Err(Fallo::WifiDirect("la pantalla no aceptó el enlace".into())),
                _ => {}
            }
            let Some(senal) = cambios.next().await else {
                return Err(Fallo::WifiDirect("NetworkManager dejó de responder".into()));
            };
            let args = senal.args()?;
            if args.state == DEACTIVATED {
                registro::anotar("enlace_rechazado", json!({"motivo": args.reason}));
            }
            estado = args.state;
        }
        let ruta = self.active.ip4_config().await?;
        let config = Ip4ConfigProxy::builder(&self.conn).path(ruta)?.build().await?;
        for dato in config.address_data().await? {
            let ip = dato
                .get("address")
                .and_then(|v| v.downcast_ref::<&str>().ok())
                .and_then(|s| s.parse::<std::net::Ipv4Addr>().ok());
            let prefijo = dato.get("prefix").and_then(|v| v.downcast_ref::<u32>().ok());
            if let (Some(ip), Some(prefijo)) = (ip, prefijo) {
                registro::anotar("enlace_ip", json!({"ip": ip.to_string(), "prefijo": prefijo}));
                return Ok(Red { ip, prefijo: prefijo.min(32) as u8 });
            }
        }
        Err(Fallo::WifiDirect("el enlace no recibió dirección IP".into()))
    }

    /// Resolves when the link drops, with NetworkManager's reason code.
    pub async fn caido(&self) -> Resultado<u32> {
        let mut cambios = self.active.receive_estado_cambiado().await?;
        if self.active.state().await.unwrap_or(DEACTIVATED) == DEACTIVATED {
            return Ok(0);
        }
        while let Some(senal) = cambios.next().await {
            let args = senal.args()?;
            if args.state == DEACTIVATED {
                registro::anotar("enlace_caido", json!({"motivo": args.reason}));
                return Ok(args.reason);
            }
        }
        Ok(0)
    }

    pub async fn cerrar(self) {
        let _ = self.nm.deactivate_connection(self.active.inner().path()).await;
    }
}

/// The name the TV shows for this laptop.
pub fn nombre_propio() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|h| h.trim().to_owned())
        .ok()
        .filter(|h| !h.is_empty())
        .unwrap_or_else(|| "Proyectar".into())
}
