//! One attempt at projecting: link, wait for the TV, negotiate, stream until something
//! ends it. Every exit path closes what it opened; the caller decides whether to retry.

use crate::captura::Captura;
use crate::control;
use crate::enlace::nombre_propio;
use crate::fallo::{Fallo, Resultado};
use crate::medios::{Codificador, Medios, Origen};
use crate::wifi_direct::{Pantalla, WifiDirect};
use crate::{Aviso, Estado, Opciones};
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
}

pub async fn intentar(cx: Contexto<'_>) -> Resultado<Fin> {
    let nombre = cx.pantalla.nombre.clone();
    (cx.avisar)(Aviso::Estado(Estado::Conectando { nombre: nombre.clone() }));
    let enlace = cx.wd.conectar(cx.pantalla, &nombre_propio()).await?;
    (cx.avisar)(Aviso::Estado(Estado::EsperandoPantalla { nombre: nombre.clone() }));
    let resultado = async {
        let red = tokio::select! {
            r = tokio::time::timeout(ESPERA_ENLACE, enlace.listo()) => r.map_err(|_| Fallo::SinRespuesta)??,
            _ = pedido(cx.parar) => return Ok(Fin::Terminada),
        };
        let tcp = tokio::select! {
            r = control::esperar_tv(red) => r?,
            r = enlace.caido() => return Err(Fallo::WifiDirect(format!("el enlace se cayó (código {})", r?))),
            _ = pedido(cx.parar) => return Ok(Fin::Terminada),
        };
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
        let (mut sesion, acordado) = wfd::Session::negotiate(tcp, &quiero).await?;
        let origen = Origen::PipeWire { fd: cx.captura.fd.try_clone()?, nodo: cx.captura.nodo };
        let destino = SocketAddr::new(remoto.ip(), acordado.rtp_port);
        let medios = Medios::iniciar(origen, destino, acordado.video, acordado.audio, perfil, cx.codificador, opciones.audio)?;
        (cx.avisar)(Aviso::Estado(Estado::Proyectando { nombre: nombre.clone() }));
        let mut reloj = tokio::time::interval(Duration::from_secs(1));
        let fin = loop {
            tokio::select! {
                evento = sesion.next_event() => match evento? {
                    wfd::Event::IdrRequest => medios.pedir_idr(),
                    wfd::Event::Teardown => break Ok(Fin::Terminada),
                    wfd::Event::Play | wfd::Event::Pause => {}
                },
                fallo = medios.fallo() => break Err(fallo),
                r = enlace.caido() => break Err(Fallo::WifiDirect(format!("el enlace se cayó (código {})", r?))),
                _ = pedido(cx.parar) => {
                    let _ = tokio::time::timeout(Duration::from_secs(3), sesion.close()).await;
                    return Ok(Fin::Terminada);
                }
                _ = cx.opciones.changed() => break Ok(Fin::Repetir),
                _ = reloj.tick() => (cx.avisar)(Aviso::Estadisticas(medios.estadisticas())),
            }
        };
        drop(medios);
        fin
    }
    .await;
    enlace.cerrar().await;
    resultado
}

/// Resolves when the user asks to stop (or nobody can ask any more); holds no lock across awaits.
pub async fn pedido(parar: &mut watch::Receiver<bool>) {
    let _ = parar.wait_for(|p| *p).await.map(|_| ());
}
