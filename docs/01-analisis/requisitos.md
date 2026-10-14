---
lang: es-CO
título: Proyectar - Especificación de requisitos
versión: 1.3.0
---

# Proyectar - Especificación de requisitos

## Control de cambios

| Versión | Fecha | Autor | Descripción del cambio |
| --- | --- | --- | --- |
| 1.0.0 | 2026-10-12 | Andrés García | Versión inicial: visión, funciones de Windows a igualar, requisitos, exclusiones y riesgos |
| 1.0.1 | 2026-10-12 | Andrés García | Limpieza para publicar: equipos de referencia sin datos del hogar ni nombres de equipos |
| 1.1.0 | 2026-10-13 | Andrés García | Primera prueba con TV real: RF-14 a RF-17 y hoja de ruta tomada del emisor de Windows |
| 1.3.0 | 2026-10-13 | Andrés García | RF-23 a RF-26: técnicas de baja latencia del modo Juegos, envío a ritmo, RTCP y cronómetro |
| 1.2.0 | 2026-10-13 | Andrés García | RF-18 a RF-22 tras la segunda prueba: captura directa, audio del TV, latencia medida y comparación con macOS y Android |

## 1. Visión

Proyectar es una aplicación de escritorio para Linux (probada en Plasma 6 sobre Wayland) que envía la pantalla y el
audio del computador a un televisor Miracast, con la misma sencillez de Win+K en Windows y la menor
latencia posible. Actúa como **emisor** (en la terminología del estándar, *source*) de Wi-Fi Display
(WFD), el protocolo de Miracast. Núcleo en Rust, interfaz en Qt 6 con QML.

**Equipos de referencia:** emisor portátil con Wi-Fi Realtek RTL8852BE (P2P) y GPU Intel Iris Xe con
codificador H.264 de bajo consumo; receptor televisor Samsung serie TU7000 (2020, Tizen).

## 2. Lo que hace Windows (Win+K y Win+P)

Windows no ofrece ninguna API pública de emisor Miracast: la emisión vive en el sistema y solo la usa
la barra de Win+K. Por eso Proyectar implementa WFD completo. Fuente: observación de un equipo
con Windows 11 23H2 y documentación de Microsoft.

## 3. Requisitos funcionales (prioridad MoSCoW: debe, debería, podría, no hará)

