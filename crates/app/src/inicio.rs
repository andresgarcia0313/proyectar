//! Start-up of `Proyeccion`: saved options, the event log folder, then the engine (or the demo).

use crate::proyeccion_diagnostico::carpeta;
use crate::{ajustes, avisos, demo, proyeccion_object::qobject};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use motor::{Aviso, Estado, Motor};

impl cxx_qt::Initialize for qobject::Proyeccion {
    fn initialize(mut self: Pin<&mut Self>) {
        let guardadas = ajustes::cargar();
        self.as_mut().set_latencia(guardadas.latencia as i32);
        self.as_mut().set_fuente(guardadas.fuente as i32);
        self.as_mut().set_audio(guardadas.audio);
        self.as_mut().set_texto(QString::from(avisos::texto(&Estado::Inactivo).as_str()));
        self.as_mut().set_ruta_registro(QString::from(carpeta(&motor::registro::ruta()).as_str()));
        let hilo = self.qt_thread();
        let enviar = move |aviso: Aviso| {
            let _ = hilo.queue(move |mut objeto| objeto.as_mut().recibir(aviso));
        };
        if let Some(escena) = std::env::var("PROYECTAR_DEMO").ok().filter(|s| !s.is_empty()) {
            demo::reproducir(&escena, enviar);
        } else {
            self.as_mut().rust_mut().motor = Some(Motor::iniciar(enviar));
        }
    }
}
