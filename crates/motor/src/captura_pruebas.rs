use super::estado::{elegir, Estado};
use super::tamano_para;
use crate::modo::Latencia;

#[test]
fn tamano_del_perfil_en_16_9() {
    assert_eq!(tamano_para(Latencia::Juegos.perfil()), (1280, 720));
    assert_eq!(tamano_para(Latencia::Trabajo.perfil()), (1920, 1080));
    assert_eq!(tamano_para(Latencia::Videos.perfil()), (1920, 1080));
}

#[test]
fn principal_sin_pantallas_es_ninguna() {
    assert!(Estado::default().principal().is_none());
}

#[test]
fn ventana_queda_para_el_portal() {
    let directa = super::kwin::pedir(crate::modo::Fuente::Ventana, "TV", (1280, 720));
    assert!(matches!(directa, Ok(None)));
}

#[test]
fn principal_es_la_primera_del_orden_de_kde() {
    let orden = vec!["HDMI-A-1".to_owned(), "eDP-1".to_owned()];
    assert_eq!(elegir(&[Some("eDP-1"), Some("HDMI-A-1")], &orden), Some(1));
    assert_eq!(elegir(&[Some("eDP-1"), None], &[]), Some(0), "sin orden: la primera anunciada");
    assert_eq!(elegir(&[Some("DP-2")], &orden), Some(0), "orden viejo: la primera anunciada");
    assert_eq!(elegir(&[], &orden), None);
}
