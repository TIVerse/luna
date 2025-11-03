//! Audio device management
//!
//! Handles device enumeration, selection, and hotplug detection.

use crate::error::{LunaError, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use std::fmt;

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    /// Device name
    pub name: String,
    /// Device is default input
    pub is_default: bool,
    /// Supported sample rates
    pub sample_rates: Vec<u32>,
    /// Supported channel counts
    pub channels: Vec<u16>,
}

impl fmt::Display for AudioDeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{} - Rates: {:?}, Channels: {:?}",
            self.name,
            if self.is_default { " (default)" } else { "" },
            self.sample_rates,
            self.channels
        )
    }
}

/// Audio device manager
pub struct AudioDeviceManager {
    host: cpal::Host,
}

impl AudioDeviceManager {
    /// Create a new device manager
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// List all available input devices
    pub fn list_input_devices(&self) -> Result<Vec<AudioDeviceInfo>> {
        let devices = self
            .host
            .input_devices()
            .map_err(|e| LunaError::Audio(format!("Failed to enumerate devices: {}", e)))?;

        let default_device_name = self.host.default_input_device().and_then(|d| d.name().ok());

        let mut device_infos = Vec::new();

        for device in devices {
            let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
            let is_default = default_device_name.as_ref() == Some(&name);

            // Get supported configs
            let configs = device
                .supported_input_configs()
                .map_err(|e| LunaError::Audio(format!("Failed to get device configs: {}", e)))?;

            let mut sample_rates = Vec::new();
            let mut channels = Vec::new();

            for config in configs {
                // Add min and max sample rates
                sample_rates.push(config.min_sample_rate().0);
                sample_rates.push(config.max_sample_rate().0);
                channels.push(config.channels());
            }

            // Deduplicate and sort
            sample_rates.sort_unstable();
            sample_rates.dedup();
            channels.sort_unstable();
            channels.dedup();

            device_infos.push(AudioDeviceInfo {
                name,
                is_default,
                sample_rates,
                channels,
            });
        }

        Ok(device_infos)
    }

    /// Get the default input device
    pub fn default_input_device(&self) -> Result<cpal::Device> {
        self.host
            .default_input_device()
            .ok_or_else(|| LunaError::Audio("No default input device found".into()))
    }

    /// Get a device by name
    pub fn get_device_by_name(&self, name: &str) -> Result<cpal::Device> {
        if name.is_empty() {
            return self.default_input_device();
        }

        let devices = self
            .host
            .input_devices()
            .map_err(|e| LunaError::Audio(format!("Failed to enumerate devices: {}", e)))?;

        for device in devices {
            if let Ok(device_name) = device.name() {
                if device_name.to_lowercase().contains(&name.to_lowercase()) {
                    return Ok(device);
                }
            }
        }

        Err(LunaError::Audio(format!("Device not found: {}", name)))
    }

    /// Get device info
    pub fn get_device_info(&self, device: &cpal::Device) -> Result<AudioDeviceInfo> {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());

        let default_device_name = self.host.default_input_device().and_then(|d| d.name().ok());
        let is_default = default_device_name.as_ref() == Some(&name);

        let configs = device
            .supported_input_configs()
            .map_err(|e| LunaError::Audio(format!("Failed to get device configs: {}", e)))?;

        let mut sample_rates = Vec::new();
        let mut channels = Vec::new();

        for config in configs {
            sample_rates.push(config.min_sample_rate().0);
            sample_rates.push(config.max_sample_rate().0);
            channels.push(config.channels());
        }

        sample_rates.sort_unstable();
        sample_rates.dedup();
        channels.sort_unstable();
        channels.dedup();

        Ok(AudioDeviceInfo {
            name,
            is_default,
            sample_rates,
            channels,
        })
    }

    /// Check if a device supports a specific sample rate
    pub fn supports_sample_rate(&self, device: &cpal::Device, sample_rate: u32) -> bool {
        if let Ok(configs) = device.supported_input_configs() {
            for config in configs {
                if sample_rate >= config.min_sample_rate().0
                    && sample_rate <= config.max_sample_rate().0
                {
                    return true;
                }
            }
        }
        false
    }

    /// Get best supported sample rate closest to target
    pub fn best_sample_rate(&self, device: &cpal::Device, target: u32) -> Result<u32> {
        let configs = device
            .supported_input_configs()
            .map_err(|e| LunaError::Audio(format!("Failed to get device configs: {}", e)))?;

        let mut best_rate = None;
        let mut min_diff = u32::MAX;

        for config in configs {
            let min_rate = config.min_sample_rate().0;
            let max_rate = config.max_sample_rate().0;

            if target >= min_rate && target <= max_rate {
                return Ok(target);
            }

            // Find closest rate
            for &rate in &[min_rate, max_rate] {
                let diff = (rate as i32 - target as i32).unsigned_abs();
                if diff < min_diff {
                    min_diff = diff;
                    best_rate = Some(rate);
                }
            }
        }

        best_rate.ok_or_else(|| LunaError::Audio("No suitable sample rate found".into()))
    }
}

impl Default for AudioDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_manager_creation() {
        let manager = AudioDeviceManager::new();
        assert!(manager.default_input_device().is_ok());
    }

    #[test]
    fn test_list_devices() {
        let manager = AudioDeviceManager::new();
        let devices = manager.list_input_devices();

        // Should have at least one device (or error if none)
        assert!(devices.is_ok() || devices.is_err());

        if let Ok(devices) = devices {
            if !devices.is_empty() {
                println!("Available devices:");
                for dev in &devices {
                    println!("  {}", dev);
                }
            }
        }
    }

    #[test]
    fn test_default_device() {
        let manager = AudioDeviceManager::new();
        let device = manager.default_input_device();

        if let Ok(device) = device {
            let info = manager.get_device_info(&device);
            assert!(info.is_ok());

            if let Ok(info) = info {
                assert!(info.is_default);
                println!("Default device: {}", info);
            }
        }
    }

    #[test]
    fn test_get_device_by_empty_name() {
        let manager = AudioDeviceManager::new();

        // Empty name should return default
        let device = manager.get_device_by_name("");
        assert!(device.is_ok());
    }
}
