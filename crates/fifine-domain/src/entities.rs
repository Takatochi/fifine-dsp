#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapsuleType {
    Condenser,
    Dynamic,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbId {
    pub vid: u16,
    pub pid: Option<u16>,
}

#[derive(Debug, Clone, Copy)]
pub struct EqBandSpec {
    pub freq_hz: f32,
    pub q: f32,
    pub gain_db: f32,
}

#[derive(Debug, Clone)]
pub struct DspPresetSpec {
    pub sample_rate: u32,
    pub enable_ai_denoise: bool,
    pub vad_gate_threshold: f32,
    pub hpf_cutoff_hz: f32,
    pub low_mid_eq: EqBandSpec,
    pub presence_eq: EqBandSpec,
    pub high_shelf_eq: EqBandSpec,
    pub comp_threshold_db: f32,
    pub comp_ratio: f32,
    pub comp_attack_ms: f32,
    pub comp_release_ms: f32,
    pub comp_makeup_db: f32,
    pub limiter_ceiling_db: f32,
}

#[derive(Debug, Clone)]
pub struct MicrophoneProfile {
    pub id: &'static str,
    pub display_name: &'static str,
    pub capsule: CapsuleType,
    pub usb_ids: &'static [UsbId],
    pub os_match_keywords: &'static [&'static str],
    pub default_preset: DspPresetSpec,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessContext {
    pub sample_rate: f32,
    pub vad_probability: f32,
    pub peak_level: f32,
}

impl ProcessContext {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            vad_probability: 1.0,
            peak_level: 0.0,
        }
    }
}
