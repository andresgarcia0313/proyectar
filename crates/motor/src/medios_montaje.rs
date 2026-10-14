//! Builds the pipeline from its description and wires the measuring probe, the error bus and
//! the paced sender that takes the packets from the appsink.

use crate::envio::Emisor;
use crate::fallo::{Fallo, Resultado};
use crate::medios_cadena::{descripcion, PID_AUDIO, PID_PMT, PID_VIDEO};
use crate::medios_estado::{clasificar, Aviso};
use crate::medios_medidas::Medidas;
use crate::medios_ritmo::Ritmo;
use crate::medios_tipos::{Codificador, Origen};
use crate::modo::Perfil;
use crate::prioridad;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::net::SocketAddr;
use std::os::fd::{AsRawFd, OwnedFd};
use std::sync::Arc;
use tokio::sync::watch;
use wfd::{AudioCodec, VideoMode};

pub struct Montaje {
    pub tuberia: gst::Pipeline,
    pub fd: Option<OwnedFd>,
    pub medidas: Medidas,
    pub aviso: watch::Receiver<Aviso>,
    pub ritmo: Arc<Ritmo>,
    pub emisor: Emisor,
}

fn medios(e: impl std::fmt::Display) -> Fallo {
    Fallo::Medios(e.to_string())
}

#[allow(clippy::too_many_arguments)]
pub fn montar(
    origen: Origen,
    destino: SocketAddr,
    modo: &VideoMode,
    audio: Option<AudioCodec>,
    perfil: Perfil,
    cod: Codificador,
    sonido_tv: bool,
) -> Resultado<Montaje> {
    gst::init().map_err(medios)?;
    let texto = descripcion(&origen, modo, audio, perfil, cod, sonido_tv);
    let tuberia = gst::parse::launch(&texto)
        .map_err(|e| match e.to_string() {
            d if d.contains(cod.nombre()) || d.contains("link pre to cod") || d.contains("vapostproc") => {
                Fallo::Codificador { codificador: cod.nombre(), detalle: d }
            }
            d => Fallo::Medios(d),
        })?
        .downcast::<gst::Pipeline>()
        .map_err(|_| medios("la descripción no es una tubería"))?;
    let fd = match origen {
        Origen::PipeWire { fd, .. } => fd,
        Origen::Prueba => None,
    };
    let elemento = |n: &str| tuberia.by_name(n).ok_or_else(|| medios(format!("falta {n}")));
    if let Some(fd) = &fd {
        elemento("fuente")?.set_property("fd", fd.as_raw_fd());
    }
    // mpegtsmux reads the pads with gst_structure_get_int (unsigned fails and posts an ERROR that
    // stops the stream: black screen on the TV) and the PMT with get_uint (signed lands elsewhere).
    let programa = gst::Structure::builder("program_map")
        .field(format!("sink_{PID_VIDEO}"), 1i32)
        .field(format!("sink_{PID_AUDIO}"), 1i32)
        .field("PMT_1", PID_PMT)
        .build();
    elemento("mux")?.set_property("prog-map", &programa);

    let medidas = Medidas::instalar(&tuberia).ok_or_else(|| medios("sin puntos de medida"))?;
    let (tx, aviso) = watch::channel(None);
    let runtime = tokio::runtime::Handle::try_current().ok();
    tuberia.bus().ok_or_else(|| medios("sin bus"))?.set_sync_handler(move |_, msg| {
        match msg.view() {
            gst::MessageView::Error(e) => {
                tx.send_if_modified(|v| v.is_none().then(|| *v = Some(clasificar(e))).is_some());
            }
            gst::MessageView::Warning(w) => {
                let origen = w.src().map(|s| s.path_string().to_string());
                crate::registro::anotar(
                    "gst_aviso",
                    serde_json::json!({"origen": origen, "texto": w.error().to_string()}),
                );
            }
            // Posted from inside each new streaming thread: the moment to raise it.
            gst::MessageView::StreamStatus(s) if s.get().0 == gst::StreamStatusType::Enter => {
                prioridad::subir_este_hilo(&runtime);
            }
            _ => {}
        }
        gst::BusSyncReply::Drop
    });
    let ritmo = Arc::new(Ritmo::nuevo(perfil.kbps));
    let emisor = Emisor::instalar(&tuberia, destino, ritmo.clone(), cod)?;
    Ok(Montaje { tuberia, fd, medidas, aviso, ritmo, emisor })
}
