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
