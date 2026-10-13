//! What `Proyeccion` does with engine reports and QML commands.

use crate::{ajustes, avisos, proyeccion_object::qobject};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use motor::{Aviso, Estado, Fuente, Latencia, Opciones};

impl qobject::Proyeccion {
    pub(crate) fn opciones(&self) -> Opciones {
        Opciones {
            latencia: [Latencia::Juegos, Latencia::Trabajo, Latencia::Videos]
                .get(self.latencia as usize)
                .copied()
                .unwrap_or_default(),
            fuente: [Fuente::Duplicar, Fuente::Extender, Fuente::Ventana]
                .get(self.fuente as usize)
                .copied()
                .unwrap_or_default(),
            audio: self.audio,
        }
    }

    pub(crate) fn recibir(mut self: Pin<&mut Self>, aviso: Aviso) {
        match aviso {
            Aviso::Pantallas(lista) => self.as_mut().rust_mut().cercanas = lista,
            Aviso::Recientes(lista) => self.as_mut().rust_mut().recientes = lista,
            Aviso::Estado(estado) => return self.estado(&estado),
            Aviso::Estadisticas(e) => {
                self.as_mut().set_hay_estadisticas(e.ancho > 0);
                self.as_mut().set_resolucion(QString::from(format!("{}×{}", e.ancho, e.alto).as_str()));
                self.as_mut().set_fps(f64::from(e.fps));
                self.as_mut().set_kbps(i32::try_from(e.kbps).unwrap_or(i32::MAX));
                self.as_mut().set_retardo(f64::from(e.retardo_ms));
                return self.set_gpu(e.gpu);
            }
            Aviso::Diagnostico(d) => return self.mostrar_diagnostico(&d),
            Aviso::Error(mensaje) => return self.set_ultimo_error(QString::from(mensaje.as_str())),
        }
        let json = avisos::lista_json(&self.cercanas, &self.recientes);
        self.set_pantallas(QString::from(json.as_str()));
    }

    fn estado(mut self: Pin<&mut Self>, estado: &Estado) {
        self.as_mut().set_codigo(avisos::codigo(estado));
        self.as_mut().set_texto(QString::from(avisos::texto(estado).as_str()));
        self.as_mut().set_nombre(QString::from(avisos::nombre(estado).as_str()));
        if matches!(estado, Estado::Proyectando { .. }) {
            self.as_mut().set_diagnostico(QString::default());
        } else {
            self.as_mut().set_hay_estadisticas(false);
        }
    }

    pub fn buscar(&self) {
        if let Some(motor) = &self.rust().motor {
            motor.buscar();
        }
    }

    pub fn desconectar(&self) {
        if let Some(motor) = &self.rust().motor {
            motor.desconectar();
        }
    }

    pub fn elegir_latencia(mut self: Pin<&mut Self>, valor: i32) {
        self.as_mut().set_latencia(valor);
        self.aplicar();
    }

    pub fn elegir_fuente(mut self: Pin<&mut Self>, valor: i32) {
        self.as_mut().set_fuente(valor);
        self.aplicar();
    }

    pub fn elegir_audio(mut self: Pin<&mut Self>, valor: bool) {
        self.as_mut().set_audio(valor);
        self.aplicar();
    }

    pub fn descartar_error(self: Pin<&mut Self>) {
        self.set_ultimo_error(QString::default());
    }

    pub fn beat(&self) {
        crate::supervisor::beat();
    }

    /// Saves the choice and applies it to the projection in progress.
    fn aplicar(self: Pin<&mut Self>) {
        let opciones = self.opciones();
        ajustes::guardar(&opciones);
        if let Some(motor) = &self.rust().motor {
            motor.cambiar(opciones);
        }
    }
}
