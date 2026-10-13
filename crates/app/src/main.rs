//! Proyectar: wireless display (Miracast) for the desktop. The motor streams; QML draws.
// No console window behind the app on Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod ajustes;
mod avisos;
mod demo;
mod inicio;
mod proyeccion_diagnostico;
mod proyeccion_logica;
mod proyeccion_object;
mod supervisor;

use std::ffi::{c_char, c_int, CString};

// src/app_shim.cpp: QApplication, Basic style and the QML engine.
unsafe extern "C" {
    fn proyectar_app_new(argc: c_int, argv: *mut *mut c_char);
    fn proyectar_app_run() -> c_int;
}

fn main() {
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
