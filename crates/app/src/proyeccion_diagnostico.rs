//! Each attempt to connect, what went wrong (the `Diagnostico` card) and how to try again.

use crate::proyeccion_object::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use motor::Diagnostico;
use std::path::Path;

/// The JSON QML parses: `{causa, titulo, detalle, pasos}`.
pub fn json(diagnostico: &Diagnostico) -> String {
    serde_json::to_string(diagnostico).unwrap_or_default()
}

/// Folder that holds the log file, as text for `Qt.openUrlExternally`.
pub fn carpeta(ruta: &Path) -> String {
    ruta.parent().unwrap_or(ruta).to_string_lossy().into_owned()
}

impl qobject::Proyeccion {
    /// A new attempt starts clean: the previous failure no longer applies.
    pub fn conectar(mut self: Pin<&mut Self>, mac: &QString) {
        self.as_mut().rust_mut().ultima_mac = mac.to_string();
        self.as_mut().set_ultimo_error(QString::default());
        self.as_mut().set_diagnostico(QString::default());
        if let Some(motor) = &self.rust().motor {
            motor.conectar(mac.to_string(), self.opciones());
        }
    }

    /// A diagnosis says more than the bare error, so it takes the error's place.
    pub(crate) fn mostrar_diagnostico(mut self: Pin<&mut Self>, diagnostico: &Diagnostico) {
        self.as_mut().set_ultimo_error(QString::default());
        self.set_diagnostico(QString::from(json(diagnostico).as_str()));
    }

    pub fn descartar_diagnostico(self: Pin<&mut Self>) {
        self.set_diagnostico(QString::default());
    }

    /// The last display tried, or else the most recent one.
    pub fn reintentar(self: Pin<&mut Self>) {
        let mac = Some(self.ultima_mac.clone())
            .filter(|m| !m.is_empty())
            .or_else(|| self.recientes.first().map(|p| p.mac.clone()));
        match mac {
            Some(mac) => self.conectar(&QString::from(mac.as_str())),
            None => self.descartar_diagnostico(),
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use motor::{Causa, Fallo};
    use serde_json::Value;

    #[test]
    fn json_con_titulo_detalle_y_pasos() {
        let d = Diagnostico::de(&Fallo::SinRespuesta, Causa::SinRespuestaTv);
        let v: Value = serde_json::from_str(&json(&d)).unwrap();
        assert_eq!(v["causa"], "SinRespuestaTv");
        assert_eq!(v["titulo"], "El televisor no respondió a la invitación");
        assert!(v["detalle"].is_string());
        assert_eq!(v["pasos"].as_array().unwrap().len(), d.pasos.len());
    }

    #[test]
    fn carpeta_del_registro() {
        let ruta = Path::new("/home/x/.local/state/proyectar/eventos.jsonl");
        assert_eq!(carpeta(ruta), "/home/x/.local/state/proyectar");
    }
}
