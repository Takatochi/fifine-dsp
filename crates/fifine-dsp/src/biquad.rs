use fifine_domain::{DspNode, ProcessContext};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    HighPass { cutoff_hz: f32, q: f32 },
    PeakingEq { freq_hz: f32, q: f32, gain_db: f32 },
    HighShelf { freq_hz: f32, q: f32, gain_db: f32 },
}

#[derive(Debug, Clone)]
pub struct BiquadFilter {
    label: &'static str,
    enabled: bool,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    pub fn new(filter_type: FilterType, sample_rate: f32) -> Self {
        Self::with_label("Biquad IIR Filter", filter_type, sample_rate)
    }

    pub fn with_label(label: &'static str, filter_type: FilterType, sample_rate: f32) -> Self {
        let mut filter = Self {
            label,
            enabled: true,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        filter.update_coefficients(filter_type, sample_rate);
        filter
    }

    pub fn update_coefficients(&mut self, filter_type: FilterType, sample_rate: f32) {
        match filter_type {
            FilterType::HighPass { cutoff_hz, q } => {
                let w0 = (2.0 * PI * cutoff_hz) / sample_rate;
                let cos_w0 = w0.cos();
                let sin_w0 = w0.sin();
                let alpha = sin_w0 / (2.0 * q);

                let a0 = 1.0 + alpha;
                self.b0 = ((1.0 + cos_w0) / 2.0) / a0;
                self.b1 = -(1.0 + cos_w0) / a0;
                self.b2 = ((1.0 + cos_w0) / 2.0) / a0;
                self.a1 = (-2.0 * cos_w0) / a0;
                self.a2 = (1.0 - alpha) / a0;
            }
            FilterType::PeakingEq {
                freq_hz,
                q,
                gain_db,
            } => {
                let a = 10.0_f32.powf(gain_db / 40.0);
                let w0 = (2.0 * PI * freq_hz) / sample_rate;
                let cos_w0 = w0.cos();
                let sin_w0 = w0.sin();
                let alpha = sin_w0 / (2.0 * q);

                let a0 = 1.0 + alpha / a;
                self.b0 = (1.0 + alpha * a) / a0;
                self.b1 = (-2.0 * cos_w0) / a0;
                self.b2 = (1.0 - alpha * a) / a0;
                self.a1 = (-2.0 * cos_w0) / a0;
                self.a2 = (1.0 - alpha / a) / a0;
            }
            FilterType::HighShelf {
                freq_hz,
                q,
                gain_db,
            } => {
                let a = 10.0_f32.powf(gain_db / 40.0);
                let w0 = (2.0 * PI * freq_hz) / sample_rate;
                let cos_w0 = w0.cos();
                let sin_w0 = w0.sin();
                let alpha = sin_w0 / (2.0 * q);
                let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

                let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
                self.b0 = (a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha)) / a0;
                self.b1 = (-2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0;
                self.b2 = (a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha)) / a0;
                self.a1 = (2.0 * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0;
                self.a2 = ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha) / a0;
            }
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, sample: f32) -> f32 {
        let out = self.b0 * sample + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = sample;
        self.y2 = self.y1;
        self.y1 = out;
        out
    }
}

impl DspNode for BiquadFilter {
    fn name(&self) -> &'static str {
        self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn process_frame(&mut self, samples: &mut [f32], _ctx: &mut ProcessContext) {
        if !self.enabled {
            return;
        }
        for s in samples.iter_mut() {
            *s = self.process_sample(*s);
        }
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn generate_sine(freq_hz: f32, sample_rate: f32, num_samples: usize) -> Vec<f32> {
        (0..num_samples)
            .map(|i| (2.0 * PI * freq_hz * (i as f32) / sample_rate).sin())
            .collect()
    }

    fn rms(samples: &[f32]) -> f32 {
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / (samples.len() as f32)).sqrt()
    }

    #[test]
    fn test_high_pass_filter_attenuates_sub_bass_rumble() {
        let sr = 48_000.0;
        let mut hpf = BiquadFilter::new(
            FilterType::HighPass {
                cutoff_hz: 100.0,
                q: 0.70710678,
            },
            sr,
        );
        let mut ctx = ProcessContext::new(sr);

        let mut low_rumble = generate_sine(25.0, sr, 4800);
        let mut voice_freq = generate_sine(1000.0, sr, 4800);

        let orig_low_rms = rms(&low_rumble[2400..]);
        let orig_voice_rms = rms(&voice_freq[2400..]);

        hpf.process_frame(&mut low_rumble, &mut ctx);
        hpf.reset();
        hpf.process_frame(&mut voice_freq, &mut ctx);

        let filtered_low_rms = rms(&low_rumble[2400..]);
        let filtered_voice_rms = rms(&voice_freq[2400..]);

        assert!(
            filtered_low_rms < orig_low_rms * 0.15,
            "25 Hz rumble should be strongly attenuated"
        );
        assert!(
            (filtered_voice_rms - orig_voice_rms).abs() < 0.05,
            "1 kHz voice tone should pass untouched"
        );
    }

    #[test]
    fn test_peaking_eq_boost_and_cut() {
        let sr = 48_000.0;
        let mut boost_eq = BiquadFilter::new(
            FilterType::PeakingEq {
                freq_hz: 3200.0,
                q: 1.0,
                gain_db: 6.0,
            },
            sr,
        );
        let mut cut_eq = BiquadFilter::new(
            FilterType::PeakingEq {
                freq_hz: 3200.0,
                q: 1.0,
                gain_db: -6.0,
            },
            sr,
        );
        let mut ctx = ProcessContext::new(sr);

        let orig = generate_sine(3200.0, sr, 4800);
        let orig_rms = rms(&orig[2400..]);

        let mut boosted = orig.clone();
        boost_eq.process_frame(&mut boosted, &mut ctx);
        assert!(rms(&boosted[2400..]) > orig_rms * 1.8);

        let mut cut = orig;
        cut_eq.process_frame(&mut cut, &mut ctx);
        assert!(rms(&cut[2400..]) < orig_rms * 0.6);
    }

    #[test]
    fn test_biquad_bypass_and_reset() {
        let sr = 48_000.0;
        let mut filter = BiquadFilter::new(
            FilterType::HighShelf {
                freq_hz: 8000.0,
                q: 0.707,
                gain_db: 12.0,
            },
            sr,
        );
        let mut ctx = ProcessContext::new(sr);

        filter.set_enabled(false);
        assert!(!filter.is_enabled());

        let orig = vec![0.5, -0.5, 0.25, -0.25];
        let mut buf = orig.clone();
        filter.process_frame(&mut buf, &mut ctx);
        assert_eq!(buf, orig);
    }
}
