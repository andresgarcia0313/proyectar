//! The streaming phase of a session: answers the TV, watches the pipeline and the link,
//! reports numbers every second and logs them every ten.

use crate::enlace::Enlace;
use crate::fallo::{Fallo, Resultado};
use crate::medios::Medios;
use crate::registro;
use crate::resumen::Acumulador;
use crate::sesion::Fin;
use crate::{Aviso, Opciones};
use serde_json::json;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::watch;

pub async fn transmitir(
    mut sesion: wfd::Session<TcpStream>,
    medios: Medios,
    enlace: &Enlace,
    opciones: &mut watch::Receiver<Opciones>,
    parar: &mut watch::Receiver<bool>,
    avisar: &(dyn Fn(Aviso) + Send + Sync),
    resumen: &mut Acumulador,
) -> Resultado<Fin> {
    let mut reloj = tokio::time::interval(Duration::from_secs(1));
    let (mut segundos, mut lentos, mut cpu) = (0u64, 0u32, crate::monitor::Cpu::nuevo());
    let fin = loop {
        tokio::select! {
            evento = sesion.next_event() => {
                let evento = evento?;
                registro::anotar("tv_evento", json!({"evento": format!("{evento:?}")}));
                match evento {
                    wfd::Event::IdrRequest => {
                        resumen.idr();
                        medios.pedir_idr()
                    }
                    wfd::Event::Teardown => break Ok(Fin::Terminada),
                    wfd::Event::Play | wfd::Event::Pause => {}
                }
            },
            fallo = medios.fallo() => break Err(fallo),
            r = enlace.caido() => break Err(Fallo::WifiDirect(format!("el enlace se cayó (código {})", r?))),
            _ = pedido(parar) => {
                let _ = tokio::time::timeout(Duration::from_secs(3), sesion.close()).await;
                return Ok(Fin::Terminada);
            }
            _ = opciones.changed() => break Ok(Fin::Repetir),
            _ = reloj.tick() => {
                let e = medios.estadisticas();
                segundos += 1;
                resumen.segundo(&e);
                // Five slow seconds in a row: start over with the GPU encoder (next in the chain).
                lentos = if medios.rezagado(e.fps) { lentos + 1 } else { 0 };
                if lentos >= 5 {
                    let detalle = format!("no sostiene {:.0} fps con el equipo ocupado", e.fps);
                    break Err(Fallo::Codificador { codificador: "x264enc", detalle });
                }
                if segundos % 10 == 0 {
                    let (radio, cpu, carga) = (crate::monitor::radio().await, cpu.medir(), crate::monitor_carga::medir());
                    registro::anotar("estadisticas", json!({"s": segundos, "video": e, "cpu_pct": cpu, "carga": carga, "radio": radio}));
                    resumen.muestra(radio.as_ref(), cpu, carga);
                }
                avisar(Aviso::Estadisticas(e));
            }
        }
    };
    drop(medios);
    fin
}

/// Resolves when the user asks to stop (or nobody can ask any more); holds no lock across awaits.
pub async fn pedido(parar: &mut watch::Receiver<bool>) {
    let _ = parar.wait_for(|p| *p).await.map(|_| ());
}
