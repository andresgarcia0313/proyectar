//! A display in sight, as the interface lists it.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pantalla {
    pub nombre: String,
    pub mac: String,
    /// Signal strength, 0-100.
    pub senal: u8,
    #[serde(skip)]
    pub ruta: String,
}

/// Any nearby device picks its own name: keep it short and drop control and bidi characters,
/// which could fake another name on screen.
pub fn limpio(nombre: &str) -> String {
    let raro = |c: &char| {
        c.is_control() || ('\u{202a}'..='\u{202e}').contains(c) || ('\u{2066}'..='\u{2069}').contains(c)
    };
    nombre.chars().filter(|c| !raro(c)).take(64).collect::<String>().trim().to_owned()
}

#[cfg(test)]
mod tests {
    #[test]
    fn nombre_sin_trucos() {
        assert_eq!(super::limpio("  [TV] Sala\u{202e}txt.exe\n"), "[TV] Salatxt.exe");
        assert_eq!(super::limpio(&"x".repeat(200)).len(), 64);
    }
}
