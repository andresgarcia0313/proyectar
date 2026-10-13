//! One attempt at projecting: link, wait for the TV, negotiate, stream until something
//! ends it. Every exit path closes what it opened; the caller decides whether to retry.
//! `fase` records how far the attempt got, so a failure can be explained to the user.

use crate::captura::Captura;
use crate::control;
use crate::diagnostico::Causa;
use crate::enlace::nombre_propio;
use crate::fallo::{Fallo, Resultado};
use crate::medios::{Codificador, Medios, Origen};
use crate::registro;
use crate::transmision;
pub use crate::transmision::pedido;
use crate::wifi_direct::{Pantalla, WifiDirect};
use crate::{Aviso, Estado, Opciones};
use serde_json::json;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::watch;

/// The TV asks "allow this device?" while the link forms: give time to answer.
const ESPERA_ENLACE: Duration = Duration::from_secs(90);

/// How a session that did not fail ended: user or TV closed it, or new options need a new one.
#[derive(Debug, PartialEq, Eq)]
pub enum Fin {
    Terminada,
    Repetir,
}

pub struct Contexto<'a> {
    pub wd: &'a WifiDirect,
    pub pantalla: &'a Pantalla,
    pub captura: &'a Captura,
    pub codificador: Codificador,
    pub opciones: &'a mut watch::Receiver<Opciones>,
    pub parar: &'a mut watch::Receiver<bool>,
    pub avisar: &'a (dyn Fn(Aviso) + Send + Sync),
    pub fase: &'a mut Causa,
}

pub async fn intentar(cx: Contexto<'_>) -> Resultado<Fin> {
    let nombre = cx.pantalla.nombre.clone();
    let estado = |e: fn(String) -> Estado| (cx.avisar)(Aviso::Estado(e(nombre.clone())));
    *cx.fase = Causa::SinRespuestaTv;
    estado(|nombre| Estado::Conectando { nombre });
    let (mac, senal) = (&cx.pantalla.mac, cx.pantalla.senal);
    registro::anotar("invitar", json!({"pantalla": nombre, "mac": mac, "senal": senal}));
    let enlace = cx.wd.conectar(cx.pantalla, &nombre_propio()).await?;
    let resultado = async {
        let red = tokio::select! {
            r = tokio::time::timeout(ESPERA_ENLACE, enlace.listo()) => r.map_err(|_| Fallo::SinRespuesta)??,
            _ = pedido(cx.parar) => return Ok(Fin::Terminada),
        };
        *cx.fase = Causa::SinSesion;
        estado(|nombre| Estado::EsperandoPantalla { nombre });
        let tcp = tokio::select! {
            r = control::esperar_tv(red) => r?,
            r = enlace.caido() => return Err(Fallo::WifiDirect(format!("el enlace se cayó (código {})", r?))),
            _ = pedido(cx.parar) => return Ok(Fin::Terminada),
        };
        *cx.fase = Causa::Rechazada;
        estado(|nombre| Estado::Negociando { nombre });
        // Sinks open the connection before their RTSP stack is ready (gnome-network-displays waits too).
        tokio::time::sleep(Duration::from_millis(500)).await;
        let (local, remoto) = (tcp.local_addr()?, tcp.peer_addr()?);
        let opciones = cx.opciones.borrow_and_update().clone();
        let perfil = opciones.latencia.perfil();
        let quiero = wfd::Wanted {
            max_height: perfil.max_height,
            max_fps: perfil.max_fps,
            audio: true,
            local_ip: local.ip().to_string(),
        };
        let (sesion, acordado) = wfd::Session::negotiate(tcp, &quiero).await?;
        let origen = Origen::PipeWire { fd: cx.captura.fd.try_clone()?, nodo: cx.captura.nodo };
        let destino = SocketAddr::new(remoto.ip(), acordado.rtp_port);
        let medios = Medios::iniciar(
            origen,
            destino,
            acordado.video,
            acordado.audio,
            perfil,
            cx.codificador,
            opciones.audio,
        )?;
        let (formato, cod) = (format!("{acordado:?}"), cx.codificador.nombre());
        registro::anotar(
            "transmitiendo",
            json!({"formato": formato, "codificador": cod, "opciones": opciones}),
        );
        *cx.fase = Causa::EnlaceCaido;
        estado(|nombre| Estado::Proyectando { nombre });
        transmision::transmitir(sesion, medios, &enlace, cx.opciones, cx.parar, cx.avisar).await
    }
    .await;
    enlace.cerrar().await;
    resultado
}
