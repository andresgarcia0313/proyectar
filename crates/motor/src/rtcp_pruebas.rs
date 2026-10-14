use super::*;

fn cabecera(cuenta: u8, pt: u8, palabras: u16) -> Vec<u8> {
    let mut v = vec![0x80 | cuenta, pt];
    v.extend((palabras - 1).to_be_bytes());
    v
}

fn bloque_de(ssrc: u32, fraccion: u8, perdidos: [u8; 3], jitter: u32) -> Vec<u8> {
    let mut v = ssrc.to_be_bytes().to_vec();
    v.push(fraccion);
    v.extend(perdidos);
    v.extend(7000u32.to_be_bytes()); // highest sequence
    v.extend(jitter.to_be_bytes());
    v.extend([0; 8]); // LSR, DLSR
    v
}

/// Like the Samsung's 84-byte reports: RR with one block, then SDES with a CNAME.
#[test]
fn rr_compuesto_con_sdes() {
    let mut p = cabecera(1, RR, 8);
    p.extend(0x1234_5678u32.to_be_bytes());
    p.extend(bloque_de(SSRC, 64, [0, 0x01, 0x2c], 450));
    let sdes = [
        cabecera(1, 202, 13),
        0x1234_5678u32.to_be_bytes().to_vec(),
        vec![1, 40], // CNAME, 40 bytes
        vec![b'a'; 40],
        vec![0; 2],
    ];
    p.extend(sdes.concat());
    assert_eq!(p.len(), 84);
    let i = leer(&p, SSRC).unwrap();
    assert_eq!(i, Informe { perdida_pct: 25.0, perdidos: 300, jitter_ms: 5.0 });
}

#[test]
fn sr_con_varios_bloques_elige_el_nuestro() {
    let mut p = cabecera(2, SR, 19);
    p.extend([9; 24]); // sender SSRC, NTP, RTP time, counts
    p.extend(bloque_de(7, 128, [0, 0, 9], 90));
    p.extend(bloque_de(SSRC, 0, [0xff, 0xff, 0xff], 9000));
    let i = leer(&p, SSRC).unwrap();
    assert_eq!((i.perdida_pct, i.perdidos, i.jitter_ms), (0.0, -1, 100.0));
    assert_eq!(leer(&p, 99).unwrap().perdidos, 9, "sin el nuestro, el primero");
}

#[test]
fn rechaza_lo_que_no_es_rtcp() {
    let mut rr = cabecera(1, RR, 8);
    rr.extend([0; 28]);
    assert!(leer(&rr[..20], SSRC).is_none(), "truncado");
    rr[0] = 0x41;
    assert!(leer(&rr, SSRC).is_none(), "versión 1");
    assert!(leer(&[0x80, 33, 0, 1, 0, 0, 0, 0], SSRC).is_none(), "RTP, no RTCP");
    assert!(leer(&cabecera(0, RR, 2).into_iter().chain([0; 4]).collect::<Vec<_>>(), SSRC).is_none());
}