| Id | Función de Windows a igualar | Requisito | Criterio de aceptación | Prioridad |
| --- | --- | --- | --- | --- |
| RF-01 | Lista de pantallas (Win+K) | Descubrir receptores por Wi-Fi Direct y listarlos con nombre y señal | Con la TV en Screen Mirroring, aparece por nombre en menos de 10 s y desaparece al apagarla | Debe |
| RF-02 | Conectar | Enlazar por Wi-Fi Direct y negociar WFD (M1 a M7) | Tras aceptar en la TV, la imagen sale en menos de 15 s; la Wi-Fi normal sigue conectada | Debe |
| RF-03 | Transmitir imagen y audio | Video H.264 por GPU y audio AAC o LPCM en MPEG-TS sobre RTP | 1080p30 sin cortes durante 30 min; nunca se emite solo video (Samsung lo rechaza) | Debe |
| RF-04 | Desconectar | Cerrar la sesión (TEARDOWN) y bajar el enlace | La TV vuelve a su fuente anterior en menos de 3 s y no queda conexión de NetworkManager | Debe |
| RF-05 | Duplicar (Win+P) | Enviar una pantalla existente | La TV muestra lo mismo que el portátil | Debe |
| RF-06 | Extender (Win+P) | Crear un monitor virtual con el portal de KDE | Aparece una pantalla nueva en Plasma y la TV la muestra; al desconectar desaparece | Debería |
| RF-07 | Solo segunda pantalla | Apagar la pantalla del portátil mientras se proyecta | Se deja fuera de la 0.1.0: exige tocar la configuración de pantallas de Plasma | Podría |
| RF-08 | Cambiar modo (receptor) | Perfiles de latencia Juegos, Trabajo y Videos (720p60, 1080p30, 1080p60) cambiables sin reconectar | Cambio visible en menos de 3 s; los kbps y la resolución coinciden con el perfil | Debería |
| RF-09 | Recuperación | Reintentar o reconectar solos ante caída del enlace, de la captura o del codificador | Apagar y encender el Wi-Fi P2P: la imagen vuelve sin acción del usuario en menos de 20 s | Debe |
| RF-10 | Dispositivo emparejado | Recordar las últimas pantallas y los permisos de captura | Segunda conexión sin diálogo del portal cuando hay token guardado | Debería |
| RF-11 | Cursor | Cursor del ratón visible en el video | Cursor incrustado en la imagen (Windows lo envía por canal aparte; se deja para después) | Debe |
| RF-12 | Seguridad de enlace | Aceptación en pantalla de la TV por WPS pulsador, sin PIN | Conexión sin pedir PIN; PIN opcional | Podría |
| RF-13 | Estado y errores | Mostrar fase, resolución, fps, kbps y si se usa GPU; errores en español claro | El panel refleja los cambios en menos de 1 s | Debe |
| RF-14 | Fases y diagnóstico | Mostrar el paso en curso (invitar, enlazar, abrir video, proyectar), preguntar si apareció "Permitir" en la TV y, al fallar, decir en qué paso y qué revisar | Cada fallo de la prueba real queda explicado sin leer bitácoras del sistema | Debe |
| RF-15 | Registro de eventos | Una línea JSON por evento en `~/.local/state/proyectar/eventos.jsonl`, con rotación a 4 MB | Un intento fallido se reconstruye paso a paso con `jq` | Debe |
| RF-16 | Nombre del equipo | Anunciar por Wi-Fi Direct el nombre del equipo y el tipo "Computador" (categoría WPS 1, subcategoría 1), como Windows | La TV muestra el nombre del equipo en el aviso de permitir, no "0" | Debe |
| RF-18 | Captura sin diálogos | En KDE, capturar con el protocolo de KWin: Duplicar transmite el monitor, Extender crea una salida con el nombre del TV y el tamaño del perfil (720p en Juegos) | Ningún diálogo de Wayland; Duplicar nunca entrega una salida virtual | Debe |
| RF-19 | Audio en el TV | Mientras se proyecta, salida de audio "[TV] nombre" en los dispositivos de KDE; se vuelve la elegida y al terminar se restaura la anterior | Aparece y desaparece con la sesión; un cierre brusco se limpia al siguiente arranque | Debe |
| RF-20 | Latencia visible | Medir el retardo dentro del equipo (captura a red) y mostrarlo en la tarjeta | El valor cambia cada segundo y se anota en el registro | Debe |
| RF-21 | Monitoreo | Cada 10 s: video, retardos, CPU del proceso y radio Wi-Fi Direct (señal, tasa, reintentos) | Un corte o un retraso se explica con el registro | Debe |
| RF-22 | Ventana | Recordar el tamaño entre aperturas y mostrar la versión en el pie | Al reabrir conserva el tamaño | Debería |
| RF-23 | Juegos de baja latencia | LPCM si el TV lo ofrece, refresco intra con búfer de 3 cuadros y respaldo a la GPU si no sostiene los fps | Retardo percibido menor que en Trabajo, medido con el cronómetro | Debe |
| RF-24 | Envío a ritmo | Emisor UDP propio con cubo de fichas y DSCP 46 | Sin ráfagas por cuadro; prueba de ritmo medido | Debería |
| RF-25 | RTCP | Leer los informes del TV en el puerto 5001 y ajustar el bitrate con pérdida sostenida | Pérdida y fluctuación en el registro | Debería |
| RF-26 | Medir latencia | Cronómetro en milisegundos para fotografiar emisor y TV | La diferencia se puede anotar en el registro | Debería |
| RF-17 | Datos del receptor | Pedir en M3 los parámetros opcionales que pide Windows (EDID, conector, modo preferido) y anotarlos | Un receptor que no los conoce no falla; los que responde quedan en el registro | Debería |

## 4. Requisitos no funcionales

| Id | Requisito | Medida | Prioridad |
| --- | --- | --- | --- |
| RNF-01 | Latencia de extremo a extremo (captura a pantalla de la TV) | Tope de la especificación: 250 ms. Meta propia: 150 ms o menos en modo Trabajo | Debe |
| RNF-02 | Codificación por GPU (VA-API, `vah264lpenc`), con cadena de reserva por software | Uso de CPU del proceso inferior a 25 % de un núcleo en 1080p30 | Debe |
| RNF-03 | El motor nunca bloquea la interfaz | Toda orden devuelve en menos de 50 ms; los resultados llegan por avisos | Debe |
| RNF-04 | Compatibilidad con el servidor RTSP de la TV Samsung | Tolera `Content-Length` duplicado y respeta el diseño MPEG-TS (PID 0x1000, 0x1011, 0x1100) | Debe |
| RNF-05 | Seguridad de red | Solo se abre el TCP 7236 en interfaces `p2p-*`; sin servicios en la LAN | Debe |
| RNF-06 | Sin sonido inesperado | Con audio desactivado se envía silencio, no se corta la pista | Debe |
| RNF-07 | Archivos de código nuevos de 100 líneas o menos; formato, lint y pruebas en verde | `cargo fmt`, `clippy`, `cargo test` sin avisos | Debe |

