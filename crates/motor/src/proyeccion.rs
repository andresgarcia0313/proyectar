//! The orchestrator: one engine thread takes the interface's commands in order, so a slow step never blocks the window.

use crate::activa::{guardar_opciones, terminar, Activa};
use crate::busqueda::buscar;
use crate::recientes::Recientes;
use crate::registro;
use crate::wifi_direct::WifiDirect;
use crate::{Avisar, Aviso, Opciones};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

enum Orden {
    Buscar,
    DejarDeBuscar,
    Conectar(String, Opciones),
    Cambiar(Opciones),
    Desconectar,
}

#[derive(Clone)]
pub struct Motor {
    ordenes: mpsc::UnboundedSender<Orden>,
}

impl Motor {
    /// Starts the engine's own runtime; `avisar` is called from engine threads.
    pub fn iniciar(avisar: impl Fn(Aviso) + Send + Sync + 'static) -> Self {
        let (ordenes, rx) = mpsc::unbounded_channel();
        let avisar: Avisar = registro::envolver(avisar);
        let hilo = std::thread::Builder::new().name("motor".into());
        let _ = hilo.spawn(move || {
            match tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build() {
                Ok(rt) => rt.block_on(atender(rx, avisar)),
                Err(e) => avisar(Aviso::Error(format!("El motor no pudo arrancar: {e}"))),
            }
        });
        Self { ordenes }
    }
    /// Starts looking for displays; `Aviso::Pantallas` arrives when the list changes.
    pub fn buscar(&self) {
        let _ = self.ordenes.send(Orden::Buscar);
    }
    pub fn dejar_de_buscar(&self) {
        let _ = self.ordenes.send(Orden::DejarDeBuscar);
    }
    /// Projects to the display with this MAC address.
    pub fn conectar(&self, mac: String, opciones: Opciones) {
        let _ = self.ordenes.send(Orden::Conectar(mac, opciones));
    }
    /// Applies new options to the running projection (latency, audio, source).
    pub fn cambiar(&self, opciones: Opciones) {
        let _ = self.ordenes.send(Orden::Cambiar(opciones));
    }
    pub fn desconectar(&self) {
        let _ = self.ordenes.send(Orden::Desconectar);
    }
}

async fn atender(mut rx: mpsc::UnboundedReceiver<Orden>, avisar: Avisar) {
    tokio::task::spawn_blocking(crate::salida_audio::limpiar_huerfano); // left by a crash
    let recientes = Arc::new(Mutex::new(Recientes::cargar()));
    avisar(Aviso::Recientes(recientes.lock().unwrap_or_else(|e| e.into_inner()).pantallas.clone()));
    let (mut wd, mut busqueda, mut activa) =
        (None::<Arc<WifiDirect>>, None::<JoinHandle<()>>, None::<Activa>);
    while let Some(orden) = rx.recv().await {
        if matches!(orden, Orden::Buscar | Orden::Conectar(..) | Orden::DejarDeBuscar) {
            if let Some(b) = busqueda.take() {
                b.abort();
            }
        }
        if wd.is_none() {
            match WifiDirect::abrir().await {
                Ok(abierto) => wd = Some(Arc::new(abierto)),
                Err(fallo) => {
                    avisar(Aviso::Error(format!("Wi-Fi Direct no está disponible: {fallo}")));
                    continue;
                }
            }
        }
        let Some(wd) = wd.clone() else { continue };
        match orden {
            Orden::Buscar => busqueda = Some(tokio::spawn(buscar(Arc::clone(&wd), avisar.clone()))),
            Orden::DejarDeBuscar => wd.parar().await,
            Orden::Cambiar(nuevas) => {
                guardar_opciones(&recientes, &nuevas);
                if let Some(a) = &activa {
                    a.opciones.send_replace(nuevas);
                }
            }
            Orden::Desconectar => terminar(activa.take(), &avisar).await,
            Orden::Conectar(mac, opciones) => {
                terminar(activa.take(), &avisar).await;
                activa = Activa::lanzar(wd, &mac, opciones, &recientes, &avisar).await;
            }
        }
    }
    terminar(activa, &avisar).await;
}
