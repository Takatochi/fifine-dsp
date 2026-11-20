use crate::biquad::{BiquadFilter, FilterType};
use crate::denoise::{RnnoiseProcessor, RNNOISE_FRAME_SIZE};
use crate::dynamics::{Compressor, Limiter};
use crate::gate::VadNoiseGate;
use fifine_domain::{DspNode, DspPresetSpec, ProcessContext};

pub struct DspPipeline {
    nodes: Vec<Box<dyn DspNode>>,
    frame_buf: [f32; RNNOISE_FRAME_SIZE],
    frame_pos: usize,
    ctx: ProcessContext,
}

impl DspPipeline {
    pub fn new(sample_rate: f32, nodes: Vec<Box<dyn DspNode>>) -> Self {
        Self {
            nodes,
            frame_buf: [0.0; RNNOISE_FRAME_SIZE],
            frame_pos: 0,
            ctx: ProcessContext::new(sample_rate),
        }
    }

    pub fn from_preset(preset: &DspPresetSpec) -> Self {
        let sr = preset.sample_rate as f32;
        DspPipelineBuilder::new(sr)
            .with_node(BiquadFilter::with_label(
                "High-Pass Filter",
                FilterType::HighPass {
                    cutoff_hz: preset.hpf_cutoff_hz,
                    q: 0.70710678,
                },
                sr,
            ))
            .with_node(RnnoiseProcessor::new(preset.enable_ai_denoise))
            .with_node(VadNoiseGate::new(
                preset.vad_gate_threshold,
                -35.0,
                4.0,
                180.0,
                sr,
            ))
            .with_node(BiquadFilter::with_label(
                "Low-Mid Parametric EQ",
                FilterType::PeakingEq {
                    freq_hz: preset.low_mid_eq.freq_hz,
                    q: preset.low_mid_eq.q,
                    gain_db: preset.low_mid_eq.gain_db,
                },
                sr,
            ))
            .with_node(BiquadFilter::with_label(
                "Presence Parametric EQ",
                FilterType::PeakingEq {
                    freq_hz: preset.presence_eq.freq_hz,
                    q: preset.presence_eq.q,
                    gain_db: preset.presence_eq.gain_db,
                },
                sr,
            ))
            .with_node(BiquadFilter::with_label(
                "High-Shelf / Sibilance Control",
                FilterType::HighShelf {
                    freq_hz: preset.high_shelf_eq.freq_hz,
                    q: preset.high_shelf_eq.q,
                    gain_db: preset.high_shelf_eq.gain_db,
                },
                sr,
            ))
            .with_node(Compressor::new(
                preset.comp_threshold_db,
                preset.comp_ratio,
                preset.comp_attack_ms,
                preset.comp_release_ms,
                preset.comp_makeup_db,
                sr,
            ))
            .with_node(Limiter::new(preset.limiter_ceiling_db, 50.0, sr))
            .build()
    }

    pub fn describe_nodes(&self) -> Vec<(&'static str, bool)> {
        self.nodes
            .iter()
            .map(|n| (n.name(), n.is_enabled()))
            .collect()
    }

    #[inline]
    pub fn push_sample<F>(&mut self, input_sample: f32, mut on_processed_frame: F)
    where
        F: FnMut(&[f32; RNNOISE_FRAME_SIZE], ProcessContext),
    {
        self.frame_buf[self.frame_pos] = input_sample;
        self.frame_pos += 1;

        if self.frame_pos == RNNOISE_FRAME_SIZE {
            self.frame_pos = 0;

            for node in self.nodes.iter_mut() {
                if node.is_enabled() {
                    node.process_frame(&mut self.frame_buf, &mut self.ctx);
                }
            }

            on_processed_frame(&self.frame_buf, self.ctx);
        }
    }
}

pub struct DspPipelineBuilder {
    sample_rate: f32,
    nodes: Vec<Box<dyn DspNode>>,
}

impl DspPipelineBuilder {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            nodes: Vec::with_capacity(8),
        }
    }

    pub fn with_node<N: DspNode + 'static>(mut self, node: N) -> Self {
        self.nodes.push(Box::new(node));
        self
    }

    pub fn build(self) -> DspPipeline {
        DspPipeline::new(self.sample_rate, self.nodes)
    }
}
