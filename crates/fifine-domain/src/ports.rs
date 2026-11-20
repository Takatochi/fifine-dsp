use crate::entities::{MicrophoneProfile, ProcessContext};

pub trait DspNode: Send {
    fn name(&self) -> &'static str;

    fn is_enabled(&self) -> bool {
        true
    }

    fn set_enabled(&mut self, _enabled: bool) {}

    fn process_frame(&mut self, samples: &mut [f32], ctx: &mut ProcessContext);

    fn reset(&mut self) {}
}

pub trait ProfileRepositoryPort: Send + Sync {
    fn list_profiles(&self) -> &[MicrophoneProfile];

    fn find_by_id(&self, id: &str) -> Option<&MicrophoneProfile>;

    fn detect_by_device_name(&self, os_device_name: &str) -> Option<&MicrophoneProfile>;
}

#[derive(Debug, Clone)]
pub struct AudioEndpointDescriptor {
    pub name: String,
    pub is_input: bool,
    pub is_default: bool,
}

pub trait DeviceDiscoveryPort {
    fn list_endpoints(&self) -> Result<Vec<AudioEndpointDescriptor>, String>;
}
