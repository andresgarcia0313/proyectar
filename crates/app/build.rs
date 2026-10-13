use cxx_qt_build::{CxxQtBuilder, QResource, QResources, QmlModule};
use qt_build_utils::QResourceFile;
use std::{fs, path::Path};

fn main() {
    let interface = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../qml/Proyectar");
    let interface = interface.canonicalize().expect("qml/Proyectar exists");
    println!("cargo:rerun-if-changed={}", interface.display());

    CxxQtBuilder::new_qml_module(QmlModule::new("ProyectarApp").version(1, 0).qml_files(["qml/Main.qml"]))
        .files(["src/proyeccion_object.rs", "src/ventana.rs"])
        // Linux and macOS pull QtGui in through QtQuick; the MSVC linker needs it named.
        .qt_module("Gui")
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .qt_module("Widgets")
        .cpp_files(["src/app_shim.cpp"])
        .qrc_resources(
            QResources::new()
                .resource(QResource::new().files(["assets/proyectar.svg"]))
                .resource(interface_module(&interface)),
        )
        .build();
}

/// The hand-written `Proyectar` module, served from `qrc:/qt/qml/Proyectar`;
/// `app_shim.cpp` puts that folder on the import path.
fn interface_module(dir: &Path) -> QResource {
    let mut files = Vec::new();
    for sub in ["", "icons"] {
        for entry in fs::read_dir(dir.join(sub)).expect("readable interface folder") {
            let path = entry.expect("readable entry").path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let wanted = name == "qmldir" || name.ends_with(".qml") || name.ends_with(".svg");
            if path.is_file() && wanted {
                let alias = if sub.is_empty() { name } else { format!("{sub}/{name}") };
                files.push(QResourceFile::new(&path).alias(alias));
            }
        }
    }
    QResource::new().prefix("/qt/qml/Proyectar").files(files)
}
