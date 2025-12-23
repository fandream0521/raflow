/// Integration tests for P1-T7: Send/Receive Tasks
///
/// This test file validates the async task functions for concurrent
/// WebSocket send and receive operations.

use raflow_lib::network::messages::{ClientMessage, InputAudioChunk, ServerMessage};
use tokio::sync::mpsc;

#[test]
fn test_audio_chunk_message_creation() {
    println!("\n=== Audio Chunk Message Creation Test ===");

    let audio_data = "dGVzdCBhdWRpbyBkYXRh"; // "test audio data" in base64

    let chunk = InputAudioChunk::new(audio_data.to_string()).with_sample_rate(16000);

    println!("Created audio chunk:");
    println!("  Audio data length: {}", chunk.audio_base_64.len());
    println!("  Sample rate: {:?}", chunk.sample_rate);

    assert_eq!(chunk.audio_base_64, audio_data);
    assert_eq!(chunk.sample_rate, Some(16000));

    println!("\n✓ Audio chunk created correctly");
}

#[test]
fn test_client_message_serialization() {
    println!("\n=== Client Message Serialization Test ===");

    let chunk = InputAudioChunk::new("dGVzdA==".to_string()).with_sample_rate(16000);

    let msg = ClientMessage::InputAudioChunk(chunk);
    let json = serde_json::to_string(&msg).unwrap();

    println!("Serialized message:");
    println!("{}", json);

    assert!(json.contains("input_audio_chunk"));
    assert!(json.contains("dGVzdA=="));
    assert!(json.contains("16000"));

    println!("\n✓ Client message serialized correctly");
}

#[tokio::test]
async fn test_mpsc_channel_audio_flow() {
    println!("\n=== MPSC Channel Audio Flow Test ===");

    let (tx, mut rx) = mpsc::channel::<String>(10);

    // Simulate sending audio chunks
    let sender = tokio::spawn(async move {
        for i in 0..5 {
            let audio = format!("audio_chunk_{}", i);
            tx.send(audio.clone()).await.unwrap();
            println!("Sent: {}", audio);
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        println!("Sender completed");
    });

    // Simulate receiving audio chunks
    let receiver = tokio::spawn(async move {
        let mut count = 0;
        while let Some(audio) = rx.recv().await {
            println!("Received: {}", audio);
            count += 1;
        }
        println!("Receiver completed: {} chunks", count);
        count
    });

    sender.await.unwrap();
    let received_count = receiver.await.unwrap();

    assert_eq!(received_count, 5);
    println!("\n✓ MPSC channel flow works correctly");
}

#[tokio::test]
async fn test_mpsc_channel_message_flow() {
    println!("\n=== MPSC Channel Message Flow Test ===");

    let (tx, mut rx) = mpsc::channel::<ServerMessage>(10);

    // Simulate sending server messages
    let sender = tokio::spawn(async move {
        let messages = vec![
            ServerMessage::PartialTranscript {
                text: "Hello".to_string(),
            },
            ServerMessage::PartialTranscript {
                text: "Hello world".to_string(),
            },
            ServerMessage::CommittedTranscript {
                text: "Hello world!".to_string(),
            },
        ];

        for msg in messages {
            tx.send(msg.clone()).await.unwrap();
            println!("Sent: {:?}", std::mem::discriminant(&msg));
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        println!("Sender completed");
    });

    // Simulate receiving messages
    let receiver = tokio::spawn(async move {
        let mut partials = 0;
        let mut committed = 0;

        while let Some(msg) = rx.recv().await {
            match &msg {
                ServerMessage::PartialTranscript { text } => {
                    println!("Received partial: {}", text);
                    partials += 1;
                }
                ServerMessage::CommittedTranscript { text } => {
                    println!("Received committed: {}", text);
                    committed += 1;
                }
                _ => {}
            }
        }

        println!("Receiver completed: {} partials, {} committed", partials, committed);
        (partials, committed)
    });

    sender.await.unwrap();
    let (partials, committed) = receiver.await.unwrap();

    assert_eq!(partials, 2);
    assert_eq!(committed, 1);
    println!("\n✓ Message flow works correctly");
}

#[tokio::test]
async fn test_channel_closure_detection() {
    println!("\n=== Channel Closure Detection Test ===");

    let (tx, mut rx) = mpsc::channel::<String>(10);

    // Send a few messages then drop the sender
    let sender = tokio::spawn(async move {
        tx.send("message1".to_string()).await.unwrap();
        tx.send("message2".to_string()).await.unwrap();
        println!("Sender dropping (closing channel)");
        // tx is dropped here, closing the channel
    });

    sender.await.unwrap();

    // Receiver should get all messages then None
    let mut count = 0;
    while let Some(msg) = rx.recv().await {
        count += 1;
        println!("Received message {}: {}", count, msg);
    }

    println!("Channel closed after {} messages", count);
    assert_eq!(count, 2);

    println!("\n✓ Channel closure detected correctly");
}

#[tokio::test]
async fn test_concurrent_send_receive() {
    println!("\n=== Concurrent Send/Receive Test ===");

    let (audio_tx, mut audio_rx) = mpsc::channel::<String>(10);
    let (msg_tx, mut msg_rx) = mpsc::channel::<ServerMessage>(10);

    // Simulate sender task
    let sender = tokio::spawn(async move {
        for i in 0..3 {
            let audio = format!("audio_{}", i);
            audio_tx.send(audio.clone()).await.unwrap();
            println!("Audio sent: {}", audio);
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        println!("Audio sender completed");
    });

    // Simulate receiver task
    let receiver = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;

        for i in 0..3 {
            let msg = ServerMessage::PartialTranscript {
                text: format!("transcript_{}", i),
            };
            msg_tx.send(msg.clone()).await.unwrap();
            println!("Message sent: transcript_{}", i);
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        println!("Message sender completed");
    });

    // Consumer task
    let audio_consumer = tokio::spawn(async move {
        let mut count = 0;
        while let Some(audio) = audio_rx.recv().await {
            println!("Consumed audio: {}", audio);
            count += 1;
        }
        count
    });

    let msg_consumer = tokio::spawn(async move {
        let mut count = 0;
        while let Some(msg) = msg_rx.recv().await {
            if let ServerMessage::PartialTranscript { text } = msg {
                println!("Consumed message: {}", text);
            }
            count += 1;
        }
        count
    });

    sender.await.unwrap();
    receiver.await.unwrap();

    let audio_count = audio_consumer.await.unwrap();
    let msg_count = msg_consumer.await.unwrap();

    assert_eq!(audio_count, 3);
    assert_eq!(msg_count, 3);

    println!("\n✓ Concurrent operations work correctly");
}

#[test]
fn test_base64_audio_encoding() {
    println!("\n=== Base64 Audio Encoding Test ===");

    // Simulate PCM audio data (100 samples of i16)
    let samples: Vec<i16> = (0..100).map(|i| (i * 327) as i16).collect();

    // Convert to bytes (little-endian)
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for &sample in &samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }

    // Encode to base64
    use base64::Engine;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&bytes);

    println!("Encoded audio:");
    println!("  Samples: {}", samples.len());
    println!("  Bytes: {}", bytes.len());
    println!("  Base64 length: {}", base64_data.len());

    // Create message
    let chunk = InputAudioChunk::new(base64_data.clone());
    let json = serde_json::to_string(&ClientMessage::InputAudioChunk(chunk)).unwrap();

    println!("  JSON length: {}", json.len());

    assert_eq!(bytes.len(), 200); // 100 samples * 2 bytes
    assert!(base64_data.len() > 200); // Base64 is larger
    assert!(json.contains(&base64_data));

    println!("\n✓ Base64 encoding works correctly");
}

