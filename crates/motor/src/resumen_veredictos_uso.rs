//! Verdicts for the manual tests 5 to 8: sound to the TV and back, Extender and Duplicar.

use crate::modo::Fuente;
use crate::resumen_tipos::{desde, Resultado::*, Resumen, Veredicto};
use crate::resumen_veredictos::{n, veredicto, MIN_S};

/// Frame rate kept, % of the negotiated one: 90 looks smooth, under 75 the motion judders.
pub const FPS_PCT: (f32, f32) = (90.0, 75.0);

pub fn veredictos(r: &Resumen) -> [Veredicto; 4] {
    [audio(r), restauracion(r), extender(r), duplicar(r)]
}

fn audio(r: &Resumen) -> Veredicto {
    let (resultado, detalle) = match (r.audio_tv, r.audio_medido_s, r.audio_senal_s) {
        (Some(false), ..) => (SinDatos, "el audio al TV estaba apagado (se envía silencio)".into()),
        (_, 0, _) => (SinDatos, "no se midió el nivel del sonido".into()),
        (_, medido, 0) => (Aviso, format!("{medido} s medidos y todos en silencio: ¿sonaba algo?")),
        (_, medido, con) => {
            (Bien, format!("{con} de {medido} s con sonido; que se oiga en el TV no se comprueba"))
        }
    };
    veredicto(5, "audio", resultado, detalle)
}

fn restauracion(r: &Resumen) -> Veredicto {
    let (resultado, detalle) = match r.audio_restaurada {
        Some(true) => (Bien, "al terminar, la salida de sonido del equipo ya no era la del TV"),
        Some(false) => (Mal, "al terminar, la salida de sonido seguía en el TV"),
        None => (SinDatos, "no se comprobó (audio al TV apagado o pactl no respondió)"),
    };
    veredicto(6, "restauracion_audio", resultado, detalle.into())
}

fn extender(r: &Resumen) -> Veredicto {
    let (resultado, detalle) = match (r.fuente, r.salida_virtual_ok, r.salida_virtual_quitada) {
        (Some(Fuente::Extender), Some(true), Some(false)) => {
            (Mal, "la pantalla virtual no se quitó al terminar")
        }
        (Some(Fuente::Extender), Some(true), _) => {
            (Bien, "KDE mostró la pantalla del TV con el tamaño pedido")
        }
        (Some(Fuente::Extender), Some(false), _) => {
            (Mal, "KDE no mostró la pantalla del TV con el tamaño pedido")
        }
        (Some(Fuente::Extender), None, _) => {
            (SinDatos, "no se pudo comprobar (captura por portal o sin kscreen-doctor)")
        }
        _ => (SinDatos, "en esta sesión no se usó Extender"),
    };
    veredicto(7, "extender", resultado, detalle.into())
}

/// Duplicar is judged by the frame rate it held: a mirrored screen that cannot keep up stutters.
fn duplicar(r: &Resumen) -> Veredicto {
    if r.fuente != Some(Fuente::Duplicar) {
        return veredicto(8, "duplicar", SinDatos, "en esta sesión no se usó Duplicar".into());
    }
    match fps(r) {
        Some((resultado, detalle)) => veredicto(8, "duplicar", resultado, detalle),
        None => veredicto(8, "duplicar", SinDatos, format!("hacen falta {MIN_S} s transmitiendo")),
    }
}

/// The frame rate held against the negotiated one, also used by the stability test.
pub fn fps(r: &Resumen) -> Option<(crate::resumen_tipos::Resultado, String)> {
    let media = r.fps_media.filter(|_| r.transmision_s >= MIN_S && r.fps_objetivo > 0)?;
    let pct = media * 100.0 / r.fps_objetivo as f32;
    let p5 = r.fps_p5.map_or(String::new(), |v| format!(", el 5 % peor bajo {}", n(v)));
    let detalle = format!("{} de {} fps ({} %){p5}", n(media), r.fps_objetivo, n(pct));
    Some((desde(pct, FPS_PCT.0, FPS_PCT.1), detalle))
}
