//! Log-spaced frequency bands from a short PCM window. Used for the live meter
//! (stationary spectrum — not a scrolling time waveform).

use std::collections::VecDeque;
use std::f32::consts::PI;

pub const SPECTRUM_BANDS: usize = 48;
pub const SPECTRUM_WINDOW: usize = 1_024;
const FREQ_MIN_HZ: f32 = 80.0;
const FREQ_MAX_HZ: f32 = 7_000.0;
/// Speech/meeting levels are well below 1.0; scale so quiet talk still moves the meter.
const DISPLAY_GAIN: f32 = 14.0;

pub struct SpectrumBuf {
    samples: VecDeque<f32>,
}

impl SpectrumBuf {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(SPECTRUM_WINDOW),
        }
    }

    pub fn push(&mut self, samples: &[f32]) {
        self.samples.extend(samples.iter().copied());
        while self.samples.len() > SPECTRUM_WINDOW {
            self.samples.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    pub fn bands(&self, sample_rate: f32) -> Vec<f32> {
        if self.samples.len() < 64 {
            return vec![0.0; SPECTRUM_BANDS];
        }
        let window: Vec<f32> = self.samples.iter().copied().collect();
        compute_log_bands(&window, sample_rate, SPECTRUM_BANDS)
    }
}

fn hann(i: usize, n: usize) -> f32 {
    0.5 - 0.5 * (2.0 * PI * i as f32 / n.saturating_sub(1).max(1) as f32).cos()
}

fn goertzel_mag(samples: &[f32], freq_hz: f32, sample_rate: f32) -> f32 {
    let omega = 2.0 * PI * freq_hz / sample_rate;
    let coeff = 2.0 * omega.cos();
    let mut s1 = 0.0f32;
    let mut s2 = 0.0f32;
    for (i, &x) in samples.iter().enumerate() {
        let s0 = x * hann(i, samples.len()) + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    let real = s1 - s2 * omega.cos();
    let imag = s2 * omega.sin();
    (real * real + imag * imag).sqrt() / samples.len() as f32
}

pub fn compute_log_bands(samples: &[f32], sample_rate: f32, bands: usize) -> Vec<f32> {
    let nyquist = sample_rate * 0.5 - 50.0;
    let f_max = FREQ_MAX_HZ.min(nyquist);
    let log_min = FREQ_MIN_HZ.ln();
    let log_max = f_max.ln();
    let mut out = Vec::with_capacity(bands);
    for i in 0..bands {
        let t = (i as f32 + 0.5) / bands as f32;
        let freq = (log_min + t * (log_max - log_min)).exp();
        let mag = goertzel_mag(samples, freq, sample_rate);
        // Mild high-frequency lift so the right side isn't dead on speech.
        let lifted = mag * (freq / FREQ_MIN_HZ).sqrt().min(6.0);
        out.push((lifted * DISPLAY_GAIN).clamp(0.0, 1.0));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{compute_log_bands, SPECTRUM_BANDS};
    use std::f32::consts::PI;

    fn sine(freq: f32, sr: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * PI * freq * i as f32 / sr).sin())
            .collect()
    }

    #[test]
    fn sine_440_peaks_in_lower_mid_bands() {
        let sr = 16_000.0;
        let bands = compute_log_bands(&sine(440.0, sr, 1_024), sr, SPECTRUM_BANDS);
        let (imax, &peak) = bands
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap();
        assert!(peak > 0.15, "peak was {peak}");
        assert!(
            (8..28).contains(&imax),
            "440 Hz peaked at band {imax}, expected lower-mid"
        );
    }

    #[test]
    fn silence_is_flat() {
        let bands = compute_log_bands(&[0.0; 1_024], 16_000.0, SPECTRUM_BANDS);
        assert!(bands.iter().all(|&v| v < 0.02));
    }
}
