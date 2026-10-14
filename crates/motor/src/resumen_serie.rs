//! A series of samples of one measure along a session, with the figures the summary needs.

/// Ten hours at one sample per second: a forgotten projection must not grow without end.
const TOPE: usize = 36_000;

/// Every measure a session summary is built from.
#[derive(Debug, Default)]
pub struct Muestras {
    pub fps: Serie,
    pub kbps: Serie,
    pub retardo: Serie,
    pub perdida: Serie,
    pub jitter: Serie,
    pub rtt: Serie,
    pub senal: Serie,
    pub tx: Serie,
    pub cpu: Serie,
    pub carga: Serie,
    pub psi: Serie,
}

#[derive(Debug, Default, Clone)]
pub struct Serie(Vec<f32>);

impl Serie {
    pub fn poner(&mut self, v: f32) {
        if v.is_finite() && self.0.len() < TOPE {
            self.0.push(v);
        }
    }

    pub fn poner_si(&mut self, v: Option<f32>) {
        if let Some(v) = v {
            self.poner(v);
        }
    }

    pub fn largo(&self) -> usize {
        self.0.len()
    }

    pub fn media(&self) -> Option<f32> {
        (!self.0.is_empty()).then(|| self.0.iter().sum::<f32>() / self.0.len() as f32)
    }

    /// Nearest-rank percentile, `q` from 0 to 1 (0.95 is the value 95 % of samples stay under).
    pub fn percentil(&self, q: f32) -> Option<f32> {
        let mut v = self.0.clone();
        v.sort_by(f32::total_cmp);
        let i = ((q * v.len() as f32).ceil() as usize).clamp(1, v.len().max(1)) - 1;
        v.get(i).copied()
    }

    pub fn min(&self) -> Option<f32> {
        self.0.iter().copied().reduce(f32::min)
    }

    pub fn max(&self) -> Option<f32> {
        self.0.iter().copied().reduce(f32::max)
    }
}

/// Rounds to one decimal for the log: nobody reads 37.48213 ms.
pub fn r1(v: Option<f32>) -> Option<f32> {
    v.map(|v| (v * 10.0).round() / 10.0)
}

#[cfg(test)]
mod tests {
    use super::Serie;

    #[test]
    fn media_percentiles_y_extremos() {
        let mut s = Serie::default();
        (1..=100).for_each(|v| s.poner(v as f32));
        s.poner(f32::NAN);
        assert_eq!((s.largo(), s.media(), s.min(), s.max()), (100, Some(50.5), Some(1.0), Some(100.0)));
        assert_eq!(
            (s.percentil(0.95), s.percentil(0.05), s.percentil(1.0)),
            (Some(95.0), Some(5.0), Some(100.0))
        );
        let vacia = Serie::default();
        assert_eq!((vacia.media(), vacia.percentil(0.95), vacia.min()), (None, None, None));
        assert_eq!(super::r1(Some(37.48)), Some(37.5));
    }
}
