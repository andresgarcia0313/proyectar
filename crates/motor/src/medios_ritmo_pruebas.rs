use super::*;

#[test]
fn baja_con_perdida_seguida_y_no_por_un_pedido_suelto() {
    let r = Ritmo::nuevo(10_000);
    assert_eq!(r.perdida(), None);
    assert_eq!(r.perdida(), Some(7_500));
    assert_eq!(r.perdida(), Some(5_600));
    for _ in 0..5 {
        r.perdida();
    }
    assert_eq!(r.perdida(), None, "nunca baja del piso");
    assert_eq!(r.calma(), None, "sin calma no sube");
}

#[test]
fn la_perdida_rtcp_sostenida_baja_y_la_suelta_no() {
    let r = Ritmo::nuevo(10_000);
    assert_eq!((r.informe(0.0), r.informe(5.0), r.informe(1.0), r.informe(5.0)), (None, None, None, None));
    assert_eq!(r.informe(5.0), Some(7_500), "dos informes con pérdida seguidos");
    assert_eq!(r.informe(5.0), None, "cada escalón pide otro par");
    assert_eq!(r.informe(9.0), Some(5_600));
    assert_eq!(r.actual(), 5_600);
    assert_eq!(r.calma(), None, "la pérdida reciente impide subir");
}
