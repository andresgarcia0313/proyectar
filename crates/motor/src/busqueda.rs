//! Discovery: keeps Wi-Fi Direct looking and reports the displays in sight, only when
//! the list changes, so the interface never redraws for nothing.

use crate::fallo::{Fallo, Resultado};
use crate::wifi_direct::{Pantalla, WifiDirect};
use crate::{Avisar, Aviso, Estado};
use std::sync::Arc;
use std::time::Duration;

/// Discovery windows last 30 s; ask again a little before each one closes.
const RENOVAR_CADA: u32 = 25;

pub async fn buscar(wd: Arc<WifiDirect>, avisar: Avisar) {
    avisar(Aviso::Estado(Estado::Buscando));
    let mut ultima: Option<Vec<Pantalla>> = None;
    let mut reloj = tokio::time::interval(Duration::from_secs(1));
    for vuelta in 0u32.. {
        reloj.tick().await;
        if vuelta % RENOVAR_CADA == 0 {
            if let Err(fallo) = wd.buscar().await {
                // The card may be busy (scan, roaming): say why and try on the next round.
                avisar(Aviso::Error(format!("No se pudo buscar pantallas: {fallo}")));
                continue;
            }
        }
        if let Ok(pantallas) = wd.pantallas().await {
            if ultima.as_ref() != Some(&pantallas) {
                avisar(Aviso::Pantallas(pantallas.clone()));
                ultima = Some(pantallas);
            }
        }
    }
}

/// The display with this MAC, looking for it for a while if it is not in sight yet
/// (a recent display picked straight from the list).
pub async fn encontrar(wd: &WifiDirect, mac: &str) -> Resultado<Pantalla> {
    for vuelta in 0..20 {
        let vista = wd.pantallas().await?.into_iter().find(|p| p.mac.eq_ignore_ascii_case(mac));
        if let Some(pantalla) = vista {
            return Ok(pantalla);
        }
        if vuelta == 0 {
            wd.buscar().await?;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(Fallo::WifiDirect("la pantalla no está a la vista; revisa que esté encendida".into()))
}