## 5. Lo que no aplica

| Función de Windows | Decisión | Motivo |
| --- | --- | --- |
| HDCP (High-bandwidth Digital Content Protection, protección de contenido) | No hará | Exige licencia y el contenido protegido no se emite desde Linux; se declara `wfd_content_protection: none` |
| UIBC (User Input Back Channel, teclado y ratón desde la TV) | No hará | El televisor Samsung de referencia no devuelve entrada; los mensajes M10 a M15 quedan sin implementar |
| MICE (Miracast over Infrastructure, TCP 7250) | No hará | La TV no anuncia MICE; Wi-Fi Direct cubre el caso |
| Modo receptor (recibir pantallas) | No hará | Fuera del alcance: Proyectar solo emite |
| Canal propio de imagen del cursor | Podría | Mejora menor; el cursor incrustado basta |

## 6. Riesgos

| Riesgo | Efecto | Respuesta |
| --- | --- | --- |
| La TV rechaza la sesión con el diseño MPEG-TS de `mpegtsmux` | Pantalla negra | `mpegtsmux` no fija `version_number=1` en PAT y PMT; reserva con el muxer de ffmpeg |
| El portal de KDE crea el monitor virtual a 1920x1080 sin opción de tamaño | 720p exige escalar | Escalar con `vapostproc` y guardar el token para reutilizar el tamaño |
| El driver Wi-Fi pierde el grupo P2P | Corte de imagen | Recuperación RF-09 |

## 7. Hoja de ruta tomada del emisor de Windows

Inspección de solo lectura del emisor de Windows 11 (`winmde.dll` negocia RTSP; `WiFiDisplay.dll` y
`dafWfdProvider.dll` manejan el enlace). Lo que Windows hace y Proyectar aún no:

| Función de Windows | Evidencia | Viabilidad en Linux | Prioridad |
| --- | --- | --- | --- |
| Grupo persistente: reconectar sin aceptar de nuevo | `WFDProfile:persistent`, `IsReconnect` | wpa_supplicant por D-Bus (`persistent`, `Invite`); NetworkManager no lo expone | Debería |
| Bitrate según los informes RTCP del receptor | `CBandwidthManager::OnRR`, `CRtcpNackSplitter` | Hecho en 0.4.0 (RF-25) | Hecho |
| Cambio de formato sin reconectar | `OnDynamicFormatChange`, `microsoft_format_change_capability` | Capa RTSP (M4 nuevo dentro de la sesión) | Podría |
| Canal del grupo junto al de la Wi-Fi normal | `IsInterfaceInMultiChannelConfiguration` | wpa_supplicant (`frequency`, `go_intent`) | Podría |
| Envío a ritmo constante | `EnableSendTimeSmoothing` | Hecho en 0.4.0 con un emisor propio (RF-24) | Hecho |
| PIN opcional | `PreferPinBasedPairing` | wpa_supplicant | Podría |

## 8. Comparación con otros emisores

| Función | Windows (Win+K) | macOS | Android (Smart View) | Proyectar |
| --- | --- | --- | --- | --- |
| Protocolo | Miracast | AirPlay (no Miracast) | Miracast o Google Cast | Miracast |
| Duplicar y extender | Sí | Sí (AirPlay) | Solo duplicar | Sí |
| Modo de latencia | Juego, trabajo, video | No | No | Juegos, Trabajo, Videos |
| Audio al TV como dispositivo | Sí | Sí | Sí | Sí (RF-19) |
| Reconexión sin aceptar de nuevo | Sí (grupo persistente) | Sí | Sí | Pendiente (§7) |
| Ocultar notificaciones mientras proyecta | Sí (asistente de concentración) | Sí | Sí | Lo hace Plasma al compartir pantalla |
| Entrada desde el TV (UIBC) | Sí | No | Parcial | No hará |
| Latencia y diagnóstico visibles | No | No | No | Sí (RF-14, RF-20) |

