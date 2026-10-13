//! A projection from start to finish: asks for the capture, runs sessions and, when one
//! fails, applies `Fallo::recovery` (the "catch") so the picture comes back on its own.

use crate::captura::Captura;
use crate::fallo::{Accion, Resultado};
use crate::medios::Codificador;
use crate::recientes::Recientes;
use crate::sesion::{self, Contexto, Fin};
use crate::wifi_direct::{Pantalla, WifiDirect};
use crate::{Avisar, Aviso, Estado, Opciones};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;

/// Failures in a row before giving up and telling the user.
const MAX_INTENTOS: u32 = 5;
/// A session that lasted this long was healthy: the count of failures starts over.
const SANA: Duration = Duration::from_secs(60);

pub struct Ciclo {
    pub wd: Arc<WifiDirect>,
    pub pantalla: Pantalla,
    pub opciones: watch::Receiver<Opciones>,
    pub parar: watch::Receiver<bool>,
    pub recientes: Arc<Mutex<Recientes>>,
    pub avisar: Avisar,
}

impl Ciclo {
    pub async fn correr(mut self) {
        let (mut codificador, mut captura, mut intento) = (Codificador::primero(), None::<Captura>, 0);
        loop {
            let inicio = Instant::now();
            let paso = self.paso(&mut captura, codificador).await;
            let fallo = match paso {
                Ok(Fin::Terminada) => break,
                Ok(Fin::Repetir) => continue,
                Err(fallo) => fallo,
            };
            intento = if inicio.elapsed() > SANA { 1 } else { intento + 1 };
            let accion = fallo.recovery();
            let rendirse = match accion {
                Accion::OtroCodificador => codificador.siguiente().map(|otro| codificador = otro).is_none(),
                Accion::NuevaCaptura => {
                    Captura::soltar(captura.take()).await;
                    false
                }
                Accion::Detener => true,
                Accion::Reintentar(_) | Accion::Reconectar => false,
            };
            if rendirse || intento > MAX_INTENTOS {
                (self.avisar)(Aviso::Error(format!("La proyección se detuvo: {fallo}")));
                break;
            }
            (self.avisar)(Aviso::Estado(Estado::Reconectando {
                nombre: self.pantalla.nombre.clone(),
                intento,
            }));
            let pausa = match accion {
                Accion::Reintentar(pausa) => pausa,
                _ => Duration::from_secs(1 << intento.min(4)),
            };
            if tokio::time::timeout(pausa, sesion::pedido(&mut self.parar)).await.is_ok() {
                break;
            }
        }
        Captura::soltar(captura).await;
        (self.avisar)(Aviso::Estado(Estado::Inactivo));
    }

    /// One session; the capture is asked for once and kept across retries.
    async fn paso(&mut self, captura: &mut Option<Captura>, codificador: Codificador) -> Resultado<Fin> {
        let fuente = self.opciones.borrow().fuente;
        if captura.as_ref().is_some_and(|c| c.fuente != fuente) {
            Captura::soltar(captura.take()).await;
        }
        if captura.is_none() {
            let token =
                self.recientes.lock().unwrap_or_else(|e| e.into_inner()).token(fuente).map(str::to_owned);
            let nueva = Captura::pedir(fuente, token.as_deref()).await?;
            let mut recientes = self.recientes.lock().unwrap_or_else(|e| e.into_inner());
            recientes.poner_token(fuente, nueva.token.clone());
            recientes.guardar();
            *captura = Some(nueva);
        }
        let Some(captura) = captura.as_ref() else { return Ok(Fin::Terminada) };
        sesion::intentar(Contexto {
            wd: &self.wd,
            pantalla: &self.pantalla,
            captura,
            codificador,
            opciones: &mut self.opciones,
            parar: &mut self.parar,
            avisar: &*self.avisar,
        })
        .await
    }
}
