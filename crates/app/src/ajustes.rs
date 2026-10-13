//! The options chosen in the panel survive restarts: one small JSON in the config folder.

use motor::Opciones;
use std::path::{Path, PathBuf};

/// `PROYECTAR_CONFIG_DIR` lets tests and demos keep their own folder.
fn carpeta() -> PathBuf {
    std::env::var_os("PROYECTAR_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::config_dir().map(|d| d.join("proyectar")))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn cargar() -> Opciones {
    leer(&carpeta().join("opciones.json"))
}

pub fn guardar(opciones: &Opciones) {
    escribir(&carpeta().join("opciones.json"), opciones);
}

fn leer(ruta: &Path) -> Opciones {
    std::fs::read_to_string(ruta).ok().and_then(|texto| serde_json::from_str(&texto).ok()).unwrap_or_default()
}

fn escribir(ruta: &Path, opciones: &Opciones) {
    if let Some(padre) = ruta.parent() {
        let _ = std::fs::create_dir_all(padre);
    }
    // Write beside the file and rename, so a crash never leaves half a file.
    let parcial = ruta.with_extension("tmp");
    if let Ok(texto) = serde_json::to_string_pretty(opciones) {
        if std::fs::write(&parcial, texto).is_ok() {
            let _ = std::fs::rename(&parcial, ruta);
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use motor::{Fuente, Latencia};

    #[test]
    fn guarda_y_recupera_las_opciones() {
        let ruta = std::env::temp_dir().join(format!("proyectar-{}/opciones.json", std::process::id()));
        let elegidas = Opciones { latencia: Latencia::Juegos, fuente: Fuente::Extender, audio: false };
        escribir(&ruta, &elegidas);
        assert_eq!(leer(&ruta), elegidas);
        let _ = std::fs::remove_dir_all(ruta.parent().unwrap());
    }

    #[test]
    fn archivo_ausente_o_danado_da_valores_por_defecto() {
        let ruta = std::env::temp_dir().join("proyectar-no-existe/opciones.json");
        assert_eq!(leer(&ruta), Opciones::default());
    }
}
