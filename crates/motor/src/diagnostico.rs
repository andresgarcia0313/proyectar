//! Why a projection did not work, in words the user can act on. Each failure is mapped to
//! the phase where it happened, so the interface can say what to check instead of a code.

use crate::fallo::Fallo;

/// The phase that broke, from the user's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Causa {
    /// The TV never answered the invitation: its "Allow" prompt did not show or was not accepted.
    SinRespuestaTv,
    /// Linked, but the TV never opened the video session.
    SinSesion,
    /// The TV turned the session down or spoke a dialect we could not agree on.
    Rechazada,
    /// The Wi-Fi Direct link dropped while projecting.
    EnlaceCaido,
    /// The screen capture (portal) failed or was cancelled.
    Captura,
    /// No video encoder worked.
    Codificador,
    /// Wi-Fi Direct itself is missing or NetworkManager failed.
    WifiDirect,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Diagnostico {
    pub causa: Causa,
    pub titulo: String,
    pub detalle: String,
    /// What to try, in order.
    pub pasos: Vec<String>,
}

impl Diagnostico {
    pub fn de(fallo: &Fallo, fase: Causa) -> Self {
        let causa = match fallo {
            Fallo::Captura(_) | Fallo::CapturaCancelada => Causa::Captura,
            Fallo::Codificador { .. } => Causa::Codificador,
            Fallo::Protocolo(e) if !e.is_transient() => Causa::Rechazada,
            Fallo::Red(_) if fase == Causa::WifiDirect => Causa::WifiDirect,
            _ => fase,
        };
        let (titulo, pasos): (&str, &[&str]) = match causa {
            Causa::SinRespuestaTv => (
                "El televisor no respondió a la invitación",
                &[
                    "En el televisor abre Fuente y elige Screen Mirroring (Duplicar pantalla).",
                    "Si aparece un aviso para permitir la conexión, acéptalo con el control.",
                    "En Samsung: Configuración > General > Administrador de dispositivos externos > Administrador de conexión de dispositivos: Notificación de acceso en Preguntar y borra este equipo de la lista de bloqueados.",
                ],
            ),
            Causa::SinSesion => (
                "El televisor se enlazó pero no abrió el video",
                &[
                    "Deja el televisor en la pantalla de Screen Mirroring mientras conecta.",
                    "Apaga y enciende el televisor con el botón (no solo en reposo) y vuelve a intentar.",
                    "Revisa que el cortafuegos permita el puerto 7236/tcp en las interfaces p2p.",
                ],
            ),
            Causa::Rechazada => (
                "El televisor rechazó la sesión",
                &["Prueba el modo Trabajo o Videos.", "Actualiza el software del televisor."],
            ),
            Causa::EnlaceCaido => (
                "Se perdió el enlace con el televisor",
                &["Acerca el equipo al televisor.", "Evita microondas o paredes gruesas en medio."],
            ),
            Causa::Captura => {
                ("No se pudo capturar la pantalla", &["Vuelve a conectar y elige la pantalla en el diálogo."])
            }
            Causa::Codificador => ("Ningún codificador de video funcionó", &["Revisa los controladores VA-API."]),
            Causa::WifiDirect => (
                "Wi-Fi Direct no está disponible",
                &["Activa el Wi-Fi.", "Comprueba que NetworkManager esté en marcha."],
            ),
        };
        Diagnostico {
            causa,
            titulo: titulo.into(),
            detalle: fallo.to_string(),
            pasos: pasos.iter().map(|p| (*p).into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_fase_decide_salvo_fallos_propios() {
        let d = Diagnostico::de(&Fallo::SinRespuesta, Causa::SinSesion);
        assert_eq!(d.causa, Causa::SinSesion);
        assert!(!d.pasos.is_empty());
        let c = Diagnostico::de(&Fallo::CapturaCancelada, Causa::SinRespuestaTv);
        assert_eq!(c.causa, Causa::Captura);
    }
}
