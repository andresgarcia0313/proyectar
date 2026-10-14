//! Proyectar: wireless display (Miracast) for the desktop. The motor streams; QML draws.
// No console window behind the app on Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod ajustes;
mod avisos;
mod demo;
mod inicio;
mod latencia;
mod proyeccion_diagnostico;
mod proyeccion_logica;
mod proyeccion_object;
mod supervisor;
mod ventana;

use std::ffi::{c_char, c_int, CString};

// src/app_shim.cpp: QApplication, Basic style and the QML engine.
unsafe extern "C" {
    fn proyectar_app_new(argc: c_int, argv: *mut *mut c_char);
    fn proyectar_app_run() -> c_int;
}

/// `proyectar --informe [N]`: the N sessions to report (5 by default), or `None` to run the app.
fn pedido_informe(args: &[String]) -> Option<usize> {
    match args {
        [opcion, resto @ ..] if opcion == "--informe" => {
            Some(resto.first().and_then(|n| n.parse().ok()).unwrap_or(motor::informe::SESIONES))
        }
        _ => None,
    }
}

fn main() {
    // Before the supervisor and Qt: the report only reads the event log and exits.
    if let Some(n) = pedido_informe(&std::env::args().skip(1).collect::<Vec<_>>()) {
        print!("{}", motor::informe::texto(n));
        return;
    }
    if let Some(code) = supervisor::run() {
        std::process::exit(code);
    }
    // Qt keeps pointers to argv for the whole run, so these live until main returns.
    let args: Vec<CString> = std::env::args().map(|a| CString::new(a).unwrap_or_default()).collect();
    let mut argv: Vec<*mut c_char> = args.iter().map(|a| a.as_ptr().cast_mut()).collect();
    argv.push(std::ptr::null_mut());
    let argc = c_int::try_from(args.len()).unwrap_or(1);

    // SAFETY: argv is null-terminated and outlives the application object.
    unsafe { proyectar_app_new(argc, argv.as_mut_ptr()) };
    cxx_qt::init_qml_module!("ProyectarApp");
    // SAFETY: called once, on the main thread, after proyectar_app_new.
    let code = unsafe { proyectar_app_run() };
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::pedido_informe;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn el_informe_se_pide_con_n_opcional() {
        assert_eq!(pedido_informe(&args(&["--informe", "3"])), Some(3));
        assert_eq!(pedido_informe(&args(&["--informe"])), Some(motor::informe::SESIONES));
        assert_eq!(pedido_informe(&args(&["--informe", "muchas"])), Some(motor::informe::SESIONES));
        assert_eq!(pedido_informe(&args(&[])), None);
        assert_eq!(pedido_informe(&args(&["--demo"])), None);
    }
}
