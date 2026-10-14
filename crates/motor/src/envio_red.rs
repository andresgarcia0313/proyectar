//! The pieces `envio` wires together: the marked UDP sockets, the appsink hook that queues each
//! RTP packet, and the bus error that turns a dead link into `Fallo::Medios`.

use crate::registro;
use gstreamer as gst;
use gstreamer::prelude::*;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

/// DSCP 46 (expedited forwarding): Wi-Fi WMM puts it in the voice queue, ahead of bulk traffic.
const TOS_EF: u32 = 0xB8;
const BUFER_ENVIO: usize = 1 << 20;

pub type Paquete = (Instant, gst::MappedBuffer<gst::buffer::Readable>);

/// A UDP socket marked EF on `puerto`; on any free port when that one is taken.
pub fn socket(puerto: u16) -> std::io::Result<UdpSocket> {
    let s = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    s.set_tos_v4(TOS_EF)?;
    s.set_send_buffer_size(BUFER_ENVIO)?;
    if let Err(e) = s.bind(&SocketAddr::from(([0, 0, 0, 0], puerto)).into()) {
        // Another projection still holds it: keep streaming, the reports just will not arrive.
        registro::anotar(
            "envio_puerto_ocupado",
            serde_json::json!({"puerto": puerto, "error": e.to_string()}),
        );
        s.bind(&SocketAddr::from(([0, 0, 0, 0], 0)).into())?;
    }
    Ok(s.into())
}

/// Queues every packet the appsink receives, stamped with the moment it arrived.
pub fn enganchar(salida: &gst::Element) -> Receiver<Paquete> {
    let (tx, rx) = mpsc::channel();
    salida.connect("new-sample", false, move |args| {
        let muestra = args[0]
            .get::<gst::Element>()
            .ok()
            .and_then(|s| s.emit_by_name::<Option<gst::Sample>>("pull-sample", &[]));
        let mapa = muestra.and_then(|m| m.buffer_owned()).and_then(|b| b.into_mapped_buffer_readable().ok());
        if let Some(paquete) = mapa {
            let _ = tx.send((Instant::now(), paquete));
        }
        Some(gst::FlowReturn::Ok.to_value())
    });
    rx
}

/// Posts the send failure on the bus as coming from `salida`, which `Medios::fallo` reads as
/// `Fallo::Medios`.
pub fn avisar_fallo(salida: gst::Element, destino: SocketAddr) -> impl Fn(std::io::Error) {
    move |e| {
        let texto = format!("envío UDP a {destino}: {e}");
        let error = gst::message::Error::builder(gst::ResourceError::Write, &texto).src(&salida).build();
        let _ = salida.post_message(error);
    }
}
