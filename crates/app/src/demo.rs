//! `PROYECTAR_DEMO=<escena>` feeds the interface with made-up reports instead of starting
//! the engine, to review screens and take screenshots without a TV: `lista`, `conectando`,
//! `proyectando`, `reconectando`, `error`, `vacia`.

use motor::{Aviso, Estadisticas, Estado, Pantalla};

fn pantalla(nombre: &str, mac: &str, senal: u8) -> Pantalla {
    Pantalla { nombre: nombre.into(), mac: mac.into(), senal, ruta: String::new() }
}

pub fn reproducir(escena: &str, enviar: impl Fn(Aviso)) {
    let sala = || "[TV] Sala".to_string();
    let cercanas = vec![
        pantalla("[TV] Sala", "aa:bb:cc:00:00:01", 82),
        pantalla("Proyector oficina", "aa:bb:cc:00:00:02", 55),
        pantalla("Pantalla cuarto", "aa:bb:cc:00:00:03", 24),
    ];
    if escena != "vacia" {
        enviar(Aviso::Pantallas(cercanas.clone()));
        enviar(Aviso::Recientes(vec![cercanas[0].clone()]));
    }
    let estado = match escena {
        "conectando" => Estado::EsperandoPantalla { nombre: sala() },
        "proyectando" | "error" => Estado::Proyectando { nombre: sala() },
        "reconectando" => Estado::Reconectando { nombre: sala(), intento: 2 },
        _ => Estado::Buscando,
    };
    enviar(Aviso::Estado(estado));
    if escena == "proyectando" {
        enviar(Aviso::Estadisticas(Estadisticas {
            ancho: 1920,
            alto: 1080,
            fps: 29.8,
            kbps: 9_840,
            gpu: true,
        }));
    }
    if escena == "error" {
        enviar(Aviso::Error("La pantalla cerró la conexión. Revisa que el televisor siga encendido.".into()));
    }
}
