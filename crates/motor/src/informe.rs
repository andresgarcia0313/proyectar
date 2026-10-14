//! `proyectar --informe [N]`: the last N projection sessions read back from the event log
//! (and its rotated copy), with their numbers, the verdict per manual test and the last
//! startup self-check. Reads only; never opens a window.

use crate::informe_sesion::agrupar;
use crate::informe_texto;
use serde_json::Value;
use std::fmt::Write;
use std::path::PathBuf;

pub const SESIONES: usize = 5;

/// The report for the log `registro::ruta()` points to.
pub fn texto(n: usize) -> String {
    let ruta = crate::registro::ruta();
    let eventos = leer(&[ruta.with_extension("1.jsonl"), ruta.clone()]);
    if eventos.is_empty() {
        return format!("No hay eventos en {}.\n", ruta.display());
    }
    armar(&eventos, n)
}

/// Every parsable line, oldest file first; broken lines (a crash mid-write) are skipped.
pub fn leer(rutas: &[PathBuf]) -> Vec<Value> {
    let textos = rutas.iter().filter_map(|r| std::fs::read_to_string(r).ok());
    textos.flat_map(|t| t.lines().filter_map(|l| serde_json::from_str(l).ok()).collect::<Vec<_>>()).collect()
}

pub fn armar(eventos: &[Value], n: usize) -> String {
    let sesiones = agrupar(eventos);
    let desde = sesiones.len().saturating_sub(n.max(1));
    let mut t = format!("Informe de Proyectar: {} de {} sesiones\n", sesiones.len() - desde, sesiones.len());
    if sesiones.is_empty() {
        t.push_str("No hay sesiones de proyección en el registro.\n");
    }
    for (i, sesion) in sesiones.iter().enumerate().skip(desde) {
        t += &informe_texto::sesion(i + 1, sesion);
    }
    t + &autodiagnostico(eventos.iter().rev().find(|e| e["evento"] == "autodiagnostico"))
}

fn autodiagnostico(evento: Option<&Value>) -> String {
    let Some(checks) = evento.and_then(|e| e["checks"].as_array()) else {
        return "\n== Autodiagnóstico ==\nNo hay autodiagnóstico registrado: abra Proyectar una vez.\n"
            .into();
    };
    let hora = evento.and_then(|e| e["hora"].as_str()).unwrap_or("hora desconocida");
    let mut t = format!("\n== Autodiagnóstico ({hora}) ==\n");
    for c in checks {
        let ok = match c["ok"].as_bool() {
            Some(true) => "bien",
            Some(false) => "MAL",
            None => "sin dato",
        };
        let _ = writeln!(
            t,
            "{:<22} {:<9} {}",
            informe_texto::texto(&c["nombre"]),
            ok,
            informe_texto::texto(&c["detalle"])
        );
    }
    t
}

#[cfg(test)]
#[path = "informe_pruebas.rs"]
mod pruebas;
