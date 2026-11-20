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
        self.profiles
            .iter()
            .find(|p| p.id.to_lowercase() == lower)
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
