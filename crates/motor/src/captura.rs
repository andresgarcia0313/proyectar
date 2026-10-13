//! Screen capture through the desktop portal (works on Wayland and X11). KDE's portal
//! can also create a virtual screen, which is how "Extender" adds a display for the TV.

use crate::fallo::{Fallo, Resultado};
use crate::modo::Fuente;
use crate::registro;
use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
use ashpd::desktop::{PersistMode, ResponseError, Session};
use serde_json::json;
use std::os::fd::OwnedFd;

/// A running capture: PipeWire connection and node to read, until dropped or closed.
pub struct Captura {
    pub fuente: Fuente,
    pub fd: OwnedFd,
    pub nodo: u32,
    pub tamano: Option<(i32, i32)>,
    /// Hand to the next request: the portal then skips its dialog (and keeps the
    /// virtual screen's size).
    pub token: Option<String>,
    sesion: Session<'static, Screencast<'static>>,
}

impl Captura {
    /// Asks the portal for a stream. The first time the user picks the screen or window in
    /// the portal's dialog; with a `token` from an earlier run the same choice comes back.
    pub async fn pedir(fuente: Fuente, token: Option<&str>) -> Resultado<Self> {
        let inicio = std::time::Instant::now();
        let portal = Screencast::new().await.map_err(captura)?;
        let sesion = portal.create_session().await.map_err(captura)?;
        let tipo = match fuente {
            Fuente::Duplicar => SourceType::Monitor,
            Fuente::Extender => SourceType::Virtual,
            Fuente::Ventana => SourceType::Window,
        };
        // Only KDE offers virtual screens today; elsewhere "Extender" mirrors a screen instead.
        let ofrece = portal.available_source_types().await.map(|t| t.contains(tipo)).unwrap_or(true);
        let tipo = if ofrece { tipo } else { SourceType::Monitor };
        portal
            .select_sources(
                &sesion,
                CursorMode::Embedded,
                tipo.into(),
                false,
                token,
                PersistMode::ExplicitlyRevoked,
            )
            .await
            .map_err(captura)?;
        let respuesta = portal.start(&sesion, None).await.map_err(captura)?.response().map_err(captura)?;
        let flujo = respuesta
            .streams()
            .first()
            .ok_or_else(|| Fallo::Captura("el portal no entregó imagen".into()))?;
        let (nodo, tamano) = (flujo.pipe_wire_node_id(), flujo.size());
        let (origen, guardado) = (format!("{:?}", flujo.source_type()), token.is_some());
        let segundos = inicio.elapsed().as_secs_f32();
        let datos = json!({"fuente": fuente, "pedido": format!("{tipo:?}"), "entregado": origen, "nodo": nodo,
            "tamano": tamano, "permiso_guardado": guardado, "segundos": segundos});
        registro::anotar("captura", datos);
        let token = respuesta.restore_token().map(str::to_owned);
        let fd = portal.open_pipe_wire_remote(&sesion).await.map_err(captura)?;
        Ok(Self { fuente, fd, nodo, tamano, token, sesion })
    }

    /// Ends the capture; for "Extender" this removes the virtual screen.
    pub async fn cerrar(self) {
        let _ = self.sesion.close().await;
    }

    pub async fn soltar(captura: Option<Self>) {
        if let Some(captura) = captura {
            captura.cerrar().await;
        }
    }
}

fn captura(e: ashpd::Error) -> Fallo {
    match e {
        ashpd::Error::Response(ResponseError::Cancelled) => Fallo::CapturaCancelada,
        other => Fallo::Captura(other.to_string()),
    }
}