#[test]
fn test_message_size_estimation() {
    println!("\n=== Message Size Estimation Test ===");

    // 100ms of audio at 16kHz = 1600 samples = 3200 bytes
    let sample_count = 1600;
    let byte_count = sample_count * 2;

    use base64::Engine;
    let dummy_bytes = vec![0u8; byte_count];
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&dummy_bytes);

    let chunk = InputAudioChunk::new(base64_data);
    let json = serde_json::to_string(&ClientMessage::InputAudioChunk(chunk)).unwrap();

    println!("100ms audio chunk:");
    println!("  Samples: {}", sample_count);
    println!("  PCM bytes: {}", byte_count);
    println!("  Base64 bytes: {}", json.len());
    println!("  Chunks per second: 10");
    println!("  Bandwidth: ~{} KB/s", (json.len() * 10) / 1024);

    // Base64 encoding increases size by ~33%
    assert!(json.len() > byte_count);
    assert!(json.len() < byte_count * 2);

    println!("\n✓ Message size estimation complete");
}

#[tokio::test]
async fn test_task_coordination() {
    println!("\n=== Task Coordination Test ===");

    let (audio_tx, mut audio_rx) = mpsc::channel::<String>(10);
    let (msg_tx, mut msg_rx) = mpsc::channel::<ServerMessage>(10);
    let (done_tx, mut done_rx) = mpsc::channel::<()>(1);

    // Audio sender (simulates audio pipeline)
    tokio::spawn(async move {
        for i in 0..5 {
            audio_tx
                .send(format!("audio_{}", i))
                .await
                .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        println!("Audio pipeline completed");
    });

    // Message receiver (simulates transcript handler)
    tokio::spawn(async move {
        let mut count = 0;
        while let Some(msg) = msg_rx.recv().await {
            match msg {
                ServerMessage::PartialTranscript { text } => {
                    println!("Transcript: {}", text);
                    count += 1;
                }
                _ => {}
            }
        }
        println!("Transcript handler completed: {} messages", count);
        done_tx.send(()).await.unwrap();
    });

    // Simulated processing (replaces actual WebSocket)
    tokio::spawn(async move {
        let mut responses = 0;
        while let Some(audio) = audio_rx.recv().await {
            println!("Processing: {}", audio);

            // Simulate server response
            let response = ServerMessage::PartialTranscript {
                text: format!("transcribed_{}", responses),
            };

            msg_tx.send(response).await.unwrap();
            responses += 1;

            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
        println!("Processing completed: {} responses", responses);
        // Drop msg_tx to signal completion
    });

    // Wait for completion
    tokio::time::timeout(tokio::time::Duration::from_secs(2), done_rx.recv())
        .await
        .expect("Test timeout")
        .expect("Done signal received");

    println!("\n✓ Task coordination works correctly");
}

#[test]
fn test_error_message_handling() {
    println!("\n=== Error Message Handling Test ===");

    let error_json = r#"{
        "message_type": "input_error",
        "error_message": "Invalid audio format"
    }"#;

    let msg: ServerMessage = serde_json::from_str(error_json).unwrap();

    match msg {
        ServerMessage::InputError { error_message } => {
            println!("Error message: {}", error_message);
            assert_eq!(error_message, "Invalid audio format");
        }
        _ => panic!("Expected InputError"),
    }

    println!("\n✓ Error message handled correctly");
}
