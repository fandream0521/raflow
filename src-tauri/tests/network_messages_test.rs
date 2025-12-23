/// Integration tests for P1-T5: Network Message Types
///
/// This test file validates all WebSocket message types used for
/// communication with the ElevenLabs Scribe v2 Realtime API.

use base64::Engine;
use raflow_lib::network::messages::*;
use serde_json;

#[test]
fn test_input_audio_chunk_full_cycle() {
    println!("\n=== InputAudioChunk Full Cycle Test ===");

    // Create chunk with all options
    let chunk = InputAudioChunk::new("SGVsbG8gV29ybGQ=".to_string())
        .with_sample_rate(16000)
        .with_commit()
        .with_previous_text("Previous context".to_string());

    println!("Created chunk:");
    println!("  Audio: {}", chunk.audio_base_64);
    println!("  Sample rate: {:?}", chunk.sample_rate);
    println!("  Commit: {:?}", chunk.commit);
    println!("  Previous text: {:?}", chunk.previous_text);

    // Serialize
    let json = serde_json::to_string(&chunk).unwrap();
    println!("\nSerialized JSON:\n{}", json);

    // Verify serialization
    assert!(json.contains("\"message_type\":\"input_audio_chunk\""));
    assert!(json.contains("\"audio_base_64\":\"SGVsbG8gV29ybGQ=\""));
    assert!(json.contains("\"sample_rate\":16000"));
    assert!(json.contains("\"commit\":true"));
    assert!(json.contains("\"previous_text\":\"Previous context\""));

    println!("\n✓ InputAudioChunk serialization correct");
}

#[test]
fn test_input_audio_chunk_minimal() {
    println!("\n=== InputAudioChunk Minimal Test ===");

    let chunk = InputAudioChunk::new("dGVzdA==".to_string());
    let json = serde_json::to_string(&chunk).unwrap();

    println!("Minimal JSON:\n{}", json);

    // Optional fields should be omitted
    assert!(!json.contains("\"commit\""));
    assert!(!json.contains("\"sample_rate\""));
    assert!(!json.contains("\"previous_text\""));

    println!("\n✓ Optional fields correctly omitted");
}

#[test]
fn test_commit_message() {
    println!("\n=== CommitMessage Test ===");

    let msg = CommitMessage::new();
    let json = serde_json::to_string(&msg).unwrap();

    println!("Commit message JSON: {}", json);

    assert_eq!(json, "{\"message_type\":\"commit\"}");

    println!("✓ CommitMessage format correct");
}

#[test]
fn test_close_message() {
    println!("\n=== CloseMessage Test ===");

    let msg = CloseMessage::new();
    let json = serde_json::to_string(&msg).unwrap();

    println!("Close message JSON: {}", json);

    assert_eq!(json, "{\"message_type\":\"close\"}");

    println!("✓ CloseMessage format correct");
}

#[test]
fn test_client_message_variants() {
    println!("\n=== ClientMessage Variants Test ===");

    // Test each variant
    let audio = ClientMessage::InputAudioChunk(InputAudioChunk::new("test".to_string()));
    let commit = ClientMessage::Commit(CommitMessage::new());
    let close = ClientMessage::Close(CloseMessage::new());

    let audio_json = serde_json::to_string(&audio).unwrap();
    let commit_json = serde_json::to_string(&commit).unwrap();
    let close_json = serde_json::to_string(&close).unwrap();

    println!("Audio variant: {}", audio_json);
    println!("Commit variant: {}", commit_json);
    println!("Close variant: {}", close_json);

    assert!(audio_json.contains("input_audio_chunk"));
    assert!(commit_json.contains("commit"));
    assert!(close_json.contains("close"));

    println!("\n✓ All ClientMessage variants serialize correctly");
}

#[test]
fn test_session_started_deserialization() {
    println!("\n=== SessionStarted Deserialization Test ===");

    let json = r#"{
        "message_type": "session_started",
        "session_id": "sess_abc123",
        "config": {
            "sample_rate": 16000,
            "audio_format": "pcm_s16le",
            "language_code": "zh",
            "model_id": "scribe_v2_realtime"
        }
    }"#;

    println!("Input JSON:\n{}", json);

    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match &msg {
        ServerMessage::SessionStarted { session_id, config } => {
            println!("\nParsed SessionStarted:");
            println!("  Session ID: {}", session_id);

            assert_eq!(session_id, "sess_abc123");

            if let Some(cfg) = config {
                println!("  Config:");
                println!("    Sample rate: {}", cfg.sample_rate);
                println!("    Audio format: {}", cfg.audio_format);
                println!("    Language: {:?}", cfg.language_code);
                println!("    Model: {}", cfg.model_id);

                assert_eq!(cfg.sample_rate, 16000);
                assert_eq!(cfg.audio_format, "pcm_s16le");
                assert_eq!(cfg.language_code, Some("zh".to_string()));
                assert_eq!(cfg.model_id, "scribe_v2_realtime");
            }
        }
        _ => panic!("Expected SessionStarted"),
    }

    assert_eq!(msg.session_id(), Some("sess_abc123"));
    println!("\n✓ SessionStarted deserialized correctly");
}

