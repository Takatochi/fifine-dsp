use fifine_domain::{DspNode, ProcessContext};
use nnnoiseless::DenoiseState;

pub const RNNOISE_FRAME_SIZE: usize = DenoiseState::FRAME_SIZE;

pub struct RnnoiseProcessor {
    state: Box<DenoiseState<'static>>,
    input_scaled: [f32; RNNOISE_FRAME_SIZE],
    output_scaled: [f32; RNNOISE_FRAME_SIZE],
    enabled: bool,
}

impl RnnoiseProcessor {
    pub fn new(enabled: bool) -> Self {
        Self {
            state: DenoiseState::new(),
            input_scaled: [0.0; RNNOISE_FRAME_SIZE],
            output_scaled: [0.0; RNNOISE_FRAME_SIZE],
            enabled,
        }
    }
}

impl DspNode for RnnoiseProcessor {
    fn name(&self) -> &'static str {
        "RNNoise Neural Denoiser"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn process_frame(&mut self, samples: &mut [f32], ctx: &mut ProcessContext) {
        if !self.enabled || samples.len() != RNNOISE_FRAME_SIZE {
            ctx.vad_probability = 1.0;
            return;
        }

        for (dst, &src) in self.input_scaled.iter_mut().zip(samples.iter()) {
            *dst = src * 32768.0;
        }

        let vad_prob = self
            .state
            .process_frame(&mut self.output_scaled, &self.input_scaled);

        ctx.vad_probability = vad_prob;

        const INV_SCALE: f32 = 1.0 / 32768.0;
        for (dst, &src) in samples.iter_mut().zip(self.output_scaled.iter()) {
            *dst = src * INV_SCALE;
        }
    }

    fn reset(&mut self) {
        self.state = DenoiseState::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rnnoise_frame_processing_and_amplitude_preservation() {
        let mut denoiser = RnnoiseProcessor::new(true);
        let mut ctx = ProcessContext::new(48_000.0);
        let mut frame = [0.0f32; RNNOISE_FRAME_SIZE];

        for (i, s) in frame.iter_mut().enumerate() {
            *s = 0.25 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48_000.0).sin();
        }

        denoiser.process_frame(&mut frame, &mut ctx);

        assert!((0.0..=1.0).contains(&ctx.vad_probability));
        assert!(frame.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
    }

    #[test]
    fn test_rnnoise_bypass_leaves_signal_unchanged() {
        let mut denoiser = RnnoiseProcessor::new(false);
        let mut ctx = ProcessContext::new(48_000.0);
        let orig = [0.42f32; RNNOISE_FRAME_SIZE];
        let mut frame = orig;

        denoiser.process_frame(&mut frame, &mut ctx);
        assert_eq!(frame, orig);
        assert_eq!(ctx.vad_probability, 1.0);
    }
}
