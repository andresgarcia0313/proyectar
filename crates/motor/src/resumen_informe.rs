//! Closes a session: turns what `Acumulador` gathered into `Resumen`, judges it and logs it as
//! `resumen_sesion`. The stopwatch delay (`latencia_medida`, logged by the interface) is read
//! back from the log, since only the person in front of the TV can measure it.

use crate::registro;
use crate::resumen::Acumulador;
use crate::resumen_serie::r1;
use crate::resumen_tipos::Resumen;
use crate::resumen_veredictos::veredictos;
use serde_json::Value;

impl Acumulador {
    pub fn resumen(&mut self, motivo: &'static str) -> Resumen {
        self.cerrar_tramo();
        let (m, c, o) = (&self.m, &self.cerrado, self.opciones.as_ref());
        let mut r = Resumen {
            pantalla: self.pantalla.clone(),
            modo: o.map(|o| o.latencia),
            fuente: o.map(|o| o.fuente),
            captura: self.captura,
            codificadores: self.codificadores.clone(),
            motivo,
            duracion_s: r1(self.inicio.map(|i| i.elapsed().as_secs_f32())).unwrap_or_default(),
            transmision_s: self.transmision_s,
            fps_objetivo: self.fps_objetivo,
            fps_media: r1(m.fps.media()),
            fps_p5: r1(m.fps.percentil(0.05)),
            kbps_media: m.kbps.media().map(f32::round),
            retardo_equipo_media_ms: r1(m.retardo.media()),
            retardo_equipo_p95_ms: r1(m.retardo.percentil(0.95)),
            perdida_media_pct: r1(m.perdida.media()),
            perdida_max_pct: r1(m.perdida.max()),
            jitter_media_ms: r1(m.jitter.media()),
            rtt_media_ms: r1(m.rtt.media()),
            idr_pedidos: self.idr,
            descartes: c.descartes,
            tasa_descartes: c.tasa_descartes,
            tasa_duplicados: c.tasa_duplicados,
            reconexiones: self.reconexiones,
            recuperadas: self.recuperaciones.largo() as u32,
            recuperacion_media_s: r1(self.recuperaciones.media()),
            recuperacion_max_s: r1(self.recuperaciones.max()),
            senal_min_dbm: m.senal.min(),
            senal_media_dbm: r1(m.senal.media()),
            tx_mbps_media: r1(m.tx.media()),
            reintentos_delta: (m.tx.largo() > 0).then_some(self.reintentos),
            cpu_media_pct: r1(m.cpu.media()),
            carga_media: r1(m.carga.media()),
            psi_cpu_media_pct: r1(m.psi.media()),
            audio_tv: o.map(|o| o.audio),
            audio_medido_s: c.audio_medido_s,
            audio_senal_s: c.audio_senal_s,
            audio_codec: self.audio_codec.clone(),
            audio_restaurada: self.audio_restaurada,
            salida_virtual_ok: self.salida_virtual,
            salida_virtual_quitada: self.salida_quitada,
            ahorro_apagado: self.ahorro.map(|a| !a),
            latencia_medida_ms: latencia_medida(&self.hora_inicio),
            veredictos: vec![],
        };
        r.veredictos = veredictos(&r);
        r
    }

    /// Logs `resumen_sesion`; called once per session, whatever ended it.
    pub fn cerrar(&mut self, motivo: &'static str) {
        let r = self.resumen(motivo);
        registro::anotar("resumen_sesion", serde_json::to_value(r).unwrap_or_default());
    }
}

/// Local time as the log writes it, so times can be compared as text.
pub fn hora() -> String {
    let ahora = gstreamer::glib::DateTime::now_local().ok();
    ahora
        .and_then(|h| Some(format!("{}.{:03}", h.format("%F %T").ok()?, h.microsecond() / 1000)))
        .unwrap_or_default()
}

/// The last stopwatch delay logged since `desde`, in ms.
pub fn latencia_medida(desde: &str) -> Option<f32> {
    let texto = std::fs::read_to_string(registro::ruta()).ok()?;
    ultima_latencia(&texto, desde)
}

pub fn ultima_latencia(texto: &str, desde: &str) -> Option<f32> {
    texto
        .lines()
        .rev()
        .filter(|l| l.contains("\"latencia_medida\""))
        .find_map(|l| {
            let v: Value = serde_json::from_str(l).ok()?;
            let hora = v["hora"].as_str()?;
            (v["evento"] == "latencia_medida" && hora >= desde).then(|| v["ms"].as_f64()).flatten()
        })
        .map(|ms| ms as f32)
}
