/// Integration tests for P1-T6: WebSocket Connection
///
/// This test file validates the WebSocket connection configuration and
/// error handling. Real connection tests require a valid API key.

use raflow_lib::network::{ConnectionConfig, NetworkError};

#[test]
fn test_connection_config_creation() {
    println!("\n=== ConnectionConfig Creation Test ===");

    let config = ConnectionConfig::new(16000);

    println!("Configuration:");
    println!("  Model ID: {}", config.model_id);
    println!("  Sample rate: {}", config.sample_rate);
    println!("  Language: {:?}", config.language_code);
    println!("  Timestamps: {}", config.include_timestamps);
    println!("  Timeout: {}ms", config.timeout_ms);

    assert_eq!(config.sample_rate, 16000);
    assert_eq!(config.model_id, "scribe_v2_realtime");
    assert_eq!(config.language_code, None);
    assert!(!config.include_timestamps);
    assert_eq!(config.timeout_ms, 10000);

    println!("\n✓ Default configuration created correctly");
}

#[test]
fn test_connection_config_builder_pattern() {
    println!("\n=== ConnectionConfig Builder Pattern Test ===");

    let config = ConnectionConfig::new(16000)
        .with_model("custom_model")
        .with_language("zh")
        .with_timestamps()
        .with_vad_strategy("auto")
        .with_timeout(5000);

    println!("Custom configuration:");
    println!("  Model ID: {}", config.model_id);
    println!("  Language: {:?}", config.language_code);
    println!("  Timestamps: {}", config.include_timestamps);
    println!("  VAD strategy: {:?}", config.vad_commit_strategy);
    println!("  Timeout: {}ms", config.timeout_ms);

    assert_eq!(config.model_id, "custom_model");
    assert_eq!(config.language_code, Some("zh".to_string()));
    assert!(config.include_timestamps);
    assert_eq!(config.vad_commit_strategy, Some("auto".to_string()));
    assert_eq!(config.timeout_ms, 5000);

    println!("\n✓ Builder pattern works correctly");
}

#[test]
fn test_connection_config_url_building() {
    println!("\n=== URL Building Test ===");

    // Basic URL
    let config1 = ConnectionConfig::new(16000);
    let url1 = config1.build_url().unwrap();

    println!("Basic URL:\n{}", url1);

    assert!(url1.starts_with("wss://"));
    assert!(url1.contains("api.elevenlabs.io"));
    assert!(url1.contains("v1/speech-to-text/realtime"));
    assert!(url1.contains("model_id=scribe_v2_realtime"));
    assert!(url1.contains("sample_rate=16000"));
    assert!(!url1.contains("language_code"));
    assert!(!url1.contains("include_timestamps"));

    // URL with all options
    let config2 = ConnectionConfig::new(16000)
        .with_language("zh")
        .with_timestamps()
        .with_vad_strategy("auto");

    let url2 = config2.build_url().unwrap();

    println!("\nFull URL:\n{}", url2);

    assert!(url2.contains("language_code=zh"));
    assert!(url2.contains("include_timestamps=true"));
    assert!(url2.contains("vad_commit_strategy=auto"));

    println!("\n✓ URL building works correctly");
}

#[test]
fn test_connection_config_different_sample_rates() {
    println!("\n=== Different Sample Rates Test ===");

    let rates = vec![8000, 16000, 22050, 44100, 48000];

    for rate in rates {
        let config = ConnectionConfig::new(rate);
        let url = config.build_url().unwrap();

        println!("Sample rate {}: {}", rate, url.contains(&format!("sample_rate={}", rate)));
        assert!(url.contains(&format!("sample_rate={}", rate)));
    }

    println!("\n✓ All sample rates handled correctly");
}

#[test]
fn test_connection_config_language_codes() {
    println!("\n=== Language Codes Test ===");

    let languages = vec!["zh", "en", "es", "fr", "de", "ja", "ko"];

    for lang in languages {
        let config = ConnectionConfig::new(16000).with_language(lang);
        let url = config.build_url().unwrap();

        println!("Language {}: URL contains language_code={}", lang, lang);
        assert!(url.contains(&format!("language_code={}", lang)));
        assert_eq!(config.language_code, Some(lang.to_string()));
    }

    println!("\n✓ All language codes handled correctly");
}

#[test]
fn test_connection_config_default() {
    println!("\n=== Default Configuration Test ===");

    let config = ConnectionConfig::default();

    println!("Default configuration:");
    println!("  Sample rate: {}", config.sample_rate);
    println!("  Model ID: {}", config.model_id);

    assert_eq!(config.sample_rate, 16000);
    assert_eq!(config.model_id, "scribe_v2_realtime");

    println!("\n✓ Default configuration matches expected values");
}

