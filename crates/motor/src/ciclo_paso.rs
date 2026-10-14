//! One step of the projection cycle: the capture is asked for once and kept across retries.

use crate::captura::{self, Captura};
use crate::ciclo::Ciclo;
use crate::diagnostico::Causa;
use crate::fallo::Resultado;
use crate::medios::Codificador;
use crate::modo::Fuente;
use crate::resumen::Acumulador;
use crate::resumen_salida::{comprobar, soltar};
use crate::sesion::{self, Contexto, Fin};

impl Ciclo {
    pub(crate) async fn paso(
        &mut self,
        captura: &mut Option<Captura>,
        codificador: Codificador,
        fase: &mut Causa,
        resumen: &mut Acumulador,
    ) -> Resultado<Fin> {
        let (fuente, perfil) = {
            let opciones = self.opciones.borrow();
            (opciones.fuente, opciones.latencia.perfil())
        };
        let tamano = captura::tamano_para(perfil);
        // A virtual screen keeps the size it was made with: a new profile needs a new one.
        let vieja = |c: &Captura| c.fuente != fuente || (fuente == Fuente::Extender && c.pedido != tamano);
        if captura.as_ref().is_some_and(vieja) {
            resumen.quitada(soltar(captura.take(), &self.pantalla.nombre).await);
        }
        if captura.is_none() {
            let token =
                self.recientes.lock().unwrap_or_else(|e| e.into_inner()).token(fuente).map(str::to_owned);
            let nueva = Captura::pedir(fuente, token.as_deref(), &self.pantalla.nombre, tamano).await?;
            if nueva.es_portal() {
                let mut recientes = self.recientes.lock().unwrap_or_else(|e| e.into_inner());
                recientes.poner_token(fuente, nueva.token.clone());
                recientes.guardar();
            }
            resumen.salida_virtual = comprobar(&nueva, &self.pantalla.nombre, tamano).await;
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
            resumen,
        })
        .await
    }
}
