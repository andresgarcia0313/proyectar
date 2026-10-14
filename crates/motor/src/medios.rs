//! Media pipeline: screen and sound -> H.264 + AAC -> MPEG-TS -> RTP -> paced UDP. Text in
//! `medios_cadena`, assembly in `medios_montaje`, sending in `envio`; here its lifetime, IDR
//! requests, statistics and failures.

use crate::envio::Emisor;
use crate::fallo::{Fallo, Resultado};
use crate::medios_estado::{evento_idr, Aviso, Clase};
use crate::medios_medidas::Medidas;
use crate::medios_montaje::montar;
use crate::medios_ritmo::{aplicar, Ritmo};
pub use crate::medios_tipos::{Codificador, Origen};
use crate::modo::Perfil;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::net::SocketAddr;
use std::os::fd::OwnedFd;
use std::sync::Arc;
use tokio::sync::watch;
use wfd::{AudioCodec, VideoMode};

pub struct Medios {
    pub(crate) pipeline: gst::Pipeline,
    pub(crate) codificador: Codificador,
    pub(crate) modo: VideoMode,
    pub(crate) medidas: Medidas,
    pub(crate) aviso: watch::Receiver<Aviso>,
    pub(crate) ritmo: Arc<Ritmo>,
    pub(crate) emisor: Emisor,
    pub(crate) _fd: Option<OwnedFd>,
}

impl Medios {
    pub fn iniciar(
        origen: Origen,
        destino: SocketAddr,
        modo: VideoMode,
        audio: Option<AudioCodec>,
        perfil: Perfil,
        codificador: Codificador,
        sonido_tv: bool,
    ) -> Resultado<Medios> {
        let m = montar(origen, destino, &modo, audio, perfil, codificador, sonido_tv)?;
        m.tuberia.set_state(gst::State::Playing).map_err(|e| Fallo::Medios(e.to_string()))?;
        Ok(Medios {
            pipeline: m.tuberia,
            codificador,
            modo,
            medidas: m.medidas,
            aviso: m.aviso,
            ritmo: m.ritmo,
            emisor: m.emisor,
            _fd: m.fd,
        })
    }

    /// IDR with SPS/PPS now (the sink lost the picture); requests in a row lower the bitrate.
    pub fn pedir_idr(&self) {
        if let Some(cod) = self.pipeline.by_name("cod") {
            cod.send_event(evento_idr());
            if let Some(kbps) = self.ritmo.perdida() {
                aplicar(&cod, self.codificador, kbps);
            }
        }
    }

    /// Waits for the first bus error and says which stage it came from.
    pub async fn fallo(&self) -> Fallo {
        match self.aviso.clone().wait_for(|v| v.is_some()).await.map(|v| v.clone()) {
            Ok(Some((Clase::Codificador, detalle))) => {
                Fallo::Codificador { codificador: self.codificador.nombre(), detalle }
            }
            Ok(Some((Clase::Captura, detalle))) => Fallo::Captura(detalle),
            Ok(Some((Clase::Otra, detalle))) => Fallo::Medios(detalle),
            _ => Fallo::Medios("la tubería se cerró".into()),
        }
    }

    pub fn tuberia(&self) -> &gst::Pipeline {
        &self.pipeline
    }
}

impl Drop for Medios {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
