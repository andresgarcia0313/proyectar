//! A live Wi-Fi Direct link: wait until it is up, learn this laptop's address on it,
//! notice when it drops, and take it down.

use crate::fallo::{Fallo, Resultado};
use crate::nm::*;
use futures_util::StreamExt;
use std::net::{IpAddr, Ipv4Addr};

pub struct Enlace {
    pub(crate) conn: zbus::Connection,
    pub(crate) nm: NetworkManagerProxy<'static>,
    pub(crate) active: ActiveProxy<'static>,
}

/// This laptop's address on the link and the link's prefix length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Red {
    pub ip: Ipv4Addr,
    pub prefijo: u8,
}

impl Red {
    /// Whether `otra` sits on this link: only the display may open the control session.
    pub fn contiene(&self, otra: IpAddr) -> bool {
        let IpAddr::V4(otra) = otra else { return false };
        let mascara = u32::MAX.checked_shl(32 - u32::from(self.prefijo.min(32))).unwrap_or(0);
        u32::from(otra) & mascara == u32::from(self.ip) & mascara
    }
}

impl Enlace {
    /// Resolves once the group is formed and DHCP gave this laptop an address.
    pub async fn listo(&self) -> Resultado<Red> {
        let mut cambios = self.active.receive_estado_cambiado().await?;
        loop {
            match self.active.state().await? {
                ACTIVATED => break,
                DEACTIVATED => return Err(Fallo::WifiDirect("la pantalla no aceptó el enlace".into())),
                _ if cambios.next().await.is_none() => {
                    return Err(Fallo::WifiDirect("NetworkManager dejó de responder".into()))
                }
                _ => {}
            }
        }
        let ruta = self.active.ip4_config().await?;
        let config = Ip4ConfigProxy::builder(&self.conn).path(ruta)?.build().await?;
        for dato in config.address_data().await? {
            let ip =
                dato.get("address").and_then(|v| v.downcast_ref::<&str>().ok()).and_then(|s| s.parse().ok());
            let prefijo = dato.get("prefix").and_then(|v| v.downcast_ref::<u32>().ok());
            if let (Some(ip), Some(prefijo)) = (ip, prefijo) {
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

#[cfg(test)]
mod tests {
    use super::Red;

    #[test]
    fn solo_acepta_la_subred_del_enlace() {
        let red = Red { ip: "192.168.49.1".parse().unwrap(), prefijo: 24 };
        assert!(red.contiene("192.168.49.37".parse().unwrap()));
        assert!(!red.contiene("192.168.1.20".parse().unwrap()));
        assert!(!red.contiene("::1".parse().unwrap()));
        assert!(Red { prefijo: 0, ..red }.contiene("8.8.8.8".parse().unwrap()));
    }
}
