---
lang: es-CO
título: Proyectar - Plan de pruebas
versión: 1.0.1
---

# Proyectar - Plan de pruebas

## Control de cambios

| Versión | Fecha | Autor | Descripción del cambio |
| --- | --- | --- | --- |
| 1.0.0 | 2026-10-12 | Andrés García | Versión inicial: pruebas automáticas con TV simulado, pruebas manuales con la Samsung y criterios de salida |
| 1.0.1 | 2026-10-12 | Andrés García | Limpieza para publicar: receptor de pruebas descrito por marca y serie |

## 1. Estrategia

Pirámide: la mayoría de las pruebas son automáticas y no necesitan hardware. El protocolo se prueba
contra un **televisor simulado** que habla RTSP en memoria; lo que depende de radio, GPU y TV real se
prueba a mano con una lista fija. Ninguna prueba automática emite sonido ni toca la red real.

## 2. Pruebas automáticas

Orden del flujo: formato, lint, tipos y pruebas.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

| Capa | Prueba (en `crates/wfd/tests`) | Qué garantiza |
| --- | --- | --- |
| Formatos | `caps_resolution_by_the_latency_mode` | La resolución elegida respeta el perfil de latencia |
| Formatos | `baseline_only_sink_with_several_codec_entries` | Un sink solo Baseline con varias entradas de códec negocia bien |
| Formatos | `audio_prefers_aac_then_lpcm` | Preferencia de audio AAC, luego LPCM |
| Formatos | `information_elements` | IE anunciado y lectura del tipo de dispositivo |
| Formatos | `parameters_and_ports` | Cuerpos `text/parameters` y puertos RTP |
| RTSP | `rtsp_round_trip_and_garbage`, `tolerates_samsung_duplicate_content_length`, `bounds_endless_lines_and_huge_bodies` | Ida y vuelta, `Content-Length` duplicado de Samsung, líneas y cuerpos acotados |
| Sesión | `negotiates_like_a_samsung_tv` | M1 a M7 completos con un sink con forma de Samsung 2020 |
| Sesión | `reports_idr_requests_and_teardown` | Eventos IDR y TEARDOWN |
| Sesión | `sends_keepalives_and_detects_a_silent_sink` | M16 periódico y detección de sink mudo |
| Sesión | `refuses_a_sink_without_common_video` | Error sin formato en común |
| Sesión | `idr_requests_do_not_postpone_the_keepalive_deadline` | El IDR no aplaza el keep-alive |

El simulado vive en `crates/wfd/tests/common/mod.rs` (`Sink`, cadena `SAMSUNG_VIDEO`).

Pruebas por añadir en `motor` (pendientes, necesarias para cerrar la 0.1.0):

| Prueba | Garantiza |
| --- | --- |
| `recovery` por cada variante de `Fallo` | La tabla de la sección 3 de la arquitectura |
| `Latencia::perfil` por modo | 720p60 8000 kbps, 1080p30 10000 kbps, 1080p60 14000 kbps |
| `Recientes` con archivo dañado | Arranca vacío sin fallar |
| Integración: `Session` con un TV simulado por localhost y el orquestador | Flujo conectar, proyectar, desconectar sin hardware |

Sabotaje de validación: cambiar un campo del M4 o quitar el audio debe romper al menos una prueba.

## 3. Pruebas manuales con un televisor Samsung (serie TU7000, 2020)

Condiciones: TV y portátil a menos de 5 m, **volumen de la TV en silencio** (audio de la sesión
desactivado, se envía silencio), Wi-Fi normal del portátil conectada.

| Id | Pasos | Resultado esperado |
| --- | --- | --- |
| M-01 | En la TV, Source -> Screen Mirroring. En Proyectar, esperar la lista | La TV aparece por nombre en menos de 10 s |
| M-02 | Elegir la TV | La TV muestra el aviso: **aceptar en pantalla** con el control remoto |
| M-03 | Aceptar | La imagen aparece en menos de 15 s; el panel pasa a Proyectando |
| M-04 | Mover una ventana y medir con cronómetro en pantalla | Latencia de 250 ms o menos; meta 150 ms |
| M-05 | Cambiar de Trabajo a Juegos y a Videos | Cambio en menos de 3 s, resolución y kbps según el perfil |
| M-06 | Duplicar y luego Extender | En Extender aparece un monitor virtual en Plasma |
| M-07 | Reproducir 30 min de video | Sin cortes ni deriva; CPU del proceso menor de 25 % de un núcleo |
| M-08 | Apagar y encender el Wi-Fi P2P a mitad de sesión | Recuperación sola en 20 s o menos |
| M-09 | Pulsar Desconectar | La TV sale de Screen Mirroring en menos de 3 s; sin conexión de NetworkManager activa |
| M-10 | Segunda conexión el mismo día | Sin diálogo del portal si hay token |

Si M-02 falla (la TV ignora la conexión), limpiar los dispositivos autorizados en la TV y reintentar:
Samsung no admite dispositivos no verificados aunque haya un token guardado.

## 4. Criterios de salida de la 0.1.0

- Pruebas automáticas en verde y sin avisos de lint.
- M-01 a M-04, M-08 y M-09 aprobadas en la TV real; el resto se registra como pendiente si falla.
- Defectos abiertos: ninguno de gravedad alta.
