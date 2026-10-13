//! What KWin has told us so far: screens, their order and how the stream went.

use std::time::{Duration, Instant};
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::EventQueue;

/// KWin answers in milliseconds; past this the stream is not coming.
const ESPERA: Duration = Duration::from_secs(5);

/// A screen: its connector name (`eDP-1`) and current mode, once KWin sends them.
pub struct Salida {
    pub objeto: WlOutput,
    pub nombre: Option<String>,
    pub modo: Option<(i32, i32)>,
}

#[derive(Default)]
pub struct Estado {
    pub salidas: Vec<Salida>,
    /// Connector names, main screen first (`kde_output_order_v1`).
    pub orden: Vec<String>,
    pub(super) nuevo_orden: Vec<String>,
    pub nodo: Option<u32>,
    pub fallo: Option<String>,
    pub cerrado: bool,
}

impl Estado {
    /// The main screen: first in KDE's order, else the first one announced.
    pub fn principal(&self) -> Option<&Salida> {
        let nombres: Vec<_> = self.salidas.iter().map(|s| s.nombre.as_deref()).collect();
        elegir(&nombres, &self.orden).map(|i| &self.salidas[i])
    }
}

/// Index of the main screen among `nombres`, as announced, given KDE's `orden`.
pub fn elegir(nombres: &[Option<&str>], orden: &[String]) -> Option<usize> {
    let primero = orden.first().map(String::as_str);
    let por_orden = nombres.iter().position(|n| n.is_some() && *n == primero);
    por_orden.or((!nombres.is_empty()).then_some(0))
}

/// The node comes once KWin has set up the PipeWire stream, a few round trips later.
pub fn esperar_nodo(cola: &mut EventQueue<Estado>, estado: &mut Estado) -> Result<u32, String> {
    let inicio = Instant::now();
    loop {
        cola.roundtrip(estado).map_err(texto)?;
        if let Some(nodo) = estado.nodo {
            return Ok(nodo);
        }
        if let Some(error) = estado.fallo.take() {
            return Err(format!("KWin rechazó la captura: {error}"));
        }
        if estado.cerrado || inicio.elapsed() > ESPERA {
            return Err("KWin no entregó la imagen".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn texto(e: impl std::fmt::Display) -> String {
    e.to_string()
}
