use fifine_domain::{DspNode, ProcessContext};

#[derive(Debug, Clone)]
pub struct Compressor {
    enabled: bool,
    threshold_db: f32,
    ratio: f32,
    attack_coeff: f32,
    release_coeff: f32,
    makeup_gain_lin: f32,
    env_db: f32,
}

impl Compressor {
    pub fn new(
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        makeup_gain_db: f32,
        sample_rate: f32,
    ) -> Self {
        let attack_coeff = (-1.0 / (0.001 * attack_ms * sample_rate)).exp();
        let release_coeff = (-1.0 / (0.001 * release_ms * sample_rate)).exp();
        let makeup_gain_lin = 10.0_f32.powf(makeup_gain_db / 20.0);

        Self {
            enabled: true,
            threshold_db,
            ratio,
            attack_coeff,
            release_coeff,
            makeup_gain_lin,
            env_db: -96.0,
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, sample: f32) -> f32 {
        let abs_x = sample.abs().max(1e-6);
        let input_db = 20.0 * abs_x.log10();
        let over_db = (input_db - self.threshold_db).max(0.0);

        let coeff = if over_db > self.env_db {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.env_db = over_db + coeff * (self.env_db - over_db);

        let gain_reduction_db = -self.env_db * (1.0 - 1.0 / self.ratio);
        let gain_lin = 10.0_f32.powf(gain_reduction_db / 20.0);

        sample * gain_lin * self.makeup_gain_lin
    }
}

impl DspNode for Compressor {
    fn name(&self) -> &'static str {
        "Vocal Dynamic Compressor"
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
        self.env_db = -96.0;
    }
}

#[derive(Debug, Clone)]
pub struct Limiter {
    enabled: bool,
    ceiling_lin: f32,
    release_coeff: f32,
    gain: f32,
}

impl Limiter {
    pub fn new(ceiling_db: f32, release_ms: f32, sample_rate: f32) -> Self {
        Self {
            enabled: true,
            ceiling_lin: 10.0_f32.powf(ceiling_db / 20.0),
            release_coeff: (-1.0 / (0.001 * release_ms * sample_rate)).exp(),
            gain: 1.0,
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, sample: f32) -> f32 {
        let abs_x = sample.abs();
        let target_gain = if abs_x > self.ceiling_lin {
            self.ceiling_lin / abs_x
        } else {
            1.0
        };

        if target_gain < self.gain {
            self.gain = target_gain;
        } else {
            self.gain = target_gain + self.release_coeff * (self.gain - target_gain);
        }

        (sample * self.gain).clamp(-self.ceiling_lin, self.ceiling_lin)
    }
}

impl DspNode for Limiter {
    fn name(&self) -> &'static str {
        "Brickwall Peak Limiter"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn process_frame(&mut self, samples: &mut [f32], ctx: &mut ProcessContext) {
        if !self.enabled {
            return;
        }
        let mut peak = 0.0_f32;
        for s in samples.iter_mut() {
            let out = self.process_sample(*s);
            let abs_out = out.abs();
            if abs_out > peak {
                peak = abs_out;
            }
            *s = out;
        }
        ctx.peak_level = peak;
    }

    fn reset(&mut self) {
        self.gain = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressor_reduces_loud_transients_and_applies_makeup() {
        let sr = 48_000.0;
        let mut comp = Compressor::new(-20.0, 4.0, 1.0, 50.0, 0.0, sr);
        let mut ctx = ProcessContext::new(sr);

        let mut loud_frame = vec![0.8f32; 480];
        comp.process_frame(&mut loud_frame, &mut ctx);

        assert!(
            loud_frame.last().unwrap().abs() < 0.5,
            "Compressor should attenuate signal above -20 dB threshold"
        );
    }

    #[test]
    fn test_brickwall_limiter_prevents_clipping() {
        let sr = 48_000.0;
        let ceiling_db = -1.0;
        let ceiling_lin = 10.0_f32.powf(ceiling_db / 20.0);
        let mut limiter = Limiter::new(ceiling_db, 50.0, sr);
        let mut ctx = ProcessContext::new(sr);

        let mut hot_signal = vec![1.5, -1.8, 0.2, 2.0, -0.95, 0.1];
        limiter.process_frame(&mut hot_signal, &mut ctx);

        for &s in &hot_signal {
            assert!(
                s.abs() <= ceiling_lin + 1e-5,
                "Sample {} exceeded brickwall ceiling {}",
                s,
                ceiling_lin
            );
        }
        assert!(ctx.peak_level <= ceiling_lin + 1e-5);
    }
}
