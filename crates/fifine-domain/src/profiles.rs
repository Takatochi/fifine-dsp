use crate::entities::{CapsuleType, DspPresetSpec, EqBandSpec, MicrophoneProfile, UsbId};

const COMMON_FIFINE_USB_IDS: &[UsbId] = &[
    UsbId {
        vid: 0x3142,
        pid: None,
    },
    UsbId {
        vid: 0x0C76,
        pid: None,
    },
    UsbId {
        vid: 0x0D8C,
        pid: None,
    },
    UsbId {
        vid: 0x1B3F,
        pid: None,
    },
];

pub static BUILTIN_PROFILES: &[MicrophoneProfile] = &[
    MicrophoneProfile {
        id: "k689",
        display_name: "FIFINE K689 (Condenser USB)",
        capsule: CapsuleType::Condenser,
        usb_ids: COMMON_FIFINE_USB_IDS,
        os_match_keywords: &["k689", "fifine", "usb pnp audio"],
        default_preset: DspPresetSpec {
            sample_rate: 48_000,
            enable_ai_denoise: true,
            vad_gate_threshold: 0.35,
            hpf_cutoff_hz: 85.0,
            low_mid_eq: EqBandSpec {
                freq_hz: 240.0,
                q: 1.2,
                gain_db: -2.0,
            },
            presence_eq: EqBandSpec {
                freq_hz: 3200.0,
                q: 1.0,
                gain_db: 1.5,
            },
            high_shelf_eq: EqBandSpec {
                freq_hz: 9500.0,
                q: 0.707,
                gain_db: 0.5,
            },
            comp_threshold_db: -20.0,
            comp_ratio: 3.0,
            comp_attack_ms: 4.0,
            comp_release_ms: 85.0,
            comp_makeup_db: 3.0,
            limiter_ceiling_db: -0.5,
        },
    },
    MicrophoneProfile {
        id: "k688",
        display_name: "FIFINE K688 (Dynamic Podcast USB/XLR)",
        capsule: CapsuleType::Dynamic,
        usb_ids: COMMON_FIFINE_USB_IDS,
        os_match_keywords: &["k688"],
        default_preset: DspPresetSpec {
            sample_rate: 48_000,
            enable_ai_denoise: true,
            vad_gate_threshold: 0.20,
            hpf_cutoff_hz: 75.0,
            low_mid_eq: EqBandSpec {
                freq_hz: 300.0,
                q: 1.4,
                gain_db: -3.0,
            },
            presence_eq: EqBandSpec {
                freq_hz: 3800.0,
                q: 1.0,
                gain_db: 2.5,
            },
            high_shelf_eq: EqBandSpec {
                freq_hz: 10_000.0,
                q: 0.707,
                gain_db: 2.0,
            },
            comp_threshold_db: -18.0,
            comp_ratio: 3.5,
            comp_attack_ms: 6.0,
            comp_release_ms: 70.0,
            comp_makeup_db: 4.0,
            limiter_ceiling_db: -0.5,
        },
    },
    MicrophoneProfile {
        id: "am8",
        display_name: "FIFINE AmpliGame AM8 (Dynamic USB/XLR)",
        capsule: CapsuleType::Dynamic,
        usb_ids: COMMON_FIFINE_USB_IDS,
        os_match_keywords: &["am8", "ampligame"],
        default_preset: DspPresetSpec {
            sample_rate: 48_000,
            enable_ai_denoise: true,
            vad_gate_threshold: 0.25,
            hpf_cutoff_hz: 80.0,
            low_mid_eq: EqBandSpec {
                freq_hz: 260.0,
                q: 1.3,
                gain_db: -2.5,
            },
            presence_eq: EqBandSpec {
                freq_hz: 3500.0,
                q: 1.1,
                gain_db: 2.0,
            },
            high_shelf_eq: EqBandSpec {
                freq_hz: 10_500.0,
                q: 0.707,
                gain_db: 1.8,
            },
            comp_threshold_db: -19.0,
            comp_ratio: 3.2,
            comp_attack_ms: 5.0,
            comp_release_ms: 75.0,
            comp_makeup_db: 3.5,
            limiter_ceiling_db: -0.5,
        },
    },
    MicrophoneProfile {
        id: "k669",
        display_name: "FIFINE K669/K669B (Condenser USB)",
        capsule: CapsuleType::Condenser,
        usb_ids: COMMON_FIFINE_USB_IDS,
        os_match_keywords: &["k669"],
        default_preset: DspPresetSpec {
            sample_rate: 48_000,
            enable_ai_denoise: true,
            vad_gate_threshold: 0.40,
            hpf_cutoff_hz: 90.0,
            low_mid_eq: EqBandSpec {
                freq_hz: 250.0,
                q: 1.2,
                gain_db: -1.5,
            },
            presence_eq: EqBandSpec {
                freq_hz: 3000.0,
                q: 1.0,
                gain_db: 1.0,
            },
            high_shelf_eq: EqBandSpec {
                freq_hz: 9000.0,
                q: 0.707,
                gain_db: -1.0,
            },
            comp_threshold_db: -21.0,
            comp_ratio: 3.0,
            comp_attack_ms: 3.5,
            comp_release_ms: 90.0,
            comp_makeup_db: 3.0,
            limiter_ceiling_db: -0.5,
        },
    },
];
