use super::{leer_ahorro, puerto};
use crate::autodiagnostico::{revisar, CODIFICADORES};
use crate::autodiagnostico_wpa::nombre_en;

#[test]
fn lee_el_nombre_anunciado_de_la_salida_de_busctl() {
    let salida = "a{sv} 2 \"DeviceName\" s \"lenovo-ideapad\" \"PrimaryDeviceType\" ay 8 0 1 0 80 242 4 0 1";
    assert_eq!(nombre_en(salida).as_deref(), Some("lenovo-ideapad"));
    assert_eq!(nombre_en("a{sv} 2 \"DeviceName\" s \"\" \"X\" u 1").as_deref(), Some(""));
    assert_eq!(nombre_en("a{sv} 0"), None);
}

#[test]
fn el_ahorro_encendido_es_un_fallo_y_lo_ilegible_queda_sin_dato() {
    assert_eq!(leer_ahorro("p2p-wlp2s0-0", Some("Power save: off\n".into())).ok, Some(true));
    let encendido = leer_ahorro("wlp2s0", Some("Power save: on\n".into()));
    assert_eq!((encendido.nombre.as_str(), encendido.ok), ("ahorro_wlp2s0", Some(false)));
    assert_eq!(leer_ahorro("wlp2s0", None).ok, None);
}

#[test]
fn un_puerto_tomado_se_reporta_ocupado() {
    let tomado = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let numero = tomado.local_addr().unwrap().port();
    let c = puerto(numero);
    assert_eq!(c.ok, Some(false), "{}", c.detalle);
    drop(tomado);
    assert_eq!(puerto(numero).ok, Some(true));
}

#[test]
fn revisa_todos_los_requisitos_sin_fallar() {
    let checks = revisar();
    checks.iter().for_each(|c| eprintln!("{:<22} {:<12} {}", c.nombre, format!("{:?}", c.ok), c.detalle));
    let nombres: Vec<String> = checks.into_iter().map(|c| c.nombre).collect();
    let esperados =
        ["wifi_direct", "dispatcher", "nombre_wifi_direct", "kwin_screencast", "pactl", "pw-metadata"];
    for n in esperados.iter().chain(CODIFICADORES.iter()).chain(["iw", "puerto_5001"].iter()) {
        assert!(nombres.iter().any(|x| x == n), "falta {n} en {nombres:?}");
    }
}
