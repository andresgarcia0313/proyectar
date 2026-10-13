//! `Ventana`: what the window remembers between runs (its size) and shows in its footer
//! (the version). Kept in `ventana.json` beside the options; a damaged file means defaults.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, version)]
        #[qproperty(i32, ancho)]
        #[qproperty(i32, alto)]
        type Ventana = super::VentanaRust;

        /// Saves the size the user left the window at.
        #[qinvokable]
        fn recordar(self: Pin<&mut Ventana>, ancho: i32, alto: i32);
    }
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Tamano {
    ancho: i32,
    alto: i32,
}

pub struct VentanaRust {
    version: QString,
    ancho: i32,
    alto: i32,
}

impl Default for VentanaRust {
    fn default() -> Self {
        let t = leer().unwrap_or(Tamano { ancho: 380, alto: 640 });
        Self { version: QString::from(env!("CARGO_PKG_VERSION")), ancho: t.ancho, alto: t.alto }
    }
}

impl qobject::Ventana {
    pub fn recordar(mut self: Pin<&mut Self>, ancho: i32, alto: i32) {
        if (ancho, alto) == (self.rust().ancho, self.rust().alto) || ancho < 200 || alto < 200 {
            return;
        }
        self.as_mut().set_ancho(ancho);
        self.as_mut().set_alto(alto);
        escribir(Tamano { ancho, alto });
    }
}

fn ruta() -> PathBuf {
    std::env::var_os("PROYECTAR_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::config_dir().map(|d| d.join("proyectar")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ventana.json")
}

fn leer() -> Option<Tamano> {
    serde_json::from_str(&std::fs::read_to_string(ruta()).ok()?).ok()
}

/// Beside the file and renamed, so a crash never leaves half a file.
fn escribir(t: Tamano) {
    let (destino, parcial) = (ruta(), ruta().with_extension("tmp"));
    if let (Some(padre), Ok(texto)) = (destino.parent(), serde_json::to_string(&t)) {
        let _ = std::fs::create_dir_all(padre);
        if std::fs::write(&parcial, texto).is_ok() {
            let _ = std::fs::rename(&parcial, &destino);
        }
    }
}
