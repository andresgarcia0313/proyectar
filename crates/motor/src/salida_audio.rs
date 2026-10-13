//! The TV as an audio output in KDE while projecting, as Windows shows it: a PipeWire null
//! sink named after the TV. With "Audio en el televisor" on it becomes the default output and
//! `medios_audio` sends its monitor to the TV; with it off it only exists (the TV gets silence).
//! Dropping the guard restores the previous output and removes the sink; a sink left behind
//! by a crash is removed (and the output restored) before the next one is made.

use crate::registro::anotar;
use crate::salida_audio_cmd as cmd;
use serde_json::json;

pub struct SalidaTv {
    modulo: String,
    /// `Some(previa)` once this guard made itself the default output.
    previa: Option<Option<String>>,
}

impl SalidaTv {
    /// Never fails a projection: without the sink the TV still gets video (and silence).
    pub fn crear(nombre_tv: &str, por_defecto: bool) -> Option<SalidaTv> {
        limpiar_huerfano();
        let args = cmd::argumentos(nombre_tv);
        let modulo = match cmd::correr("pactl", &["load-module", "module-null-sink", &args]) {
            Ok(modulo) => modulo,
            Err(e) => return fallo("crear", e),
        };
        let datos = json!({"sumidero": cmd::SUMIDERO, "modulo": modulo, "nombre": nombre_tv, "por_defecto": por_defecto});
        anotar("salida_audio_creada", datos);
        let mut salida = SalidaTv { modulo, previa: None };
        if por_defecto {
            salida.previa = volverse_predeterminada();
        }
        Some(salida)
    }
}

impl Drop for SalidaTv {
    fn drop(&mut self) {
        if let Some(previa) = self.previa.take() {
            restaurar(previa);
        }
        quitar(&self.modulo);
    }
}

/// Removes sinks a crashed run left and gives back the output it had taken.
pub fn limpiar_huerfano() {
    let lista = cmd::correr("pactl", &["list", "modules", "short"]);
    let modulos = lista.map(|t| cmd::modulos_propios(&t)).unwrap_or_default();
    if let Some(previa) = cmd::recordada() {
        restaurar(previa);
    }
    if !modulos.is_empty() {
        anotar("salida_audio_huerfana", json!({"modulos": modulos}));
        modulos.iter().for_each(|m| quitar(m));
    }
}

fn volverse_predeterminada() -> Option<Option<String>> {
    let previa = cmd::elegida().map_err(|e| fallo::<()>("leer_previa", e)).ok()?;
    cmd::recordar(previa.as_deref());
    if let Err(e) = cmd::elegir(Some(&cmd::valor_propio())) {
        cmd::olvidar();
        return fallo("predeterminar", e);
    }
    anotar("salida_audio_previa", json!({"previa": previa}));
    Some(previa)
}

/// Puts back `previa` unless the user chose another output meanwhile.
fn restaurar(previa: Option<String>) {
    match cmd::elegida() {
        Ok(actual) if !cmd::es_propio(actual.as_deref()) => {
            anotar("salida_audio_respetada", json!({"actual": actual, "previa": previa}))
        }
        Ok(_) => match cmd::elegir(previa.as_deref()) {
            Ok(_) => anotar("salida_audio_restaurada", json!({"previa": previa})),
            Err(e) => return avisar("restaurar", e),
        },
        Err(e) => return avisar("restaurar", e),
    }
    cmd::olvidar();
}

fn quitar(modulo: &str) {
    match cmd::correr("pactl", &["unload-module", modulo]) {
        Ok(_) => anotar("salida_audio_quitada", json!({"modulo": modulo})),
        Err(e) => avisar("quitar", e),
    }
}

fn avisar(paso: &str, error: String) {
    anotar("salida_audio_error", json!({"paso": paso, "error": error}));
}

fn fallo<T>(paso: &str, error: String) -> Option<T> {
    avisar(paso, error);
    None
}
