//! Manual check of KWin's direct capture: mirrors the main screen for 3 s into a fakesink and
//! counts frames. Never creates a virtual screen and never opens a dialog. KWin answers only
//! programs whose `.desktop` declares the protocol (see `protocolos/LEEME.md`); otherwise it
//! exits with code 2.
//!
//! `cargo run -p motor --example kwin_captura`

use gstreamer as gst;
use gstreamer::prelude::*;
use motor::captura::Captura;
use motor::Fuente;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let pedido = Captura::directa(Fuente::Duplicar, "prueba", (1280, 720)).await;
    let Some(captura) = pedido.expect("KWin falló") else {
        eprintln!("KWin no ofrece zkde_screencast_unstable_v1 a este programa");
        std::process::exit(2);
    };
    println!("nodo {} tamaño {:?} portal {}", captura.nodo, captura.tamano, captura.es_portal());
    gst::init().expect("GStreamer");
    // The same source the projection uses (medios_cadena.rs).
    let texto = format!(
        "pipewiresrc path={} do-timestamp=true always-copy=true keepalive-time=33 max-buffers=64 \
         resend-last=true stream-properties=\"props,media.type=Video\" ! fakesink name=s",
        captura.nodo
    );
    let tuberia = gst::parse::launch(&texto).expect("tubería");
    let tuberia = tuberia.downcast::<gst::Pipeline>().expect("es tubería");
    let cuadros = Arc::new(AtomicU32::new(0));
    let cuenta = cuadros.clone();
    let entrada = tuberia.by_name("s").and_then(|s| s.static_pad("sink")).expect("fakesink");
    entrada.add_probe(gst::PadProbeType::BUFFER, move |_, _| {
        cuenta.fetch_add(1, Ordering::Relaxed);
        gst::PadProbeReturn::Ok
    });
    if tuberia.set_state(gst::State::Playing).is_err() {
        let bus = tuberia.bus().expect("bus");
        let error = bus.timed_pop_filtered(gst::ClockTime::from_seconds(1), &[gst::MessageType::Error]);
        eprintln!("GStreamer no arrancó: {error:?}");
        std::process::exit(3);
    }
    tokio::time::sleep(Duration::from_secs(3)).await;
    let _ = tuberia.set_state(gst::State::Null);
    println!("cuadros en 3 s: {}", cuadros.load(Ordering::Relaxed));
    captura.cerrar().await;
}
