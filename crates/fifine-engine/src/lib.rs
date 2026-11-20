use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use fifine_dsp::{DspPipeline, RNNOISE_FRAME_SIZE};
use ringbuf::{
    traits::{Consumer, Observer, Producer, Split},
    HeapRb,
};
use std::sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc,
};

pub const TARGET_SAMPLE_RATE: u32 = 48_000;

#[derive(Default)]
pub struct EngineTelemetry {
    pub underruns: AtomicUsize,
    pub overruns: AtomicUsize,
    pub drift_drops: AtomicUsize,
    pub vad_permille: AtomicU32,
    pub peak_permille: AtomicU32,
}

pub struct AudioEngine {
    _input_stream: Stream,
    _output_stream: Stream,
    pub telemetry: Arc<EngineTelemetry>,
    pub input_name: String,
    pub output_name: String,
}

impl AudioEngine {
    pub fn start_with_pipeline(
        input_device: cpal::Device,
        output_device: cpal::Device,
        mut pipeline: DspPipeline,
    ) -> Result<Self> {
        let input_name = input_device
            .name()
            .unwrap_or_else(|_| "Unknown Input".into());
        let output_name = output_device
            .name()
            .unwrap_or_else(|_| "Unknown Output".into());

        let in_config = get_48k_config(&input_device, true)?;
        let out_config = get_48k_config(&output_device, false)?;

        let in_channels = in_config.channels as usize;
        let out_channels = out_config.channels as usize;

        let rb = HeapRb::<f32>::new(RNNOISE_FRAME_SIZE * 10);
        let (mut producer, mut consumer) = rb.split();

        for _ in 0..(RNNOISE_FRAME_SIZE * 2) {
            let _ = producer.try_push(0.0);
        }

        let telemetry = Arc::new(EngineTelemetry::default());
        let tel_in = Arc::clone(&telemetry);
        let tel_out = Arc::clone(&telemetry);

        let input_stream = input_device.build_input_stream(
            &in_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for frame in data.chunks_exact(in_channels) {
                    let mono_sample = frame[0];
                    pipeline.push_sample(mono_sample, |processed_frame, ctx| {
                        tel_in
                            .vad_permille
                            .store((ctx.vad_probability * 1000.0) as u32, Ordering::Relaxed);
                        tel_in
                            .peak_permille
                            .store((ctx.peak_level * 1000.0) as u32, Ordering::Relaxed);

                        for &sample in processed_frame.iter() {
                            if producer.try_push(sample).is_err() {
                                tel_in.overruns.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    });
                }
            },
            move |err| eprintln!("[AudioEngine] Input stream error: {}", err),
            None,
        )?;

        let mut last_sample = 0.0f32;
        let output_stream = output_device.build_output_stream(
            &out_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if consumer.occupied_len() > RNNOISE_FRAME_SIZE * 4 {
                    consumer.skip(RNNOISE_FRAME_SIZE);
                    tel_out.drift_drops.fetch_add(1, Ordering::Relaxed);
                }

                for frame in data.chunks_exact_mut(out_channels) {
                    let sample = match consumer.try_pop() {
                        Some(s) => {
                            last_sample = s;
                            s
                        }
                        None => {
                            tel_out.underruns.fetch_add(1, Ordering::Relaxed);
                            last_sample *= 0.95;
                            last_sample
                        }
                    };

                    for ch in frame.iter_mut() {
                        *ch = sample;
                    }
                }
            },
            move |err| eprintln!("[AudioEngine] Output stream error: {}", err),
            None,
        )?;

        input_stream.play()?;
        output_stream.play()?;

        Ok(Self {
            _input_stream: input_stream,
            _output_stream: output_stream,
            telemetry,
            input_name,
            output_name,
        })
    }
}

pub fn get_48k_config(device: &cpal::Device, is_input: bool) -> Result<StreamConfig> {
    let supported_configs: Vec<cpal::SupportedStreamConfigRange> = if is_input {
        device.supported_input_configs()?.collect()
    } else {
        device.supported_output_configs()?.collect()
    };

    for range in supported_configs {
        if range.sample_format() == SampleFormat::F32
            && range.min_sample_rate().0 <= TARGET_SAMPLE_RATE
            && range.max_sample_rate().0 >= TARGET_SAMPLE_RATE
        {
            return Ok(range
                .with_sample_rate(cpal::SampleRate(TARGET_SAMPLE_RATE))
                .config());
        }
    }

    let default_cfg = if is_input {
        device.default_input_config()?
    } else {
        device.default_output_config()?
    };

    if default_cfg.sample_rate().0 != TARGET_SAMPLE_RATE {
        return Err(anyhow!(
            "Device '{}' does not support 48000 Hz (current: {} Hz).",
            device.name().unwrap_or_default(),
            default_cfg.sample_rate().0
        ));
    }

    Ok(default_cfg.into())
}