#[test]
fn test_partial_transcript_deserialization() {
    println!("\n=== PartialTranscript Deserialization Test ===");

    let json = r#"{
        "message_type": "partial_transcript",
        "text": "你好世界"
    }"#;

    println!("Input JSON:\n{}", json);

    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    assert!(msg.is_partial());
    assert!(!msg.is_committed());
    assert!(!msg.is_error());

    match msg {
        ServerMessage::PartialTranscript { text } => {
            println!("\nParsed text: {}", text);
            assert_eq!(text, "你好世界");
        }
        _ => panic!("Expected PartialTranscript"),
    }

    println!("\n✓ PartialTranscript deserialized correctly");
}

#[test]
fn test_committed_transcript_deserialization() {
    println!("\n=== CommittedTranscript Deserialization Test ===");

    let json = r#"{
        "message_type": "committed_transcript",
        "text": "这是最终的转写结果"
    }"#;

    println!("Input JSON:\n{}", json);

    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    assert!(!msg.is_partial());
    assert!(msg.is_committed());
    assert_eq!(msg.text(), Some("这是最终的转写结果"));

    println!("Parsed text: {:?}", msg.text());
    println!("\n✓ CommittedTranscript deserialized correctly");
}

#[test]
fn test_committed_with_timestamps_deserialization() {
    println!("\n=== CommittedTranscriptWithTimestamps Test ===");

    let json = r#"{
        "message_type": "committed_transcript_with_timestamps",
        "text": "Hello world.",
        "language_code": "en",
        "words": [
            {
                "word": "Hello",
                "start": 0.0,
                "end": 0.5,
                "type": "word",
                "logprob": -1.234
            },
            {
                "word": "world",
                "start": 0.6,
                "end": 1.0,
                "type": "word",
                "logprob": -0.567
            },
            {
                "word": ".",
                "start": 1.0,
                "end": 1.05,
                "type": "punctuation"
            }
        ]
    }"#;

    println!("Input JSON:\n{}", json);

    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    assert!(msg.is_committed());

    match msg {
        ServerMessage::CommittedTranscriptWithTimestamps {
            text,
            language_code,
            words,
        } => {
            println!("\nParsed transcript with timestamps:");
            println!("  Text: {}", text);
            println!("  Language: {}", language_code);
            println!("  Words: {}", words.len());

            assert_eq!(text, "Hello world.");
            assert_eq!(language_code, "en");
            assert_eq!(words.len(), 3);

            // Check first word
            let word1 = &words[0];
            println!("\n  Word 1:");
            println!("    Text: {}", word1.word);
            println!("    Timing: {:.2}s - {:.2}s", word1.start, word1.end);
            println!("    Duration: {:.2}s", word1.duration());
            println!("    Type: {}", word1.word_type);
            println!("    Confidence: {:?}", word1.logprob);

            assert_eq!(word1.word, "Hello");
            assert_eq!(word1.start, 0.0);
            assert_eq!(word1.end, 0.5);
            assert_eq!(word1.duration(), 0.5);
            assert!(!word1.is_punctuation());

            // Check punctuation
            let punct = &words[2];
            println!("\n  Word 3 (punctuation):");
            println!("    Text: {}", punct.word);
            println!("    Is punctuation: {}", punct.is_punctuation());

            assert_eq!(punct.word, ".");
            assert!(punct.is_punctuation());
        }
        _ => panic!("Expected CommittedTranscriptWithTimestamps"),
    }

    println!("\n✓ CommittedTranscriptWithTimestamps deserialized correctly");
}

#[test]
fn test_input_error_deserialization() {
    println!("\n=== InputError Deserialization Test ===");

    let json = r#"{
        "message_type": "input_error",
        "error_message": "Invalid audio format: expected PCM 16kHz"
    }"#;

    println!("Input JSON:\n{}", json);

    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    assert!(msg.is_error());
    assert!(!msg.is_partial());
    assert!(!msg.is_committed());

    match &msg {
        ServerMessage::InputError { error_message } => {
            println!("\nParsed error: {}", error_message);
            assert_eq!(
                error_message,
                "Invalid audio format: expected PCM 16kHz"
            );
        }
        _ => panic!("Expected InputError"),
    }

    assert_eq!(
        msg.error_message(),
        Some("Invalid audio format: expected PCM 16kHz")
    );

    println!("\n✓ InputError deserialized correctly");
}

