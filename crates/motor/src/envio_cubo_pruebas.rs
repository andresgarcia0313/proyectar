use super::*;
use std::sync::atomic::AtomicU32;
use std::sync::{mpsc, Arc};

#[test]
fn el_reloj_virtual_sale_al_ritmo_pedido() {
    let inicio = Instant::now();
    let (mut ahora, mut cubo) = (inicio, Cubo::nuevo(inicio));
    for _ in 0..100 {
        ahora += cubo.tomar(1328, 8_000_000.0, ahora);
    }
    let esperado = (100.0 * 1328.0 - RAFAGA_BYTES) * 8.0 / 8_000_000.0;
    let medido = (ahora - inicio).as_secs_f64();
    assert!((medido - esperado).abs() < 0.001, "{medido} contra {esperado}");
}

#[test]
fn en_reposo_solo_junta_una_rafaga_corta() {
    let inicio = Instant::now();
    let mut cubo = Cubo::nuevo(inicio);
    let despues = inicio + Duration::from_secs(10);
    let gratis = (0..10).take_while(|_| cubo.tomar(1328, 8_000_000.0, despues).is_zero()).count();
    assert_eq!(gratis, 4, "diez segundos quietos no dejan pasar una ráfaga larga");
}

/// Sends `n` packets queued at once and returns the spread between first and last arrival.
fn medir(n: usize, atraso_max: Duration) -> Duration {
    // A burst of 150 packets overflows the default 208 KiB buffer when the machine is busy and
    // the test then waited forever: room for all of them, and a timeout that fails instead.
    let receptor = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
    receptor.set_recv_buffer_size(1 << 20).unwrap();
    receptor.bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 0)).into()).unwrap();
    let receptor: UdpSocket = receptor.into();
    receptor.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let destino = receptor.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
    for _ in 0..n {
        tx.send((Instant::now(), vec![0u8; 1250])).unwrap();
    }
    drop(tx);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    let r = Reparto {
        destino,
        bps: || 10_000_000.0,
        atraso_max,
        fallar: |_| {},
        fallos_max: 1,
        cuenta: Arc::default(),
    };
    let hilo = std::thread::spawn(move || repartir(rx, sock, r, &AtomicBool::new(false)));
    let mut buf = [0u8; 1500];
    receptor.recv(&mut buf).unwrap();
    let primero = Instant::now();
    for _ in 1..n {
        receptor.recv(&mut buf).unwrap();
    }
    hilo.join().unwrap();
    primero.elapsed()
}

#[test]
fn el_ritmo_medido_en_la_red_es_el_del_cubo() {
    let esperado = (150.0 * 1250.0 - RAFAGA_BYTES) * 8.0 / 10_000_000.0;
    let medido = medir(150, Duration::from_secs(10)).as_secs_f64();
    // Never faster than asked; slower only by what a loaded machine adds to each wake-up.
    assert!(medido > esperado * 0.9 && medido < esperado * 2.5, "{medido} s contra {esperado} s");
}

#[test]
fn un_cuadro_atrasado_no_acumula_mas_retardo() {
    let a_ritmo = (150.0 * 1250.0 - RAFAGA_BYTES) * 8.0 / 10_000_000.0;
    let medido = medir(150, Duration::from_millis(5)).as_secs_f64();
    assert!(medido < a_ritmo / 3.0, "{medido} s: el tope de atraso no actuó");
}

#[test]
fn los_envios_fallidos_seguidos_se_avisan_una_vez() {
    let (tx, rx) = mpsc::channel();
    for _ in 0..8 {
        tx.send((Instant::now(), [0u8; 10])).unwrap();
    }
    drop(tx);
    let avisos = Arc::new(AtomicU32::new(0));
    let a = avisos.clone();
    let r = Reparto {
        destino: "[::1]:9".parse().unwrap(), // IPv6 from an IPv4 socket: every send fails
        bps: || 10_000_000.0,
        atraso_max: Duration::from_secs(1),
        fallar: move |_| {
            a.fetch_add(1, Relaxed);
        },
        fallos_max: 3,
        cuenta: Arc::default(),
    };
    repartir(rx, UdpSocket::bind("127.0.0.1:0").unwrap(), r, &AtomicBool::new(false));
    assert_eq!(avisos.load(Relaxed), 1);
}
