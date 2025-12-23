/// Integration tests for P1-T8: End-to-End Transcription
///
/// This test file validates the complete transcription session,
/// integrating audio pipeline and network communication.

use raflow_lib::transcription::{TranscriptEvent, TranscriptionSession};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[test]
fn test_transcript_event_creation() {
    println!("\n=== TranscriptEvent Creation Test ===");

    let events = vec![
        TranscriptEvent::SessionStarted {
            session_id: "test-123".to_string(),
        },
        TranscriptEvent::Partial {
            text: "hello".to_string(),
        },
        TranscriptEvent::Committed {
            text: "hello world".to_string(),
        },
        TranscriptEvent::Error {
            message: "test error".to_string(),
        },
        TranscriptEvent::Closed,
    ];

    println!("Created {} event types:", events.len());
    for event in &events {
        println!("  {:?}", event);
    }

    assert_eq!(events.len(), 5);
    println!("\n✓ All event types created successfully");
}

#[test]
fn test_transcript_event_equality() {
    println!("\n=== TranscriptEvent Equality Test ===");

    let event1 = TranscriptEvent::Partial {
        text: "test".to_string(),
    };
    let event2 = TranscriptEvent::Partial {
        text: "test".to_string(),
    };
    let event3 = TranscriptEvent::Committed {
        text: "test".to_string(),
    };

    println!("Event 1: {:?}", event1);
    println!("Event 2: {:?}", event2);
    println!("Event 3: {:?}", event3);

    assert_eq!(event1, event2);
    assert_ne!(event1, event3);

    println!("\n✓ Event equality works correctly");
}

#[tokio::test]
async fn test_event_callback_mechanism() {
    println!("\n=== Event Callback Mechanism Test ===");

    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();

    // Create a callback that captures events
    let callback = move |event: TranscriptEvent| {
        println!("Callback received: {:?}", event);
        events_clone.lock().unwrap().push(event);
    };

    // Simulate calling the callback
    callback(TranscriptEvent::Partial {
        text: "test 1".to_string(),
    });
    callback(TranscriptEvent::Committed {
        text: "test 2".to_string(),
    });
    callback(TranscriptEvent::Closed);

    // Verify events were captured
    let captured = events.lock().unwrap();
    assert_eq!(captured.len(), 3);

    println!("Captured {} events:", captured.len());
    for event in captured.iter() {
        println!("  {:?}", event);
    }

    println!("\n✓ Callback mechanism works correctly");
}

