//! One step of the projection cycle: the capture is asked for once and kept across retries.

use crate::captura::Captura;
use crate::ciclo::Ciclo;
use crate::diagnostico::Causa;
use crate::fallo::Resultado;
use crate::medios::Codificador;
use crate::sesion::{self, Contexto, Fin};

impl Ciclo {
    pub(crate) async fn paso(
        &mut self,
        captura: &mut Option<Captura>,
        codificador: Codificador,
        fase: &mut Causa,
    ) -> Resultado<Fin> {
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
            fase,
        })
        .await
    }
}