#[test]
fn test_server_message_helper_methods() {
    println!("\n=== ServerMessage Helper Methods Test ===");

    let partial = ServerMessage::PartialTranscript {
        text: "test".to_string(),
    };
    let committed = ServerMessage::CommittedTranscript {
        text: "final".to_string(),
    };
    let error = ServerMessage::InputError {
        error_message: "error".to_string(),
    };

    println!("Testing helper methods:");
    println!("  partial.is_partial(): {}", partial.is_partial());
    println!("  committed.is_committed(): {}", committed.is_committed());
    println!("  error.is_error(): {}", error.is_error());

    assert!(partial.is_partial());
    assert!(committed.is_committed());
    assert!(error.is_error());

    println!("  partial.text(): {:?}", partial.text());
    println!("  committed.text(): {:?}", committed.text());
    println!("  error.error_message(): {:?}", error.error_message());

    assert_eq!(partial.text(), Some("test"));
    assert_eq!(committed.text(), Some("final"));
    assert_eq!(error.error_message(), Some("error"));

    println!("\n✓ All helper methods work correctly");
}

#[test]
fn test_real_world_session_flow() {
    println!("\n=== Real-World Session Flow Test ===");

    // 1. Session started
    let session_json = r#"{
        "message_type": "session_started",
        "session_id": "test-session"
    }"#;

    let session: ServerMessage = serde_json::from_str(session_json).unwrap();
    println!("1. Session started: {}", session.session_id().unwrap());

    // 2. Multiple partial transcripts
    for i in 1..=3 {
        let partial_json = format!(
            r#"{{
                "message_type": "partial_transcript",
                "text": "Partial text {}"
            }}"#,
            i
        );

        let partial: ServerMessage = serde_json::from_str(&partial_json).unwrap();
        assert!(partial.is_partial());
        println!("2.{} Partial: {}", i, partial.text().unwrap());
    }

    // 3. Final committed transcript
    let committed_json = r#"{
        "message_type": "committed_transcript",
        "text": "Final complete text"
    }"#;

    let committed: ServerMessage = serde_json::from_str(committed_json).unwrap();
    assert!(committed.is_committed());
    println!("3. Committed: {}", committed.text().unwrap());

    println!("\n✓ Real-world session flow works correctly");
}

#[test]
fn test_round_trip_serialization() {
    println!("\n=== Round-Trip Serialization Test ===");

    // Create client message
    let original = InputAudioChunk::new("dGVzdCBhdWRpbw==".to_string())
        .with_sample_rate(16000);

    // Serialize
    let json = serde_json::to_string(&original).unwrap();
    println!("Serialized:\n{}", json);

    // Parse as generic JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify fields
    assert_eq!(
        parsed["message_type"].as_str().unwrap(),
        "input_audio_chunk"
    );
    assert_eq!(
        parsed["audio_base_64"].as_str().unwrap(),
        "dGVzdCBhdWRpbw=="
    );
    assert_eq!(parsed["sample_rate"].as_u64().unwrap(), 16000);

    println!("\n✓ Round-trip serialization successful");
}

#[test]
fn test_message_size_estimation() {
    println!("\n=== Message Size Estimation Test ===");

    // Create a typical audio chunk (100ms @ 16kHz = 1600 samples = 3200 bytes)
    let pcm_bytes = vec![0u8; 3200];
    let base64_audio = base64::engine::general_purpose::STANDARD.encode(&pcm_bytes);

    let chunk = InputAudioChunk::new(base64_audio.clone()).with_sample_rate(16000);

    let json = serde_json::to_string(&chunk).unwrap();

    println!("Message sizes:");
    println!("  PCM bytes: {}", pcm_bytes.len());
    println!("  Base64 length: {}", base64_audio.len());
    println!("  JSON length: {}", json.len());
    println!("  Overhead: {} bytes", json.len() - base64_audio.len());

    // Verify Base64 encoding ratio (4/3 of original)
    let expected_base64_len = (pcm_bytes.len() * 4 + 2) / 3;
    assert!(
        (base64_audio.len() as i32 - expected_base64_len as i32).abs() <= 2,
        "Base64 length should be ~4/3 of original"
    );

    println!("\n✓ Message sizes are as expected");
}
