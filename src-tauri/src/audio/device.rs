use crate::audio::error::{AudioError, AudioResult};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, StreamConfig};
use serde::{Deserialize, Serialize};

/// Information about an audio device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Device identifier (unique name)
    pub id: String,
    /// Human-readable device name
    pub name: String,
    /// Whether this is the default input device
    pub is_default: bool,
    /// Supported sample rates
    pub sample_rates: Vec<u32>,
}

/// List all available input devices
///
/// # Returns
/// A vector of `AudioDevice` containing information about all available input devices.
///
/// # Errors
/// Returns `AudioError::DeviceNotFound` if no input devices are found.
/// Returns `AudioError::CpalError` if there's an error accessing devices.
///
/// # Example
/// ```no_run
/// use raflow_lib::audio::device::list_input_devices;
///
/// let devices = list_input_devices().unwrap();
/// for device in devices {
///     println!("Device: {} ({})", device.name, device.id);
/// }
/// ```
pub fn list_input_devices() -> AudioResult<Vec<AudioDevice>> {
    let host = cpal::default_host();
    let devices: Vec<Device> = host.input_devices()?.collect();

    if devices.is_empty() {
        return Err(AudioError::DeviceNotFound);
    }

    let default_device = host.default_input_device();
    let default_name = default_device.as_ref().and_then(|d| d.name().ok());

    let mut audio_devices = Vec::new();

    for device in devices {
        let name = device
            .name()
            .map_err(|_| AudioError::InvalidDeviceName)?;

        let is_default = default_name.as_ref().map_or(false, |dn| dn == &name);

        // Get supported sample rates
        let sample_rates = get_supported_sample_rates(&device);

        audio_devices.push(AudioDevice {
            id: name.clone(),
            name: name.clone(),
            is_default,
            sample_rates,
        });
    }

    Ok(audio_devices)
}

/// Get the default input device
///
/// # Returns
/// An `AudioDevice` representing the default input device.
///
/// # Errors
/// Returns `AudioError::DeviceNotFound` if no default input device is found.
///
/// # Example
/// ```no_run
/// use raflow_lib::audio::device::get_default_input_device;
///
/// let device = get_default_input_device().unwrap();
/// println!("Default device: {}", device.name);
/// ```
pub fn get_default_input_device() -> AudioResult<AudioDevice> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(AudioError::DeviceNotFound)?;

    let name = device
        .name()
        .map_err(|_| AudioError::InvalidDeviceName)?;

    let sample_rates = get_supported_sample_rates(&device);

    Ok(AudioDevice {
        id: name.clone(),
        name: name.clone(),
        is_default: true,
        sample_rates,
    })
}

/// Get the configuration for a specific device
///
/// # Arguments
/// * `device_id` - The device identifier (name)
///
/// # Returns
/// A `StreamConfig` containing the device's configuration.
///
/// # Errors
/// Returns `AudioError::DeviceNotFound` if the device is not found.
/// Returns `AudioError::DefaultConfigError` if unable to get the default config.
///
/// # Example
/// ```no_run
/// use raflow_lib::audio::device::get_device_config;
///
/// let config = get_device_config("Default Microphone").unwrap();
/// println!("Sample rate: {}", config.sample_rate.0);
/// ```
pub fn get_device_config(device_id: &str) -> AudioResult<StreamConfig> {
    let host = cpal::default_host();
    let device = find_device_by_id(&host, device_id)?;

    let config = device.default_input_config()?;

    Ok(StreamConfig {
        channels: config.channels(),
        sample_rate: config.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    })
}

/// Find a device by its ID (name)
pub(crate) fn find_device_by_id(host: &Host, device_id: &str) -> AudioResult<Device> {
    let devices: Vec<Device> = host
        .input_devices()
        .map_err(AudioError::CpalError)?
        .collect();

    for device in devices {
        if let Ok(name) = device.name() {
            if name == device_id {
                return Ok(device);
            }
        }
    }

    Err(AudioError::DeviceNotFound)
}

/// Get supported sample rates for a device
fn get_supported_sample_rates(device: &Device) -> Vec<u32> {
    let mut rates = Vec::new();

    // Try to get supported configurations
    if let Ok(configs) = device.supported_input_configs() {
        for config in configs {
            // Common sample rates to check
            let common_rates = [8000, 16000, 22050, 32000, 44100, 48000, 96000];

            for &rate in &common_rates {
                let sample_rate = cpal::SampleRate(rate);
                if sample_rate >= config.min_sample_rate()
                    && sample_rate <= config.max_sample_rate()
                {
                    if !rates.contains(&rate) {
                        rates.push(rate);
                    }
                }
            }
        }
    }

    // If we couldn't get any rates, try the default config
    if rates.is_empty() {
        if let Ok(config) = device.default_input_config() {
            rates.push(config.sample_rate().0);
        }
    }

    rates.sort_unstable();
    rates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() {
        let result = list_input_devices();
        match result {
            Ok(devices) => {
                assert!(!devices.is_empty(), "Should have at least one input device");
                for device in &devices {
                    assert!(!device.name.is_empty(), "Device name should not be empty");
                    assert!(!device.id.is_empty(), "Device ID should not be empty");
                    assert!(
                        !device.sample_rates.is_empty(),
                        "Device should support at least one sample rate"
                    );
                }
                println!("Found {} input devices", devices.len());
                for device in devices {
                    println!(
                        "  - {} (default: {}, rates: {:?})",
                        device.name, device.is_default, device.sample_rates
                    );
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not list devices: {}", e);
                // Don't fail the test if no devices are available (e.g., in CI)
            }
        }
    }

    #[test]
    fn test_default_device() {
        let result = get_default_input_device();
        match result {
            Ok(device) => {
                assert!(!device.name.is_empty(), "Default device name should not be empty");
                assert!(device.is_default, "Should be marked as default");
                assert!(
                    !device.sample_rates.is_empty(),
                    "Default device should support at least one sample rate"
                );
                println!("Default device: {}", device.name);
                println!("  Sample rates: {:?}", device.sample_rates);
            }
            Err(e) => {
                eprintln!("Warning: Could not get default device: {}", e);
                // Don't fail the test if no default device is available (e.g., in CI)
            }
        }
    }

    #[test]
    fn test_device_config() {
        // First get the default device
        if let Ok(device) = get_default_input_device() {
            let result = get_device_config(&device.id);
            match result {
                Ok(config) => {
                    assert!(config.channels > 0, "Should have at least one channel");
                    assert!(
                        config.sample_rate.0 > 0,
                        "Sample rate should be greater than 0"
                    );
                    println!("Device config:");
                    println!("  Channels: {}", config.channels);
                    println!("  Sample rate: {}", config.sample_rate.0);
                }
                Err(e) => {
                    eprintln!("Warning: Could not get device config: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_device_not_found() {
        let result = get_device_config("NonExistentDevice123456789");
        assert!(result.is_err());
        if let Err(AudioError::DeviceNotFound) = result {
            // Expected error
        } else {
            panic!("Expected DeviceNotFound error");
        }
    }
}
