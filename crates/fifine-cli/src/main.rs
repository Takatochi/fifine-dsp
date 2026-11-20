use anyhow::Result;
use cpal::traits::DeviceTrait;
use fifine_domain::{DeviceDiscoveryPort, InMemoryProfileRegistry, ProfileRepositoryPort};
use fifine_dsp::DspPipeline;
use fifine_engine::AudioEngine;
use fifine_hal::{CpalDeviceDiscovery, VIRTUAL_OUT_KEYWORDS};
use std::env;
use std::io::{self, Write};
use std::sync::atomic::Ordering;
use std::time::Duration;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let profile_repo = InMemoryProfileRegistry::new();
    let discovery = CpalDeviceDiscovery::new();

    if args.iter().any(|a| a == "--profiles" || a == "-p") {
        for p in profile_repo.list_profiles() {
            println!(
                "{:<6} | {:<38} | {:?} | HPF: {} Hz | Gate: {:2.0}%",
                p.id,
                p.display_name,
                p.capsule,
                p.default_preset.hpf_cutoff_hz,
                p.default_preset.vad_gate_threshold * 100.0
            );
        }
        return Ok(());
    }

    if args.iter().any(|a| a == "--list" || a == "-l") {
        for dev in discovery.list_endpoints().map_err(|e| anyhow::anyhow!(e))? {
            let dir = if dev.is_input { "IN " } else { "OUT" };
            let def = if dev.is_default { " *" } else { "" };
            println!("[{}] {}{}", dir, dev.name, def);
        }
        return Ok(());
    }

    let requested_profile = get_arg_value(&args, "--profile");
    let custom_in = get_arg_value(&args, "--in");
    let (input_device, active_profile) = discovery.auto_detect_microphone(
        &profile_repo,
        requested_profile.as_deref(),
        custom_in.as_deref(),
    )?;

    let custom_out = get_arg_value(&args, "--out");
    let output_device = if let Some(ref query) = custom_out {
        discovery.find_cpal_device(false, &[query.as_str()], false)?
    } else {
        discovery.find_cpal_device(false, VIRTUAL_OUT_KEYWORDS, true)?
    };

    let mut preset = active_profile.default_preset.clone();
    if args.iter().any(|a| a == "--no-rnnoise") {
        preset.enable_ai_denoise = false;
    }
    let pipeline = DspPipeline::from_preset(&preset);

    println!(
        "Profile : {} ({})",
        active_profile.display_name, active_profile.id
    );
    println!("Input   : {}", input_device.name().unwrap_or_default());
    println!("Output  : {}", output_device.name().unwrap_or_default());

    let engine = AudioEngine::start_with_pipeline(input_device, output_device, pipeline)?;

    loop {
        std::thread::sleep(Duration::from_millis(200));
        let vad = engine.telemetry.vad_permille.load(Ordering::Relaxed) as f32 / 10.0;
        let peak = engine.telemetry.peak_permille.load(Ordering::Relaxed) as f32 / 10.0;
        let underruns = engine.telemetry.underruns.load(Ordering::Relaxed);
        let overruns = engine.telemetry.overruns.load(Ordering::Relaxed);
        let drift_drops = engine.telemetry.drift_drops.load(Ordering::Relaxed);

        print!(
            "\rVAD: {:5.1}% | Peak: {:5.1}% | Underrun: {} | Overrun: {} | Drift: {}   ",
            vad, peak, underruns, overruns, drift_drops
        );
        let _ = io::stdout().flush();
    }
}

fn get_arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}
