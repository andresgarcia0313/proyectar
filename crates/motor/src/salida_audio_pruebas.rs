use super::*;

#[test]
fn argumentos_llevan_nombre_del_tv_icono_y_prioridad_minima() {
    let args = argumentos("Sa\"l'a");
    assert!(args.starts_with("sink_name=proyectar_tv rate=48000 channels=2"));
    assert!(args.contains("device.description=\"[TV] Sala\""));
    assert!(args.contains("device.icon_name=video-display"));
    assert!(args.contains("priority.session=0"));
}

#[test]
fn reconoce_solo_los_modulos_propios() {
    let lista = "536870915\tmodule-stream-restore\t\t\n\
                 536870920\tmodule-null-sink\tsink_name=proyectar_tv rate=48000\t\n\
                 536870921\tmodule-null-sink\tsink_name=proyectar_tv2\t\n";
    assert_eq!(modulos_propios(lista), vec!["536870920"]);
}

#[test]
fn lee_la_salida_elegida_de_pw_metadata() {
    let texto = "Found \"default\" metadata 41\n\
                 update: id:0 key:'default.configured.audio.sink' value:'{\"name\":\"alsa_x\"}' type:'Spa:String:JSON'";
    assert_eq!(valor_metadato(texto).as_deref(), Some("{\"name\":\"alsa_x\"}"));
    assert_eq!(valor_metadato("Found \"default\" metadata 41"), None);
}

#[test]
fn distingue_la_salida_propia_de_la_del_usuario() {
    assert!(es_propio(Some(&valor_propio())));
    assert!(es_propio(None));
    assert!(!es_propio(Some("{\"name\":\"easyeffects_sink\"}")));
}

/// Real PipeWire, no sound: creates the sink without taking the default and removes it.
/// `cargo test -p motor --lib -- --ignored sumidero_real`
#[test]
#[ignore]
fn sumidero_real_aparece_y_desaparece_sin_tocar_la_salida_elegida() {
    let sumideros = || correr("pactl", &["list", "sinks", "short"]).unwrap();
    let antes = elegida().unwrap();
    let salida = crate::salida_audio::SalidaTv::crear("Prueba", false).expect("sumidero");
    assert!(sumideros().contains(SUMIDERO));
    assert!(correr("pactl", &["list", "sinks"]).unwrap().contains("[TV] Prueba"));
    assert_eq!(elegida().unwrap(), antes);
    drop(salida);
    assert!(!sumideros().contains(SUMIDERO));
    assert_eq!(elegida().unwrap(), antes);
}
