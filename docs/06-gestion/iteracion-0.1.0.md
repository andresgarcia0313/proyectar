---
lang: es-CO
título: Proyectar - Iteración 0.1.0
versión: 1.0.1
---

# Proyectar - Iteración 0.1.0

## Control de cambios

| Versión | Fecha | Autor | Descripción del cambio |
| --- | --- | --- | --- |
| 1.0.0 | 2026-10-12 | Andrés García | Alcance, reparto, estado real al abrir la iteración, riesgos y criterio de cierre |
| 1.0.1 | 2026-10-12 | Andrés García | Limpieza para publicar: sin datos del hogar ni referencias a notas internas |

## 1. Inicio: alcance

Pedido del 2026-10-12 (10:58, hora de Colombia): una aplicación `proyectar` en Rust con Qt y QML en
paleta Drácula que iguale a Win+K, descubra receptores Miracast, conecte por Wi-Fi Direct, negocie
WFD (Wi-Fi Display) por RTSP y transmita pantalla y audio con H.264 por VA-API de baja latencia.

Restricciones: sin emitir audio durante las pruebas, sin robar el foco del teclado y solo lectura en el
equipo Windows usado como referencia.

## 2. Decisiones al inicio

- Investigar antes de codificar: cómo proyecta Windows (observado en un equipo con Windows 11) y el
  protocolo (código de GND, Android y FluxCast).
- Tres crates: `wfd` (protocolo puro), `motor` (sistema y medios) y `app` (interfaz). Detalle en
  `docs/02-diseno/arquitectura.md`.
- Escribir `wfd` primero con un televisor simulado, porque es lo único probable sin hardware.
- No reutilizar GND por sus cinco causas de latencia (arquitectura, sección 4).

## 3. Reparto

| Rol | Entregable | Dueño de ficheros |
| --- | --- | --- |
| Investigación de Windows | Notas de investigación | Investigación |
| Investigación del protocolo | Notas de investigación | Investigación |
| Desarrollo | `crates/**`, `Cargo.toml` | Desarrollo |
| Documentación | `README.md`, `docs/**` | Documentación |

## 4. Estado al 2026-10-12

| Pieza | Estado | Evidencia |
| --- | --- | --- |
| `wfd`: RTSP, M1 a M7, keep-alive, IDR, IE | Hecho, con 13 pruebas | `crates/wfd/tests/` |
| `motor::fallo` (Fallo -> Acción) | Hecho; faltan sus pruebas | `crates/motor/src/fallo.rs` |
| `motor::wifi_direct` y `nm` | Hecho; sin probar contra la TV | `crates/motor/src/wifi_direct.rs` |
| `motor::captura`, `modo`, `recientes` | Hecho; sin pruebas | `crates/motor/src/` |
| `motor::medios` (GStreamer) | En escritura | `crates/motor/src/medios.rs` |
| `motor::proyeccion` (orquestador) | Esqueleto: las órdenes no hacen nada aún | `crates/motor/src/proyeccion.rs` |
| `app` (Qt/QML) | Pendiente | - |

## 5. Plan restante

1. Cadena de medios con `vah264lpenc` y MPEG-TS sobre RTP.
2. Orquestador: buscar, conectar, servir RTSP en 7236, arrancar medios, aplicar la política de recuperación.
3. Pruebas del motor y de integración con el televisor simulado por localhost.
4. Interfaz `app` según `docs/03-ui-ux/especificacion-ui-ux.md`.
5. Pruebas manuales M-01 a M-10 con la Samsung (plan de pruebas, sección 3).
6. Commit con formato `tipo: descripción`.

## 6. Riesgos abiertos

| Riesgo | Probabilidad | Respuesta |
| --- | --- | --- |
| La Samsung rechaza el diseño MPEG-TS de `mpegtsmux` | Media | Muxer de ffmpeg con `-tables_version 1` |
| Tamaño del monitor virtual fijo en 1920x1080 | Alta | Escalar con `vapostproc`, guardar el token del portal |
| Latencia sobre la meta de 150 ms | Media | Revisar colas, GOP y VBV (tamaño del búfer del codificador) |
| El portal exige diálogo en cada sesión | Media | `PersistMode::ExplicitlyRevoked` y token en `recientes.json` |

## 7. Criterio de cierre

Pruebas automáticas en verde, pruebas manuales M-01 a M-04, M-08 y M-09 aprobadas con la TV real,
documentación al día y commit hecho. Los tiempos reales y las lecciones se anotan aquí al cerrar: en
esta versión solo consta el estado de apertura, no se estimó duración por fase.
