//! Miracast displays over Wi-Fi Direct through NetworkManager; the normal Wi-Fi stays up.

use crate::enlace::Enlace;
use crate::fallo::{Fallo, Resultado};
use crate::nm::*;
pub use crate::pantalla::{limpio, Pantalla};
use std::collections::HashMap;
use wfd::ie;
use zbus::zvariant::{OwnedObjectPath, Value};

pub struct WifiDirect {
    conn: zbus::Connection,
    nm: NetworkManagerProxy<'static>,
    p2p: WifiP2PProxy<'static>,
    device: OwnedObjectPath,
}

impl WifiDirect {
    pub async fn abrir() -> Resultado<Self> {
        let conn = zbus::Connection::system().await?;
        let nm = NetworkManagerProxy::new(&conn).await?;
        for path in nm.get_devices().await? {
            let device = DeviceProxy::builder(&conn).path(path.clone())?.build().await?;
            if device.device_type().await? == DEVICE_TYPE_P2P {
                let p2p = WifiP2PProxy::builder(&conn).path(path.clone())?.build().await?;
                tokio::spawn(crate::vigia::vigilar(conn.clone(), path.clone()));
                return Ok(Self { conn, nm, p2p, device: path });
            }
        }
        Err(Fallo::WifiDirect("esta tarjeta Wi-Fi no ofrece Wi-Fi Direct".into()))
    }

    /// Starts (or extends) a 30 s discovery; call again before it runs out.
    pub async fn buscar(&self) -> Resultado<()> {
        let options = HashMap::from([("timeout", Value::from(30i32))]);
        Ok(self.p2p.start_find(options).await?)
    }

    pub async fn parar(&self) {
        let _ = self.p2p.stop_find().await;
    }

    /// Displays seen so far: peers whose Wi-Fi Display element says they are sinks.
    pub async fn pantallas(&self) -> Resultado<Vec<Pantalla>> {
        let mut found = Vec::new();
        for path in self.p2p.peers().await? {
            let peer = PeerProxy::builder(&self.conn).path(path.clone())?.build().await?;
            let ies = peer.wfd_ies().await.unwrap_or_default();
            if !ie::device_type(&ies).is_some_and(ie::DeviceType::is_sink) {
                continue;
            }
            let mac = peer.hw_address().await?;
            let nombre =
                peer.name().await.ok().map(|n| limpio(&n)).filter(|n| !n.is_empty()).unwrap_or(mac.clone());
            let senal = peer.strength().await.unwrap_or(0);
            found.push(Pantalla { nombre, mac, senal, ruta: path.to_string() });
        }
        found.sort_by(|a, b| b.senal.cmp(&a.senal));
        Ok(found)
    }

    /// Links to `pantalla`; NetworkManager negotiates who leads the group and runs DHCP.
    /// The connection is volatile: NetworkManager forgets it once it goes down.
    pub async fn conectar(&self, pantalla: &Pantalla, nombre_propio: &str) -> Resultado<Enlace> {
        let settings: Settings = HashMap::from([
            (
                "connection",
                HashMap::from([
                    ("id", Value::from("Proyectar")),
                    ("type", Value::from("wifi-p2p")),
                    ("autoconnect", Value::from(false)),
                ]),
            ),
            (
                "wifi-p2p",
                HashMap::from([
                    ("peer", Value::from(pantalla.mac.as_str())),
                    ("wfd-ies", Value::from(ie::source_ies(nombre_propio))),
                ]),
            ),
            ("ipv4", HashMap::from([("method", Value::from("auto")), ("never-default", Value::from(true))])),
            ("ipv6", HashMap::from([("method", Value::from("ignore"))])),
        ]);
        let peer = zbus::zvariant::ObjectPath::try_from(pantalla.ruta.as_str())
            .map_err(|e| Fallo::WifiDirect(e.to_string()))?;
        let options = HashMap::from([("persist", Value::from("volatile"))]);
        let (_, active, _) =
            self.nm.add_and_activate_connection2(settings, &self.device, &peer, options).await?;
        let active = ActiveProxy::builder(&self.conn)
            .path(active)?
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await?;
        Ok(Enlace { conn: self.conn.clone(), nm: self.nm.clone(), active })
    }
}
