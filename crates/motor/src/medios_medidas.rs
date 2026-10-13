//! Everything measured on a running pipeline: rate, and delay up to the encoder and to the network.

use crate::medios_estado::Contador;
use crate::medios_latencia::Retardo;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::Arc;

/// Everything measured on a running pipeline: rate, and delay up to the encoder and to the network.
pub struct Medidas {
    pub contador: Arc<Contador>,
    codificado: Option<Arc<Retardo>>,
    enviado: Option<Arc<Retardo>>,
}

impl Medidas {
    pub fn instalar(tuberia: &gst::Pipeline) -> Option<Self> {
        let contador = Arc::new(Contador::nuevo());
        let c = contador.clone();
        tuberia.by_name("analizador")?.static_pad("src")?.add_probe(
            gst::PadProbeType::BUFFER,
            move |_, info| {
                if let Some(gst::PadProbeData::Buffer(b)) = &info.data {
                    c.sumar(b.size());
                }
                gst::PadProbeReturn::Ok
            },
        );
        let codificado = Retardo::vigilar(tuberia, "analizador", "src");
        Some(Self { contador, codificado, enviado: Retardo::vigilar(tuberia, "salida", "sink") })
    }

    /// (average to the network, worst to the network, average to the encoder output) in ms.
    pub fn retardos(&self) -> (f32, f32, f32) {
        let (total, peor) = self.enviado.as_ref().map(|r| r.tomar()).unwrap_or_default();
        (total, peor, self.codificado.as_ref().map(|r| r.tomar().0).unwrap_or_default())
    }
}
