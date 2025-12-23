/// Integration tests for P1-T1: Audio Device Enumeration
///
/// This test file validates that the audio device enumeration functionality
/// works correctly in an integration test context.

use raflow_lib::audio::{get_default_input_device, get_device_config, list_input_devices};

#[test]
fn test_list_input_devices_integration() {
    let result = list_input_devices();

    match result {
        Ok(devices) => {
            println!("\n=== Audio Input Devices ===");
            println!("Found {} input device(s):\n", devices.len());

            for (idx, device) in devices.iter().enumerate() {
                println!("Device {}:", idx + 1);
                println!("  Name: {}", device.name);
                println!("  ID: {}", device.id);
                println!("  Default: {}", device.is_default);
                println!("  Sample Rates: {:?}", device.sample_rates);
                println!();
            }

            // Verify at least one device exists
            assert!(
                !devices.is_empty(),
                "Should have at least one input device"
            );

            // Verify at least one device is marked as default
            let has_default = devices.iter().any(|d| d.is_default);
            assert!(has_default, "Should have a default device");

            // Verify all devices have valid properties
            for device in &devices {
                assert!(!device.name.is_empty(), "Device name should not be empty");
                assert!(!device.id.is_empty(), "Device ID should not be empty");
                assert!(
                    !device.sample_rates.is_empty(),
                    "Device should support at least one sample rate"
                );
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not list input devices: {}", e);
            eprintln!("This may be expected in CI environments without audio hardware");
        }
    }
}

#[test]
fn test_default_device_integration() {
    let result = get_default_input_device();

    match result {
        Ok(device) => {
            println!("\n=== Default Input Device ===");
            println!("Name: {}", device.name);
            println!("ID: {}", device.id);
            println!("Sample Rates: {:?}", device.sample_rates);
            println!();

            assert!(!device.name.is_empty());
            assert!(device.is_default);
            assert!(!device.sample_rates.is_empty());

            // Verify we can get the config for this device
            let config_result = get_device_config(&device.id);
            match config_result {
                Ok(config) => {
                    println!("Device Configuration:");
                    println!("  Channels: {}", config.channels);
                    println!("  Sample Rate: {}", config.sample_rate.0);
                    println!();

                    assert!(config.channels > 0);
                    assert!(config.sample_rate.0 > 0);
                }
                Err(e) => {
                    eprintln!("Warning: Could not get device config: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not get default input device: {}", e);
            eprintln!("This may be expected in CI environments without audio hardware");
        }
    }
}

#[test]
fn test_device_config_error_handling() {
    // Test with a non-existent device
    let result = get_device_config("NonExistentDevice_12345");

    assert!(result.is_err(), "Should return an error for non-existent device");

    println!("\n=== Error Handling Test ===");
    if let Err(e) = result {
        println!("Expected error for non-existent device: {}", e);
    }
}

#[test]
fn test_device_sample_rate_validity() {
    let result = list_input_devices();

    if let Ok(devices) = result {
        for device in devices {
            println!("\n=== Validating Sample Rates for {} ===", device.name);

            // Check that sample rates are in a reasonable range
            for &rate in &device.sample_rates {
                println!("  Checking rate: {}", rate);
                assert!(
                    rate >= 8000 && rate <= 192000,
                    "Sample rate {} is outside reasonable range",
                    rate
                );

                // Check for common sample rates
                let common_rates = [8000, 16000, 22050, 32000, 44100, 48000, 96000, 192000];
                if common_rates.contains(&rate) {
                    println!("    ✓ Common sample rate");
                }
            }

            // Verify sample rates are sorted
            let mut sorted_rates = device.sample_rates.clone();
            sorted_rates.sort_unstable();
            assert_eq!(
                device.sample_rates, sorted_rates,
                "Sample rates should be sorted"
            );
        }
    }
}
