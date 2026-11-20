use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use fifine_domain::{AudioEndpointDescriptor, DeviceDiscoveryPort, ProfileRepositoryPort};

pub const VIRTUAL_OUT_KEYWORDS: &[&str] = &[
    "cable input",
    "vb-audio",
    "virtual",
    "fifine_virtual",
    "voice meeter",
];

pub struct CpalDeviceDiscovery {
    host: cpal::Host,
}

impl Default for CpalDeviceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl CpalDeviceDiscovery {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    pub fn host(&self) -> &cpal::Host {
        &self.host
    }

    pub fn find_cpal_device(
        &self,
        is_input: bool,
        keywords: &[&str],
        fallback_to_default: bool,
    ) -> Result<cpal::Device> {
        let devices: Vec<cpal::Device> = if is_input {
            self.host.input_devices()?.collect()
        } else {
            self.host.output_devices()?.collect()
        };

        for dev in devices {
            if let Ok(name) = dev.name() {
                let lower = name.to_lowercase();
                if keywords.iter().any(|k| lower.contains(&k.to_lowercase())) {
                    return Ok(dev);
                }
            }
        }

        if fallback_to_default {
            if is_input {
                self.host
                    .default_input_device()
                    .context("Default input device not found")
            } else {
                self.host
                    .default_output_device()
                    .context("Default output device not found")
            }
        } else {
            Err(anyhow!("Device matching {:?} not found", keywords))
        }
    }

    pub fn auto_detect_microphone<'a>(
        &self,
        repo: &'a dyn ProfileRepositoryPort,
        requested_profile_id: Option<&str>,
        custom_device_query: Option<&str>,
    ) -> Result<(cpal::Device, &'a fifine_domain::MicrophoneProfile)> {
        let explicit_profile = if let Some(pid) = requested_profile_id {
            Some(
                repo.find_by_id(pid)
                    .ok_or_else(|| anyhow!("Profile '{}' not found", pid))?,
            )
        } else {
            None
        };

        if let Some(query) = custom_device_query {
            let dev = self.find_cpal_device(true, &[query], false)?;
            let dev_name = dev.name().unwrap_or_default();
            let profile = explicit_profile
                .or_else(|| repo.detect_by_device_name(&dev_name))
                .unwrap_or_else(|| &repo.list_profiles()[0]);
            return Ok((dev, profile));
        }

        if let Some(prof) = explicit_profile {
            let dev = self.find_cpal_device(true, prof.os_match_keywords, true)?;
            return Ok((dev, prof));
        }

        if let Ok(inputs) = self.host.input_devices() {
            for dev in inputs {
                if let Ok(name) = dev.name() {
                    if let Some(prof) = repo.detect_by_device_name(&name) {
                        return Ok((dev, prof));
                    }
                }
            }
        }

        let default_dev = self
            .host
            .default_input_device()
            .context("No input device available")?;
        let default_prof = &repo.list_profiles()[0];
        Ok((default_dev, default_prof))
    }
}

impl DeviceDiscoveryPort for CpalDeviceDiscovery {
    fn list_endpoints(&self) -> Result<Vec<AudioEndpointDescriptor>, String> {
        let mut list = Vec::new();
        let default_in = self.host.default_input_device().and_then(|d| d.name().ok());
        let default_out = self
            .host
            .default_output_device()
            .and_then(|d| d.name().ok());

        if let Ok(inputs) = self.host.input_devices() {
            for dev in inputs {
                if let Ok(name) = dev.name() {
                    let is_default = default_in.as_deref() == Some(name.as_str());
                    list.push(AudioEndpointDescriptor {
                        name,
                        is_input: true,
                        is_default,
                    });
                }
            }
        }

        if let Ok(outputs) = self.host.output_devices() {
            for dev in outputs {
                if let Ok(name) = dev.name() {
                    let is_default = default_out.as_deref() == Some(name.as_str());
                    list.push(AudioEndpointDescriptor {
                        name,
                        is_input: false,
                        is_default,
                    });
                }
            }
        }

        Ok(list)
    }
}