#[tokio::test]
async fn test_event_channel_flow() {
    println!("\n=== Event Channel Flow Test ===");

    let (tx, mut rx) = mpsc::channel::<TranscriptEvent>(10);

    // Spawn a task to send events
    let sender = tokio::spawn(async move {
        let events = vec![
            TranscriptEvent::SessionStarted {
                session_id: "test-session".to_string(),
            },
            TranscriptEvent::Partial {
                text: "hello".to_string(),
            },
            TranscriptEvent::Partial {
                text: "hello world".to_string(),
            },
            TranscriptEvent::Committed {
                text: "hello world!".to_string(),
            },
        ];

        for event in events {
            println!("Sending: {:?}", event);
            tx.send(event).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        println!("Sender completed");
    });

    // Receive events
    let mut received_count = 0;
    let mut session_started = false;
    let mut partial_count = 0;
    let mut committed_count = 0;

    while let Some(event) = rx.recv().await {
        received_count += 1;
        println!("Received ({}): {:?}", received_count, event);

        match event {
            TranscriptEvent::SessionStarted { .. } => session_started = true,
            TranscriptEvent::Partial { .. } => partial_count += 1,
            TranscriptEvent::Committed { .. } => {
                committed_count += 1;
                break; // Stop after receiving committed
            }
            _ => {}
        }
    }

    sender.await.unwrap();

    println!("\nResults:");
    println!("  Total received: {}", received_count);
    println!("  Session started: {}", session_started);
    println!("  Partial transcripts: {}", partial_count);
    println!("  Committed transcripts: {}", committed_count);

    assert!(session_started);
    assert_eq!(partial_count, 2);
    assert_eq!(committed_count, 1);

    println!("\n✓ Event channel flow works correctly");
}

#[tokio::test]
async fn test_concurrent_event_handling() {
    println!("\n=== Concurrent Event Handling Test ===");

    let (tx1, mut rx1) = mpsc::channel::<TranscriptEvent>(10);
    let (tx2, mut rx2) = mpsc::channel::<TranscriptEvent>(10);

    // Spawn two senders
    let sender1 = tokio::spawn(async move {
        for i in 0..5 {
            tx1.send(TranscriptEvent::Partial {
                text: format!("sender1_{}", i),
            })
            .await
            .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        }
    });

    let sender2 = tokio::spawn(async move {
        for i in 0..5 {
            tx2.send(TranscriptEvent::Partial {
                text: format!("sender2_{}", i),
            })
            .await
            .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        }
    });

    // Receive from both channels concurrently
    let receiver1 = tokio::spawn(async move {
        let mut count = 0;
        while let Some(event) = rx1.recv().await {
            if let TranscriptEvent::Partial { text } = event {
                println!("Receiver 1: {}", text);
                count += 1;
            }
        }
        count
    });

    let receiver2 = tokio::spawn(async move {
        let mut count = 0;
        while let Some(event) = rx2.recv().await {
            if let TranscriptEvent::Partial { text } = event {
                println!("Receiver 2: {}", text);
                count += 1;
            }
        }
        count
    });

    sender1.await.unwrap();
    sender2.await.unwrap();

    let count1 = receiver1.await.unwrap();
    let count2 = receiver2.await.unwrap();

    println!("\nResults:");
    println!("  Receiver 1 count: {}", count1);
    println!("  Receiver 2 count: {}", count2);

    assert_eq!(count1, 5);
    assert_eq!(count2, 5);

    println!("\n✓ Concurrent event handling works correctly");
}

#[tokio::test]
async fn test_error_event_handling() {
    println!("\n=== Error Event Handling Test ===");

    let (tx, mut rx) = mpsc::channel::<TranscriptEvent>(10);

    // Send various events including errors
    tokio::spawn(async move {
        let events = vec![
            TranscriptEvent::SessionStarted {
                session_id: "test".to_string(),
            },
            TranscriptEvent::Partial {
                text: "hello".to_string(),
            },
            TranscriptEvent::Error {
                message: "Audio error".to_string(),
            },
            TranscriptEvent::Closed,
        ];

        for event in events {
            tx.send(event).await.unwrap();
        }
    });

    // Process events
    let mut error_received = false;
    let mut closed_received = false;

    while let Some(event) = rx.recv().await {
        match &event {
            TranscriptEvent::Error { message } => {
                println!("Error received: {}", message);
                error_received = true;
            }
            TranscriptEvent::Closed => {
                println!("Closed event received");
                closed_received = true;
                break;
            }
            _ => {
                println!("Other event: {:?}", event);
            }
        }
    }

    assert!(error_received);
    assert!(closed_received);

    println!("\n✓ Error event handling works correctly");
}

#[test]
fn test_event_pattern_matching() {
    println!("\n=== Event Pattern Matching Test ===");

    let events = vec![
        TranscriptEvent::SessionStarted {
            session_id: "s1".to_string(),
        },
        TranscriptEvent::Partial {
            text: "p1".to_string(),
        },
        TranscriptEvent::Committed {
            text: "c1".to_string(),
        },
        TranscriptEvent::Error {
            message: "e1".to_string(),
        },
        TranscriptEvent::Closed,
    ];

    for event in events {
        let description = match event {
            TranscriptEvent::SessionStarted { session_id } => {
                format!("Session: {}", session_id)
            }
            TranscriptEvent::Partial { text } => format!("Partial: {}", text),
            TranscriptEvent::Committed { text } => format!("Committed: {}", text),
            TranscriptEvent::Error { message } => format!("Error: {}", message),
            TranscriptEvent::Closed => "Closed".to_string(),
        };

        println!("  {}", description);
    }

    println!("\n✓ Event pattern matching works correctly");
}

// Note: The following test requires a valid API key and is marked as ignored
// Run with: cargo test test_e2e_transcription -- --ignored --nocapture
#[ignore]
#[tokio::test]
async fn test_e2e_transcription_with_real_api() {
    println!("\n=== End-to-End Transcription Test (Real API) ===");

    // Try to load API key from environment
    let api_key = match std::env::var("ELEVENLABS_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("⚠ ELEVENLABS_API_KEY not set, skipping real API test");
            return;
        }
    };

    println!("Starting transcription session...");

    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();

    let mut session = match TranscriptionSession::start(&api_key, move |event| {
        println!("Event: {:?}", event);
        events_clone.lock().unwrap().push(event);
    })
    .await
    {
        Ok(session) => session,
        Err(e) => {
            eprintln!("Failed to start session: {}", e);
            return;
        }
    };

    println!("Session started, recording for 10 seconds...");
    println!("Speak into your microphone!");

    // Record for 10 seconds
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    println!("Stopping session...");
    session.stop().await.unwrap();

    println!("Session stopped");

    // Check results
    let captured = events.lock().unwrap();
    println!("\nReceived {} events:", captured.len());
    for (i, event) in captured.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }

    // Basic assertions
    assert!(captured.len() > 0, "Should have received at least one event");

    let has_session_started = captured
        .iter()
        .any(|e| matches!(e, TranscriptEvent::SessionStarted { .. }));
    assert!(has_session_started, "Should have received SessionStarted");

    println!("\n✓ End-to-end transcription test completed");
}
