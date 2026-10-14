use super::*;

#[test]
fn el_sr_lleva_cabecera_ssrc_y_cuentas() {
    let p = sr(1, 0x0102_0304_0506_0708, 90_000, 10, 13_160);
    assert_eq!(p[..4], [0x80, 200, 0, 6], "versión 2, sin bloques, 28 bytes");
    assert_eq!(p[4..8], 1u32.to_be_bytes());
    assert_eq!(p[8..16], [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(u32::from_be_bytes(p[16..20].try_into().unwrap()), 90_000);
    assert_eq!(u32::from_be_bytes(p[20..24].try_into().unwrap()), 10);
    assert_eq!(u32::from_be_bytes(p[24..28].try_into().unwrap()), 13_160);
}

#[test]
fn ntp_cuenta_desde_1900_con_fraccion() {
    let t = UNIX_EPOCH + Duration::from_millis(1500);
    let n = ntp(t);
    assert_eq!(n >> 32, DESDE_1900 + 1);
    assert_eq!(n as u32, 1 << 31, "medio segundo es la mitad de la fracción");
    assert_eq!(medio(n), (((DESDE_1900 + 1) as u32) << 16) | 0x8000);
}

#[test]
fn rtt_resta_el_tiempo_que_el_tv_retuvo_el_informe() {
    // SR left at 10 s, the TV held it 0,5 s, the RR came back at 10,52 s: 20 ms of link.
    let s = |x: f64| (x * 65_536.0) as u32;
    let rtt = rtt_ms(s(10.0), s(0.5), s(10.52)).unwrap();
    assert!((rtt - 20.0).abs() < 0.1, "{rtt}");
    assert_eq!(rtt_ms(0, 0, s(3.0)), None, "sin LSR el TV no ha visto ningún SR");
    assert_eq!(rtt_ms(s(10.0), s(1.0), s(10.5)), None, "negativo: eco ajeno o reloj roto");
}

#[test]
fn la_cuenta_extiende_el_tiempo_rtp_del_ultimo_paquete() {
    let c = Cuenta::default();
    let t = Instant::now();
    assert!(c.informe(1, t, 0).is_none(), "sin paquetes no hay informe");
    let mut rtp = [0u8; 1328];
    rtp[4..8].copy_from_slice(&1_000u32.to_be_bytes());
    c.anotar(&rtp, t);
    c.anotar(&rtp, t);
    let p = c.informe(1, t + Duration::from_millis(100), 7).unwrap();
    assert_eq!(u32::from_be_bytes(p[16..20].try_into().unwrap()), 1_000 + 9_000, "100 ms a 90 kHz");
    assert_eq!(u32::from_be_bytes(p[20..24].try_into().unwrap()), 2);
    assert_eq!(u32::from_be_bytes(p[24..28].try_into().unwrap()), 2 * 1316);
}
