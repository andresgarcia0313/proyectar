//! `PROYECTAR_DEMO=<escena>` feeds the interface with made-up reports instead of starting
//! the engine, to review screens and take screenshots without a TV: `lista`, `invitando`,
//! `conectando`, `negociando`, `proyectando`, `reconectando`, `error`, `diagnostico`, `vacia`.

use motor::{Aviso, Causa, Diagnostico, Estadisticas, Estado, Fallo, Pantalla};

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
        "invitando" => Estado::Conectando { nombre: sala() },
        "conectando" => Estado::EsperandoPantalla { nombre: sala() },
        "negociando" => Estado::Negociando { nombre: sala() },
        "diagnostico" => Estado::Inactivo,
        "proyectando" | "error" => Estado::Proyectando { nombre: sala() },
        "reconectando" => Estado::Reconectando {
            nombre: sala(),
            intento: 2,
            motivo: "Se perdió el enlace con el televisor".into(),
        },
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
    if escena == "diagnostico" {
        // The bare error must give way to the diagnosis that follows it.
        enviar(Aviso::Error("El televisor no respondió.".into()));
        enviar(Aviso::Diagnostico(Diagnostico::de(&Fallo::SinRespuesta, Causa::SinRespuestaTv)));
    }
}
