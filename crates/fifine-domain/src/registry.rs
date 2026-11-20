use crate::entities::MicrophoneProfile;
use crate::ports::ProfileRepositoryPort;
use crate::profiles::BUILTIN_PROFILES;

pub struct InMemoryProfileRegistry {
    profiles: Vec<MicrophoneProfile>,
}

impl Default for InMemoryProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryProfileRegistry {
    pub fn new() -> Self {
        Self {
            profiles: BUILTIN_PROFILES.to_vec(),
        }
    }

    pub fn register_profile(&mut self, profile: MicrophoneProfile) {
        self.profiles.push(profile);
    }
}

impl ProfileRepositoryPort for InMemoryProfileRegistry {
    fn list_profiles(&self) -> &[MicrophoneProfile] {
        &self.profiles
    }

    fn find_by_id(&self, id: &str) -> Option<&MicrophoneProfile> {
        let lower = id.to_lowercase();
        self.profiles.iter().find(|p| p.id.to_lowercase() == lower)
    }

    fn detect_by_device_name(&self, os_device_name: &str) -> Option<&MicrophoneProfile> {
        let lower = os_device_name.to_lowercase();
        for profile in &self.profiles {
            if lower.contains(profile.id) {
                return Some(profile);
            }
        }
        self.profiles.iter().find(|profile| {
            profile
                .os_match_keywords
                .iter()
                .any(|kw| lower.contains(kw))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{
        CapsuleType, DspPresetSpec, EqBandSpec, MicrophoneProfile, ProcessContext,
    };

    #[test]
    fn test_builtin_profiles_exist() {
        let reg = InMemoryProfileRegistry::new();
        let profiles = reg.list_profiles();
        assert!(profiles.len() >= 4);
        assert!(reg.find_by_id("k689").is_some());
        assert!(reg.find_by_id("K689").is_some());
        assert!(reg.find_by_id("k688").is_some());
        assert!(reg.find_by_id("am8").is_some());
        assert!(reg.find_by_id("k669").is_some());
        assert!(reg.find_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_capsule_types_match_hardware() {
        let reg = InMemoryProfileRegistry::new();
        assert_eq!(
            reg.find_by_id("k689").unwrap().capsule,
            CapsuleType::Condenser
        );
        assert_eq!(
            reg.find_by_id("k669").unwrap().capsule,
            CapsuleType::Condenser
        );
        assert_eq!(
            reg.find_by_id("k688").unwrap().capsule,
            CapsuleType::Dynamic
        );
        assert_eq!(reg.find_by_id("am8").unwrap().capsule, CapsuleType::Dynamic);
    }

    #[test]
    fn test_detect_by_device_name() {
        let reg = InMemoryProfileRegistry::new();

        let p_k689 = reg
            .detect_by_device_name("Microphone (FIFINE K689 Microphone)")
            .unwrap();
        assert_eq!(p_k689.id, "k689");

        let p_k688 = reg
            .detect_by_device_name("FIFINE K688 Dynamic Microphone")
            .unwrap();
        assert_eq!(p_k688.id, "k688");

        let p_am8 = reg.detect_by_device_name("FIFINE AmpliGame AM8").unwrap();
        assert_eq!(p_am8.id, "am8");

        let p_generic = reg
            .detect_by_device_name("Microphone (FIFINE Microphone)")
            .unwrap();
        assert_eq!(p_generic.id, "k689");

        assert!(reg
            .detect_by_device_name("Realtek HD Audio Output")
            .is_none());
    }

    #[test]
    fn test_register_custom_profile_di() {
        let mut reg = InMemoryProfileRegistry::new();
        let initial_count = reg.list_profiles().len();

        let custom = MicrophoneProfile {
            id: "tank3",
            display_name: "FIFINE Tank3 Dynamic",
            capsule: CapsuleType::Dynamic,
            usb_ids: &[],
            os_match_keywords: &["tank3"],
            default_preset: DspPresetSpec {
                sample_rate: 48_000,
                enable_ai_denoise: true,
                vad_gate_threshold: 0.25,
                hpf_cutoff_hz: 80.0,
                low_mid_eq: EqBandSpec {
                    freq_hz: 250.0,
                    q: 1.2,
                    gain_db: -2.0,
                },
                presence_eq: EqBandSpec {
                    freq_hz: 3500.0,
                    q: 1.0,
                    gain_db: 2.0,
                },
                high_shelf_eq: EqBandSpec {
                    freq_hz: 10000.0,
                    q: 0.707,
                    gain_db: 1.5,
                },
                comp_threshold_db: -18.0,
                comp_ratio: 3.0,
                comp_attack_ms: 5.0,
                comp_release_ms: 80.0,
                comp_makeup_db: 3.0,
                limiter_ceiling_db: -0.5,
            },
        };

        reg.register_profile(custom);
        assert_eq!(reg.list_profiles().len(), initial_count + 1);
        assert_eq!(
            reg.detect_by_device_name("FIFINE Tank3 USB").unwrap().id,
            "tank3"
        );
    }

    #[test]
    fn test_process_context_initialization() {
        let ctx = ProcessContext::new(48_000.0);
        assert_eq!(ctx.sample_rate, 48_000.0);
        assert_eq!(ctx.vad_probability, 1.0);
        assert_eq!(ctx.peak_level, 0.0);
    }
}
