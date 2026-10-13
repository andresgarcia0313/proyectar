//! Screen capture. On KDE straight from KWin (no dialog; "Extender" adds a screen named after
//! the TV with the profile's size); elsewhere, and for single windows, through the portal.

#[path = "captura_kwin_estado.rs"]
mod estado;
#[path = "captura_kwin_eventos.rs"]
mod eventos;
#[path = "captura_kwin.rs"]
mod kwin;
#[path = "captura_portal.rs"]
mod portal;
#[path = "captura_kwin_protocolo.rs"]
mod protocolo;

use crate::fallo::Resultado;
use crate::medios::Origen;
use crate::modo::{Fuente, Perfil};
use ashpd::desktop::screencast::Screencast;
use ashpd::desktop::Session;
use std::os::fd::OwnedFd;

/// Width and height in pixels.
pub type Tamano = (u32, u32);

/// A running capture: PipeWire node to read (and the portal's connection to it, if any).
pub struct Captura {
    pub fuente: Fuente,
    /// Only the portal gives one; KWin's nodes are on the session's PipeWire.
    pub fd: Option<OwnedFd>,
    pub nodo: u32,
    pub tamano: Option<(i32, i32)>,
    /// Size asked for: a virtual screen keeps it, so a new profile needs a new capture.
    pub pedido: Tamano,
    /// The portal's saved choice, for the next request; `None` with KWin.
    pub token: Option<String>,
    via: Via,
}

enum Via {
    Kwin(kwin::Kwin),
    Portal(Session<'static, Screencast<'static>>),
}

impl Captura {
    /// Starts a capture for `fuente`. `nombre` (the TV) and `tamano` shape the virtual screen
    /// of "Extender"; `token` is the portal's saved choice, used only when KWin is not there.
    pub async fn pedir(fuente: Fuente, token: Option<&str>, nombre: &str, tamano: Tamano) -> Resultado<Self> {
        if let Some(directa) = Self::directa(fuente, nombre, tamano).await? {
            return Ok(directa);
        }
        let portal::Portal { fd, nodo, tamano: real, token, sesion } = portal::pedir(fuente, token).await?;
        Ok(Self { fuente, fd: Some(fd), nodo, tamano: real, pedido: tamano, token, via: Via::Portal(sesion) })
    }

    /// KWin only, never a dialog. `None` when the compositor does not offer it to this program.
    pub async fn directa(fuente: Fuente, nombre: &str, tamano: Tamano) -> Resultado<Option<Self>> {
        let Some(k) = kwin::directa(fuente, nombre, tamano).await? else { return Ok(None) };
        let (nodo, real) = (k.nodo, k.tamano);
        let c = Self { fuente, fd: None, nodo, tamano: real, pedido: tamano, token: None, via: Via::Kwin(k) };
        Ok(Some(c))
    }

    pub fn es_portal(&self) -> bool {
        matches!(self.via, Via::Portal(_))
    }

    /// What the media pipeline reads.
    pub fn origen(&self) -> std::io::Result<Origen> {
        let fd = self.fd.as_ref().map(OwnedFd::try_clone).transpose()?;
        Ok(Origen::PipeWire { fd, nodo: self.nodo })
    }

    /// Ends the capture; for "Extender" this removes the virtual screen.
    pub async fn cerrar(self) {
        match self.via {
            Via::Portal(sesion) => drop(sesion.close().await),
            Via::Kwin(k) => drop(k),
        }
    }

    pub async fn soltar(captura: Option<Self>) {
        if let Some(captura) = captura {
            captura.cerrar().await;
        }
    }
}

/// Virtual screen size for a profile: 16:9 at its height (720 -> 1280x720).
pub fn tamano_para(perfil: Perfil) -> Tamano {
    let alto = perfil.max_height;
    ((alto * 16 / 9 + 1) & !1, alto)
}

#[cfg(test)]
#[path = "captura_pruebas.rs"]
mod pruebas;
