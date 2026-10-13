//! The commands behind the TV's audio output: `pactl` loads and unloads the null sink,
//! `pw-metadata` reads and writes the output the user chose (the *configured* default, so an
//! absent headset stays preferred) and a small file remembers it in case the app dies.

use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

/// PipeWire node name of the TV's output; `medios_audio` captures its monitor.
pub const SUMIDERO: &str = "proyectar_tv";
const CLAVE: &str = "default.configured.audio.sink";

/// Runs a sound tool with the C locale (es_CO translates `pactl` output) and returns stdout.
pub fn correr(programa: &str, args: &[&str]) -> Result<String, String> {
    let salida = Command::new(programa)
        .args(args)
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| format!("{programa}: {e}"))?;
    if salida.status.success() {
        return Ok(String::from_utf8_lossy(&salida.stdout).trim().to_owned());
    }
    let error = String::from_utf8_lossy(&salida.stderr);
    Err(format!("{programa} {}: {}", args.join(" "), error.trim()))
}

/// Null-sink arguments: "[TV] name" as description, a screen icon, 48 kHz stereo (the TV's
/// format, so nothing resamples) and the lowest priority so WirePlumber never picks it alone.
pub fn argumentos(nombre_tv: &str) -> String {
    let limpio: String = nombre_tv.chars().filter(|c| !matches!(c, '"' | '\'' | '\\')).collect();
    format!(
        "sink_name={SUMIDERO} rate=48000 channels=2 channel_map=front-left,front-right \
         sink_properties='device.description=\"[TV] {limpio}\" device.icon_name=video-display \
         priority.session=0 priority.driver=0'"
    )
}

/// Indexes of our sinks in `pactl list modules short` (one left behind by a crash, too).
pub fn modulos_propios(lista: &str) -> Vec<String> {
    let propio = format!("sink_name={SUMIDERO}");
    lista
        .lines()
        .filter(|l| l.contains("module-null-sink") && l.split_whitespace().any(|p| p == propio))
        .filter_map(|l| l.split_whitespace().next().map(str::to_owned))
        .collect()
}

/// The output the user chose, as WirePlumber stores it (`{"name":"..."}`), if any.
pub fn elegida() -> Result<Option<String>, String> {
    Ok(valor_metadato(&correr("pw-metadata", &["0", CLAVE])?))
}

pub fn valor_metadato(texto: &str) -> Option<String> {
    let linea = texto.lines().find(|l| l.contains(&format!("key:'{CLAVE}'")))?;
    Some(linea.split("value:'").nth(1)?.rsplit_once("' type:")?.0.to_owned())
}

/// Sets the chosen output; `None` clears it so WirePlumber falls back by priority.
pub fn elegir(valor: Option<&str>) -> Result<String, String> {
    match valor {
        Some(v) => correr("pw-metadata", &["0", CLAVE, v, "Spa:String:JSON"]),
        None => correr("pw-metadata", &["-d", "0", CLAVE]),
    }
}

pub fn valor_propio() -> String {
    json!({"name": SUMIDERO}).to_string()
}

pub fn es_propio(valor: Option<&str>) -> bool {
    valor.is_none_or(|v| v.contains(&format!("\"{SUMIDERO}\"")))
}

fn archivo() -> PathBuf {
    crate::registro::ruta().with_file_name("salida-previa.json")
}

pub fn recordar(previa: Option<&str>) {
    let _ = std::fs::write(archivo(), json!({"previa": previa}).to_string());
}

/// `Some(previa)` when a projection changed the default and has not restored it yet.
pub fn recordada() -> Option<Option<String>> {
    let texto = std::fs::read_to_string(archivo()).ok()?;
    let valor: Value = serde_json::from_str(&texto).ok()?;
    Some(valor.get("previa")?.as_str().map(str::to_owned))
}

pub fn olvidar() {
    let _ = std::fs::remove_file(archivo());
}

#[cfg(test)]
#[path = "salida_audio_pruebas.rs"]
mod pruebas;