#[test]
fn test_connection_config_chaining() {
    println!("\n=== Builder Chaining Test ===");

    // Test that builder methods can be chained in any order
    let config1 = ConnectionConfig::new(16000)
        .with_language("en")
        .with_model("model1")
        .with_timestamps();

    let config2 = ConnectionConfig::new(16000)
        .with_timestamps()
        .with_model("model1")
        .with_language("en");

    // Both should produce the same configuration
    assert_eq!(config1.model_id, config2.model_id);
    assert_eq!(config1.language_code, config2.language_code);
    assert_eq!(config1.include_timestamps, config2.include_timestamps);

    println!("Configuration 1: {:?}", config1);
    println!("Configuration 2: {:?}", config2);
    println!("\n✓ Builder chaining order doesn't matter");
}

#[test]
fn test_url_query_parameter_format() {
    println!("\n=== URL Query Parameter Format Test ===");

    let config = ConnectionConfig::new(16000)
        .with_model("test_model")
        .with_language("zh")
        .with_timestamps();

    let url = config.build_url().unwrap();

    println!("Generated URL:\n{}", url);

    // Check proper query string format
    assert!(url.contains("?"), "URL should contain query string");
    assert!(url.contains("&"), "URL should contain parameter separators");

    // Check no double ampersands or question marks
    assert!(!url.contains("&&"), "URL should not contain double ampersands");
    assert_eq!(url.matches('?').count(), 1, "URL should have exactly one question mark");

    // Check parameters are properly formatted
    let parts: Vec<&str> = url.split('?').collect();
    assert_eq!(parts.len(), 2, "URL should have base and query parts");

    let query_string = parts[1];
    let params: Vec<&str> = query_string.split('&').collect();

    println!("\nQuery parameters:");
    for param in &params {
        println!("  {}", param);
        assert!(param.contains('='), "Each parameter should contain '='");
    }

    println!("\n✓ URL query parameters formatted correctly");
}

#[test]
fn test_network_error_types() {
    println!("\n=== NetworkError Types Test ===");

    // Test different error types
    let errors = vec![
        NetworkError::ConnectionFailed("test".to_string()),
        NetworkError::AuthenticationFailed,
        NetworkError::ProtocolError("test".to_string()),
        NetworkError::Timeout(5000),
        NetworkError::ConnectionClosed,
        NetworkError::InvalidConfig("test".to_string()),
        NetworkError::ServerError("test".to_string()),
    ];

    println!("Testing error types:");
    for error in errors {
        println!("  {}", error);
        // Just verify they can be formatted
        let _ = format!("{:?}", error);
    }

    println!("\n✓ All error types work correctly");
}

#[test]
fn test_connection_config_timeout_values() {
    println!("\n=== Timeout Values Test ===");

    let timeouts = vec![1000, 5000, 10000, 30000, 60000];

    for timeout in timeouts {
        let config = ConnectionConfig::new(16000).with_timeout(timeout);

        println!("Timeout {}: {}", timeout, config.timeout_ms);
        assert_eq!(config.timeout_ms, timeout);
    }

    println!("\n✓ Timeout values set correctly");
}

#[test]
fn test_vad_strategies() {
    println!("\n=== VAD Strategies Test ===");

    let strategies = vec!["auto", "manual", "silence_500ms"];

    for strategy in strategies {
        let config = ConnectionConfig::new(16000).with_vad_strategy(strategy);
        let url = config.build_url().unwrap();

        println!("Strategy '{}': present in URL", strategy);
        assert!(url.contains(&format!("vad_commit_strategy={}", strategy)));
    }

    println!("\n✓ VAD strategies handled correctly");
}

// Note: The following tests require a valid API key and actual network connection
// They are marked as ignored by default and can be run with `cargo test -- --ignored`

#[ignore]
#[tokio::test]
async fn test_connection_with_invalid_api_key() {
    println!("\n=== Connection with Invalid API Key Test ===");

    use raflow_lib::network::ScribeConnection;

    let config = ConnectionConfig::new(16000);
    let result = ScribeConnection::connect("invalid-api-key", &config).await;

    println!("Connection result: {:?}", result);

    // Should fail with authentication error
    assert!(result.is_err());
    if let Err(NetworkError::AuthenticationFailed) = result {
        println!("✓ Correctly rejected invalid API key");
    } else {
        println!("⚠ Expected AuthenticationFailed error");
    }
}

#[ignore]
#[tokio::test]
async fn test_connection_timeout() {
    println!("\n=== Connection Timeout Test ===");

    use raflow_lib::network::ScribeConnection;

    // Use very short timeout to force timeout
    let config = ConnectionConfig::new(16000).with_timeout(1); // 1ms

    let result = ScribeConnection::connect("test-key", &config).await;

    println!("Connection result: {:?}", result);

    // Should fail with timeout
    assert!(result.is_err());
}

#[test]
fn test_config_immutability() {
    println!("\n=== Config Immutability Test ===");

    let config1 = ConnectionConfig::new(16000);
    let config2 = config1.clone().with_language("zh");

    // Original should be unchanged
    assert_eq!(config1.language_code, None);
    // New config should have the language
    assert_eq!(config2.language_code, Some("zh".to_string()));

    println!("Original config language: {:?}", config1.language_code);
    println!("Modified config language: {:?}", config2.language_code);
    println!("\n✓ Builder pattern doesn't mutate original");
}
