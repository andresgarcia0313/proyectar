//! `Proyeccion`: the QML face of `motor::Motor`. The engine reports from its own threads;
//! every `Aviso` is queued to the Qt thread, where it becomes properties QML binds to.

use cxx_qt_lib::QString;
use motor::{Motor, Pantalla};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, codigo)]
        #[qproperty(QString, texto)]
        #[qproperty(QString, nombre)]
        #[qproperty(QString, pantallas)]
        #[qproperty(bool, hay_estadisticas, cxx_name = "hayEstadisticas")]
        #[qproperty(QString, resolucion)]
        #[qproperty(f64, fps)]
        #[qproperty(i32, kbps)]
        #[qproperty(bool, gpu)]
        /// Delay inside this laptop, capture to network, in ms.
        #[qproperty(f64, retardo)]
        #[qproperty(QString, ultimo_error, cxx_name = "ultimoError")]
        #[qproperty(i32, latencia)]
        #[qproperty(i32, fuente)]
        #[qproperty(bool, audio)]
        /// JSON of the last `motor::Diagnostico`, empty when there is none.
        #[qproperty(QString, diagnostico)]
        /// Folder holding the event log, for "Ver registro".
        #[qproperty(QString, ruta_registro, cxx_name = "rutaRegistro")]
        type Proyeccion = super::ProyeccionRust;

        #[qinvokable]
        fn buscar(self: &Proyeccion);
        #[qinvokable]
        fn conectar(self: Pin<&mut Proyeccion>, mac: &QString);
        /// Connects again to the last display tried.
        #[qinvokable]
        fn reintentar(self: Pin<&mut Proyeccion>);
        #[qinvokable]
        fn desconectar(self: &Proyeccion);
        #[qinvokable]
        #[cxx_name = "elegirLatencia"]
        fn elegir_latencia(self: Pin<&mut Proyeccion>, valor: i32);
        #[qinvokable]
        #[cxx_name = "elegirFuente"]
        fn elegir_fuente(self: Pin<&mut Proyeccion>, valor: i32);
        #[qinvokable]
        #[cxx_name = "elegirAudio"]
        fn elegir_audio(self: Pin<&mut Proyeccion>, valor: bool);
        #[qinvokable]
        #[cxx_name = "descartarError"]
        fn descartar_error(self: Pin<&mut Proyeccion>);
        #[qinvokable]
        #[cxx_name = "descartarDiagnostico"]
        fn descartar_diagnostico(self: Pin<&mut Proyeccion>);
        /// The event loop is alive; the supervisor restarts the app when beats stop.
        #[qinvokable]
        fn beat(self: &Proyeccion);
    }

    impl cxx_qt::Threading for Proyeccion {}
    impl cxx_qt::Initialize for Proyeccion {}
}

#[derive(Default)]
pub struct ProyeccionRust {
    pub(crate) codigo: i32,
    pub(crate) texto: QString,
    pub(crate) nombre: QString,
    pub(crate) pantallas: QString,
    pub(crate) hay_estadisticas: bool,
    pub(crate) resolucion: QString,
    pub(crate) fps: f64,
    pub(crate) kbps: i32,
    pub(crate) gpu: bool,
    pub(crate) retardo: f64,
    pub(crate) ultimo_error: QString,
    pub(crate) latencia: i32,
    pub(crate) fuente: i32,
    pub(crate) audio: bool,
    pub(crate) diagnostico: QString,
    pub(crate) ruta_registro: QString,
    pub(crate) ultima_mac: String,
    pub(crate) motor: Option<Motor>,
    pub(crate) cercanas: Vec<Pantalla>,
    pub(crate) recientes: Vec<Pantalla>,
}
