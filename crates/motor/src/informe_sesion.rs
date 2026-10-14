//! Splits the event log into projection sessions. A session opens with the first `captura` or
//! `invitar` and closes with `resumen_sesion`, the engine going idle (`Inactivo`, `Buscando`) or a
//! new start. Logs older than `resumen_sesion` get their numbers rebuilt from `estadisticas`.

use serde_json::Value;

#[derive(Debug, Default)]
pub struct Sesion {
    pub eventos: Vec<Value>,
    pub resumen: Option<Value>,
}

pub fn agrupar(eventos: &[Value]) -> Vec<Sesion> {
    let (mut hechas, mut abierta) = (Vec::<Sesion>::new(), None::<Sesion>);
    for e in eventos {
        let estado = e["estado"].as_str().unwrap_or_default();
        match e["evento"].as_str().unwrap_or_default() {
            "resumen_sesion" => {
                let mut sesion = match abierta.take() {
                    Some(a) => a,
                    None if hechas.last().is_some_and(|u| u.resumen.is_none()) => {
                        hechas.pop().unwrap_or_default()
                    }
                    None => Sesion::default(),
                };
                sesion.eventos.push(e.clone());
                sesion.resumen = Some(e.clone());
                hechas.push(sesion);
            }
            "captura" | "invitar" if abierta.is_none() => {
                abierta = Some(Sesion { eventos: vec![e.clone()], resumen: None })
            }
            "inicio" => hechas.extend(abierta.take()),
            "estado" if estado.starts_with("Inactivo") || estado.starts_with("Buscando") => {
                if let Some(mut a) = abierta.take() {
                    a.eventos.push(e.clone());
                    hechas.push(a);
                }
            }
            _ => abierta.iter_mut().for_each(|a| a.eventos.push(e.clone())),
        }
    }
    hechas.extend(abierta);
    hechas
}

impl Sesion {
    pub fn primero(&self, evento: &str) -> Option<&Value> {
        self.eventos.iter().find(|e| e["evento"] == evento)
    }
    pub fn pantalla(&self) -> String {
        let r = self.resumen.as_ref().and_then(|r| r["pantalla"].as_str());
        r.or_else(|| self.primero("invitar")?["pantalla"].as_str())
            .unwrap_or("pantalla desconocida")
            .to_owned()
    }
    pub fn modo(&self) -> String {
        let texto =
            |v: &Value| v.as_str().map(str::to_owned).or_else(|| v.is_object().then(|| v.to_string()));
        let opciones = self.primero("transmitiendo").map(|t| &t["opciones"]);
        let partes = match (self.resumen.as_ref(), opciones) {
            (Some(r), _) if !r["modo"].is_null() => vec![texto(&r["fuente"]), texto(&r["modo"])],
            (_, Some(o)) => vec![texto(&o["fuente"]), texto(&o["latencia"])],
            _ => vec![self.primero("captura").and_then(|c| texto(&c["fuente"]))],
        };
        let partes: Vec<String> = partes.into_iter().flatten().collect();
        if partes.is_empty() {
            "modo desconocido".into()
        } else {
            partes.join(", ")
        }
    }
    pub fn inicio(&self) -> &str {
        self.eventos.first().and_then(|e| e["hora"].as_str()).unwrap_or("hora desconocida")
    }
    pub fn duracion_s(&self) -> Option<f64> {
        if let Some(d) = self.resumen.as_ref().and_then(|r| r["duracion_s"].as_f64()) {
            return Some(d);
        }
        let hora = |e: Option<&Value>| crate::informe_metricas::segundos(e?["hora"].as_str()?);
        Some(hora(self.eventos.last())? - hora(self.eventos.first())?)
    }
    /// The `resumen_sesion` numbers, or what `estadisticas` and `latencia_medida` still tell.
    pub fn metricas(&self) -> Value {
        self.resumen.clone().unwrap_or_else(|| crate::informe_metricas::reconstruir(&self.eventos))
    }
}
