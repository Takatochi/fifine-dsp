use fifine_domain::{DspNode, ProcessContext};

pub struct VadNoiseGate {
    vad_threshold: f32,
    floor_gain: f32,
    attack_coeff: f32,
    release_coeff: f32,
    current_gain: f32,
    enabled: bool,
}

impl VadNoiseGate {
    pub fn new(
        vad_threshold: f32,
        floor_db: f32,
        attack_ms: f32,
        release_ms: f32,
        sample_rate: f32,
    ) -> Self {
        Self {
            vad_threshold,
            floor_gain: 10.0_f32.powf(floor_db / 20.0),
            attack_coeff: (-1.0 / (0.001 * attack_ms * sample_rate)).exp(),
            release_coeff: (-1.0 / (0.001 * release_ms * sample_rate)).exp(),
            current_gain: 1.0,
            enabled: vad_threshold > 0.01,
        }
    }
}

impl DspNode for VadNoiseGate {
    fn name(&self) -> &'static str {
        "VAD Smooth Noise Gate"
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

        let target_gain = if ctx.vad_probability >= self.vad_threshold {
            1.0
        } else {
            self.floor_gain
        };

        for sample in samples.iter_mut() {
            let coeff = if target_gain > self.current_gain {
                self.attack_coeff
            } else {
                self.release_coeff
            };
            self.current_gain = target_gain + coeff * (self.current_gain - target_gain);
            *sample *= self.current_gain;
        }
    }

    fn reset(&mut self) {
        self.current_gain = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_gate_attenuates_when_vad_below_threshold() {
        let sr = 48_000.0;
        let mut gate = VadNoiseGate::new(0.40, -30.0, 1.0, 5.0, sr);
        let mut ctx = ProcessContext::new(sr);
        ctx.vad_probability = 0.05;

        let mut noise = vec![0.1f32; 2400];
        gate.process_frame(&mut noise, &mut ctx);

        assert!(
            noise.last().unwrap().abs() < 0.01,
            "Noise floor should be attenuated when VAD is low"
        );

        ctx.vad_probability = 0.95;
        let mut voice = vec![0.5f32; 2400];
        gate.process_frame(&mut voice, &mut ctx);
        assert!(
            (voice.last().unwrap() - 0.5).abs() < 0.01,
            "Voice should pass at unity gain when VAD is high"
        );
    }
}
