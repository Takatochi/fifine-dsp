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
