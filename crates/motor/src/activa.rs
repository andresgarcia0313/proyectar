//! The projection in course: how to change its options, stop it, and keep its panics
//! from reaching the engine.

use crate::busqueda::encontrar;
use crate::ciclo::Ciclo;
use crate::diagnostico::{Causa, Diagnostico};
use crate::recientes::Recientes;
use crate::wifi_direct::WifiDirect;
use crate::{Avisar, Aviso, Estado, Opciones};
use futures_util::FutureExt;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub struct Activa {
    pub opciones: watch::Sender<Opciones>,
    pub parar: watch::Sender<bool>,
    pub tarea: JoinHandle<()>,
}

impl Activa {
    /// Finds the display, remembers it among the recent ones and starts projecting to it.
    pub async fn lanzar(
        wd: Arc<WifiDirect>,
        mac: &str,
        opciones: Opciones,
        recientes: &Arc<Mutex<Recientes>>,
        avisar: &Avisar,
    ) -> Option<Activa> {
        let pantalla = match encontrar(&wd, mac).await {
            Ok(p) => p,
            Err(fallo) => {
                avisar(Aviso::Diagnostico(Diagnostico::de(&fallo, Causa::SinRespuestaTv)));
                avisar(Aviso::Error(format!("No se pudo conectar: {fallo}")));
                return None;
            }
        };
        wd.parar().await;
        guardar_opciones(recientes, &opciones);
        let mut lista = recientes.lock().unwrap_or_else(|e| e.into_inner());
        lista.usar(&pantalla);
        lista.guardar();
        avisar(Aviso::Recientes(lista.pantallas.clone()));
        drop(lista);
        let (tx_op, rx_op) = watch::channel(opciones);
        let (tx_parar, rx_parar) = watch::channel(false);
        let ciclo = Ciclo {
            wd,
            pantalla,
            opciones: rx_op,
            parar: rx_parar,
            recientes: Arc::clone(recientes),
            avisar: avisar.clone(),
        };
        Some(Activa { opciones: tx_op, parar: tx_parar, tarea: Activa::vigilar(ciclo) })
    }

    /// Runs the projection in its own task; a panic there ends that projection, never the engine.
    pub fn vigilar(ciclo: Ciclo) -> JoinHandle<()> {
        let avisar = ciclo.avisar.clone();
        // One task, so aborting it also ends the projection inside.
        tokio::spawn(async move {
            if AssertUnwindSafe(ciclo.correr()).catch_unwind().await.is_err() {
                avisar(Aviso::Error(
                    "La proyección falló por un error interno; puedes volver a conectar.".into(),
                ));
                avisar(Aviso::Estado(Estado::Inactivo));
            }
        })
    }
}

/// Asks the projection to end cleanly (TEARDOWN, link down); after 5 s it is cut short.
pub async fn terminar(activa: Option<Activa>, avisar: &Avisar) {
    let Some(mut activa) = activa else { return };
    activa.parar.send_replace(true);
    if tokio::time::timeout(Duration::from_secs(5), &mut activa.tarea).await.is_err() {
        activa.tarea.abort();
        avisar(Aviso::Estado(Estado::Inactivo));
    }
}

pub fn guardar_opciones(recientes: &Mutex<Recientes>, opciones: &Opciones) {
    let mut lista = recientes.lock().unwrap_or_else(|e| e.into_inner());
    lista.opciones = opciones.clone();
    lista.guardar();
}
