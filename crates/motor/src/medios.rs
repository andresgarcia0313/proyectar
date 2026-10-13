//! Media pipeline: screen and sound -> H.264 + AAC -> MPEG-TS -> RTP -> UDP. Text in `medios_cadena`,
//! assembly in `medios_montaje`; here its lifetime, IDR requests, statistics and failures.

use crate::fallo::{Fallo, Resultado};
use crate::medios_estado::{evento_idr, Aviso, Clase, Contador};
use crate::medios_montaje::montar;
use crate::medios_ritmo::{aplicar, Ritmo};
pub use crate::medios_tipos::{Codificador, Origen};
use crate::modo::Perfil;
use crate::Estadisticas;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::net::SocketAddr;
use std::os::fd::OwnedFd;
use std::sync::Arc;
use tokio::sync::watch;
use wfd::{AudioCodec, VideoMode};

pub struct Medios {
    pipeline: gst::Pipeline,
    codificador: Codificador,
    modo: VideoMode,
    contador: Arc<Contador>,
    aviso: watch::Receiver<Aviso>,
    ritmo: Ritmo,
    _fd: Option<OwnedFd>,
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
        let ritmo = Ritmo::nuevo(perfil.kbps);
        Ok(Medios {
            pipeline: m.tuberia,
            codificador,
            modo,
            contador: m.contador,
            aviso: m.aviso,
            ritmo,
            _fd: m.fd,
        })
    }

    /// Asks the encoder for an IDR frame with SPS/PPS (the sink lost the picture).
    /// Requests in a row mean packet loss: the bitrate then steps down.
    pub fn pedir_idr(&self) {
        if let Some(cod) = self.pipeline.by_name("cod") {
            cod.send_event(evento_idr());
            if let Some(kbps) = self.ritmo.perdida() {
                aplicar(&cod, self.codificador, kbps);
            }
        }
    }

    /// Called every second; also lets the bitrate climb back after a calm spell.
    pub fn estadisticas(&self) -> Estadisticas {
        if let (Some(kbps), Some(cod)) = (self.ritmo.calma(), self.pipeline.by_name("cod")) {
            aplicar(&cod, self.codificador, kbps);
        }
        let (fps, kbps) = self.contador.tasa();
        Estadisticas {
            ancho: self.modo.width,
            alto: self.modo.height,
            fps,
            kbps,
            gpu: self.codificador.es_gpu(),
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
