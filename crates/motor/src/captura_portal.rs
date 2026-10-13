//! Capture through the desktop portal, for desktops other than KDE and for single windows.
//! KDE's portal can also create a virtual screen, which is how "Extender" works there.

use crate::fallo::{Fallo, Resultado};
use crate::modo::Fuente;
use crate::registro;
use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
use ashpd::desktop::{PersistMode, ResponseError, Session};
use serde_json::json;
use std::os::fd::OwnedFd;

pub struct Portal {
    pub fd: OwnedFd,
    pub nodo: u32,
    pub tamano: Option<(i32, i32)>,
    /// Hand to the next request: the portal then skips its dialog.
    pub token: Option<String>,
    pub sesion: Session<'static, Screencast<'static>>,
}

/// With a saved `token` the earlier choice comes back without a dialog. KDE may remember
/// "new virtual screen" under a token asked for "Duplicar": that token is dropped and the
/// user asked again, so mirroring never turns into an extra screen.
pub async fn pedir(fuente: Fuente, token: Option<&str>) -> Resultado<Portal> {
    let (portal, entregado) = una_vez(fuente, token).await?;
    if fuente == Fuente::Duplicar && token.is_some() && entregado == Some(SourceType::Virtual) {
        registro::anotar("captura", json!({"metodo": "portal", "descarta_permiso": "entregó Virtual"}));
        let _ = portal.sesion.close().await;
        return Ok(una_vez(fuente, None).await?.0);
    }
    Ok(portal)
}

async fn una_vez(fuente: Fuente, token: Option<&str>) -> Resultado<(Portal, Option<SourceType>)> {
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
    let persistir = PersistMode::ExplicitlyRevoked;
    portal
        .select_sources(&sesion, CursorMode::Embedded, tipo.into(), false, token, persistir)
        .await
        .map_err(captura)?;
    let respuesta = portal.start(&sesion, None).await.map_err(captura)?.response().map_err(captura)?;
    let flujo =
        respuesta.streams().first().ok_or_else(|| Fallo::Captura("el portal no entregó imagen".into()))?;
    let (nodo, tamano, entregado) = (flujo.pipe_wire_node_id(), flujo.size(), flujo.source_type());
    let datos = json!({"metodo": "portal", "fuente": fuente, "pedido": format!("{tipo:?}"),
        "entregado": format!("{entregado:?}"), "nodo": nodo, "tamano": tamano,
        "permiso_guardado": token.is_some(), "segundos": inicio.elapsed().as_secs_f32()});
    registro::anotar("captura", datos);
    let token = respuesta.restore_token().map(str::to_owned);
    let fd = portal.open_pipe_wire_remote(&sesion).await.map_err(captura)?;
    Ok((Portal { fd, nodo, tamano, token, sesion }, entregado))
}

fn captura(e: ashpd::Error) -> Fallo {
    match e {
        ashpd::Error::Response(ResponseError::Cancelled) => Fallo::CapturaCancelada,
        other => Fallo::Captura(other.to_string()),
    }
}
