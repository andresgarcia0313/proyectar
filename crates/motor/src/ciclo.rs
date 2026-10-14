//! A projection from start to finish: asks for the capture, runs sessions and, when one
//! fails, applies `Fallo::recovery` (the "catch") so the picture comes back on its own.

use crate::busqueda::encontrar;
use crate::captura::Captura;
use crate::diagnostico::{Causa, Diagnostico};
use crate::fallo::Accion;
use crate::medios::Codificador;
use crate::recientes::Recientes;
use crate::registro;
use crate::sesion::{self, Fin};
use crate::wifi_direct::{Pantalla, WifiDirect};
use crate::{Avisar, Aviso, Estado, Opciones};
use serde_json::json;
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
        let (mut codificador, mut captura, mut intento) =
            (Codificador::para(self.opciones.borrow().latencia.perfil()), None::<Captura>, 0);
        loop {
            let (inicio, mut fase) = (Instant::now(), Causa::Captura);
            let paso = self.paso(&mut captura, codificador, &mut fase).await;
            let fallo = match paso {
                Ok(Fin::Terminada) => break,
                Ok(Fin::Repetir) => {
                    codificador = Codificador::para(self.opciones.borrow().latencia.perfil());
                    continue;
                }
                Err(fallo) => fallo,
            };
            intento = if inicio.elapsed() > SANA { 1 } else { intento + 1 };
            let accion = fallo.recovery();
            registro::anotar(
                "fallo",
                json!({"fase": fase, "detalle": fallo.to_string(), "accion": format!("{accion:?}"),
                "intento": intento, "duro_s": inicio.elapsed().as_secs_f32()}),
            );
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
                (self.avisar)(Aviso::Diagnostico(Diagnostico::de(&fallo, fase)));
                (self.avisar)(Aviso::Error(format!("La proyección se detuvo: {fallo}")));
                break;
            }
            (self.avisar)(Aviso::Estado(Estado::Reconectando {
                nombre: self.pantalla.nombre.clone(),
                intento,
                motivo: Diagnostico::de(&fallo, fase).titulo,
            }));
            let pausa = match accion {
                Accion::Reintentar(pausa) => pausa,
                _ => Duration::from_secs(1 << intento.min(4)),
            };
            if tokio::time::timeout(pausa, sesion::pedido(&mut self.parar)).await.is_ok() {
                break;
            }
            // NetworkManager forgets a peer once discovery stops: look again so the next attempt
            // never invites a stale peer ("P2P peer ... is unknown").
            if let Ok(vista) = encontrar(&self.wd, &self.pantalla.mac).await {
                self.pantalla = vista;
            }
            self.wd.parar().await;
        }
        Captura::soltar(captura).await;
        (self.avisar)(Aviso::Estado(Estado::Inactivo));
    }
}
