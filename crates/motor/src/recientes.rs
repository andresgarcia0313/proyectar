//! Displays used before, newest first, and the portal tokens that skip its dialog.
//! Kept in `~/.config/proyectar/recientes.json`; a damaged file just starts empty.

use crate::modo::{Fuente, Opciones};
use crate::Pantalla;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAXIMO: usize = 5;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Recientes {
    pub pantallas: Vec<Pantalla>,
    pub opciones: Opciones,
    /// Portal restore tokens, one per source kind.
    pub tokens: Vec<(Fuente, String)>,
}

impl Recientes {
    pub fn cargar() -> Self {
        std::fs::read(ruta()).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
    }

    /// Writes atomically (temporary file and rename) so a crash never leaves half a file.
    pub fn guardar(&self) {
        let destino = ruta();
        let temporal = destino.with_extension("tmp");
        let escrito = destino
            .parent()
            .map(std::fs::create_dir_all)
            .transpose()
            .ok()
            .and_then(|_| serde_json::to_vec_pretty(self).ok())
            .and_then(|json| std::fs::write(&temporal, json).ok());
        if escrito.is_some() {
            let _ = std::fs::rename(&temporal, &destino);
        }
    }

    pub fn usar(&mut self, pantalla: &Pantalla) {
        self.pantallas.retain(|p| p.mac != pantalla.mac);
        self.pantallas.insert(0, pantalla.clone());
        self.pantallas.truncate(MAXIMO);
    }

    pub fn token(&self, fuente: Fuente) -> Option<&str> {
        self.tokens.iter().find(|(f, _)| *f == fuente).map(|(_, t)| t.as_str())
    }

    pub fn poner_token(&mut self, fuente: Fuente, token: Option<String>) {
        self.tokens.retain(|(f, _)| *f != fuente);
        if let Some(token) = token {
            self.tokens.push((fuente, token));
        }
    }
}

fn ruta() -> PathBuf {
    let base = std::env::var_os("PROYECTAR_CONFIG").map(PathBuf::from);
    base.or_else(|| dirs::config_dir().map(|d| d.join("proyectar")))
        .unwrap_or_else(|| PathBuf::from(".proyectar"))
        .join("recientes.json")
}
