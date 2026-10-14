//! What the pipeline's bus says, handled on the thread that posts it: the first error ends the
//! session, warnings go to the log, `level` feeds the audio measure, and each new streaming
//! thread gets its priority raised.

use crate::medios_estado::{clasificar, Aviso};
use crate::medios_nivel::Nivel;
use crate::{prioridad, registro};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::Arc;
use tokio::sync::watch;

pub fn vigilar(bus: &gst::Bus, tx: watch::Sender<Aviso>, nivel: Arc<Nivel>) {
    let runtime = tokio::runtime::Handle::try_current().ok();
    bus.set_sync_handler(move |_, msg| {
        match msg.view() {
            gst::MessageView::Error(e) => {
                tx.send_if_modified(|v| v.is_none().then(|| *v = Some(clasificar(e))).is_some());
            }
            gst::MessageView::Warning(w) => {
                let origen = w.src().map(|s| s.path_string().to_string());
                registro::anotar(
                    "gst_aviso",
                    serde_json::json!({"origen": origen, "texto": w.error().to_string()}),
                );
            }
            gst::MessageView::Element(e) => {
                if let Some(s) = e.structure() {
                    nivel.mensaje(s);
                }
            }
            // Posted from inside each new streaming thread: the moment to raise it.
            gst::MessageView::StreamStatus(s) if s.get().0 == gst::StreamStatusType::Enter => {
                prioridad::subir_este_hilo(&runtime);
            }
            _ => {}
        }
        gst::BusSyncReply::Drop
    });
}
