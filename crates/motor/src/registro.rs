//! Event log: one JSON object per line in `~/.local/state/proyectar/eventos.jsonl`, so a failure
//! can be traced step by step afterwards (and fed to `jq`). It rotates at 4 MB and never fails:
//! a log that cannot be written must not stop a projection.

use serde_json::{json, Value};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

const TOPE: u64 = 4 * 1024 * 1024;

static ARCHIVO: Mutex<Option<File>> = Mutex::new(None);

/// Where the log lives; `PROYECTAR_REGISTRO` points it elsewhere (tests, bug reports).
pub fn ruta() -> PathBuf {
    std::env::var_os("PROYECTAR_REGISTRO").map(PathBuf::from).unwrap_or_else(|| {
        dirs::state_dir().unwrap_or_else(std::env::temp_dir).join("proyectar").join("eventos.jsonl")
    })
}

/// Appends `{"hora", "evento", ...datos}`. `datos` should be an object.
pub fn anotar(evento: &str, datos: Value) {
    let hora = gstreamer::glib::DateTime::now_local()
        .ok()
        .and_then(|h| Some(format!("{}.{:03}", h.format("%F %T").ok()?, h.microsecond() / 1000)))
        .unwrap_or_default();
    let mut linea = json!({"hora": hora, "evento": evento});
    if let (Some(destino), Value::Object(campos)) = (linea.as_object_mut(), datos) {
        destino.extend(campos);
    }
    let mut guardado = ARCHIVO.lock().unwrap_or_else(|e| e.into_inner());
    if guardado.as_ref().and_then(|f| f.metadata().ok()).is_some_and(|m| m.len() > TOPE) {
        let _ = std::fs::rename(ruta(), ruta().with_extension("1.jsonl"));
        *guardado = None;
    }
    if guardado.is_none() {
        *guardado = abrir();
    }
    if let Some(archivo) = guardado.as_mut() {
        if writeln!(archivo, "{linea}").is_err() {
            *guardado = None;
        }
    }
}

/// Wraps the interface's callback so every report is logged too; logs the start.
pub fn envolver(avisar: impl Fn(crate::Aviso) + Send + Sync + 'static) -> crate::Avisar {
    let codificador = crate::medios::Codificador::primero().nombre();
    let equipo = crate::enlace::nombre_propio();
    anotar(
        "inicio",
        json!({"version": env!("CARGO_PKG_VERSION"), "equipo": equipo, "codificador": codificador}),
    );
    std::sync::Arc::new(move |a| {
        aviso(&a);
        avisar(a)
    })
}

/// What the engine told the interface; numbers and lists go in their own events.
pub fn aviso(aviso: &crate::Aviso) {
    use crate::Aviso::*;
    match aviso {
        Estado(e) => anotar("estado", json!({"estado": format!("{e:?}")})),
        Error(texto) => anotar("error", json!({"texto": texto})),
        Diagnostico(d) => anotar("diagnostico", json!({"diagnostico": d})),
        Pantallas(l) => {
            anotar("pantallas", json!({"vistas": l.iter().map(|p| (&p.nombre, p.senal)).collect::<Vec<_>>()}))
        }
        Recientes(_) | Estadisticas(_) => {}
    }
}

fn abrir() -> Option<File> {
    let ruta = ruta();
    std::fs::create_dir_all(ruta.parent()?).ok()?;
    OpenOptions::new().create(true).append(true).open(ruta).ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn escribe_una_linea_json_por_evento() {
        let dir = std::env::temp_dir().join(format!("proyectar-registro-{}", std::process::id()));
        std::env::set_var("PROYECTAR_REGISTRO", dir.join("e.jsonl"));
        super::anotar("prueba", serde_json::json!({"n": 1}));
        let texto = std::fs::read_to_string(dir.join("e.jsonl")).unwrap();
        let linea: serde_json::Value = serde_json::from_str(texto.lines().last().unwrap()).unwrap();
        assert_eq!(linea["evento"], "prueba");
        assert_eq!(linea["n"], 1);
        let _ = std::fs::remove_dir_all(dir);
    }
}
