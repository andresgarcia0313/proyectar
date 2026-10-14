//! What a running pipeline tells: live numbers, whether the CPU encoder falls behind, the RTCP port.

use crate::medios::{Codificador, Medios};
use crate::medios_ritmo::aplicar;
use crate::Estadisticas;
use gstreamer::prelude::*;

impl Medios {
    /// The CPU encoder (Juegos) is below 80 % of the frame rate on a busy computer.
    pub fn rezagado(&self, fps: f32) -> bool {
        self.codificador == Codificador::Refresco && fps < self.modo.fps as f32 * 0.8
    }

    /// Called every second; also lets the bitrate climb back after a calm spell.
    pub fn estadisticas(&self) -> Estadisticas {
        if let (Some(kbps), Some(cod)) = (self.ritmo.calma(), self.pipeline.by_name("cod")) {
            aplicar(&cod, self.codificador, kbps);
        }
        let ((fps, kbps), (retardo_ms, retardo_peor_ms, retardo_cod_ms)) =
            (self.medidas.contador.tasa(), self.medidas.retardos());
        let (ancho, alto, gpu) = (self.modo.width, self.modo.height, self.codificador.es_gpu());
        let crate::rtcp::Informe { perdida_pct, perdidos, jitter_ms, rtt_ms, .. } = self.emisor.calidad();
        let ((tasa_descartes, tasa_duplicados), descartes) =
            (self.medidas.descartes.tasa(), self.medidas.descartes.cola());
        let (audio_medido_s, audio_senal_s) = self.medidas.nivel.segundos();
        Estadisticas {
            ancho,
            alto,
            fps,
            kbps,
            gpu,
            retardo_ms,
            retardo_peor_ms,
            retardo_cod_ms,
            perdida_pct,
            perdidos,
            jitter_ms,
            rtt_ms,
            descartes,
            tasa_descartes,
            tasa_duplicados,
            audio_medido_s,
            audio_senal_s,
        }
    }

    /// Where the TV's RTCP reports are read (`envio::PUERTO_RTCP` unless it was taken).
    pub fn puerto_rtcp(&self) -> u16 {
        self.emisor.puerto_rtcp()
    }
}
