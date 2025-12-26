import { useEffect, useState, useCallback } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { Waveform } from "./Waveform";
import { TranscriptDisplay } from "./TranscriptDisplay";
import { StatusIndicator } from "./StatusIndicator";

export type OverlayStatus = "idle" | "connecting" | "recording" | "processing" | "injecting" | "error";

interface OverlayState {
  status: OverlayStatus;
  partialText: string;
  finalText: string;
  audioLevel: number;
  errorMessage: string;
  isTranscribing: boolean;
}

interface StateChangeEvent {
  state: string;
  sub_state: string | null;
  partial_text: string | null;
  confidence: number | null;
  error_message: string | null;
}

interface PartialTranscriptEvent {
  text: string;
}

interface AudioLevelEvent {
  level: number;
}

interface SessionEventPayload {
  type: string;
  payload: {
    text?: string;
    session_id?: string;
    message?: string;
    strategy?: string;
  };
}

export function Overlay() {
  const [state, setState] = useState<OverlayState>({
    status: "idle",
    partialText: "",
    finalText: "",
    audioLevel: 0,
    errorMessage: "",
    isTranscribing: false,
  });

  const mapStateToStatus = useCallback((stateStr: string): OverlayStatus => {
    switch (stateStr.toLowerCase()) {
      case "idle":
        return "idle";
      case "connecting":
        return "connecting";
      case "recording":
        return "recording";
      case "processing":
        return "processing";
      case "injecting":
        return "injecting";
      case "error":
        return "error";
      default:
        return "idle";
    }
  }, []);

  useEffect(() => {
    const unlistenFns: Promise<UnlistenFn>[] = [];

    // Listen for state changes
    unlistenFns.push(
      listen<StateChangeEvent>("app:state_changed", (event) => {
        const { state: stateName, sub_state, partial_text, error_message } = event.payload;
        setState((prev) => ({
          ...prev,
          status: mapStateToStatus(stateName),
          partialText: partial_text || prev.partialText,
          errorMessage: error_message || "",
          isTranscribing: sub_state === "transcribing",
        }));
      })
    );

    // Listen for partial transcript updates
    unlistenFns.push(
      listen<PartialTranscriptEvent>("transcript:partial", (event) => {
        setState((prev) => ({
          ...prev,
          partialText: event.payload.text,
          isTranscribing: true,
        }));
      })
    );

    // Listen for session events
    unlistenFns.push(
      listen<SessionEventPayload>("session:event", (event) => {
        const { type, payload } = event.payload;

        switch (type) {
          case "Started":
            setState((prev) => ({
              ...prev,
              status: "recording",
              partialText: "",
              finalText: "",
              errorMessage: "",
            }));
            break;

          case "PartialTranscript":
            if (payload.text) {
              setState((prev) => ({
                ...prev,
                partialText: payload.text!,
                isTranscribing: true,
              }));
            }
            break;

          case "CommittedTranscript":
            if (payload.text) {
              setState((prev) => ({
                ...prev,
                finalText: payload.text!,
                status: "processing",
              }));
            }
            break;

          case "TextInjected":
          case "TextCopied":
            setState((prev) => ({
              ...prev,
              status: "idle",
            }));
            break;

          case "Stopped":
            setState((prev) => ({
              ...prev,
              status: "idle",
              partialText: "",
              isTranscribing: false,
            }));
            break;

          case "Error":
            setState((prev) => ({
              ...prev,
              status: "error",
              errorMessage: payload.message || "Unknown error",
            }));
            break;
        }
      })
    );

    // Listen for audio level updates
    unlistenFns.push(
      listen<AudioLevelEvent>("audio:level", (event) => {
        setState((prev) => ({
          ...prev,
          audioLevel: event.payload.level,
        }));
      })
    );

    // Listen for connecting event
    unlistenFns.push(
      listen("session:connecting", () => {
        setState((prev) => ({
          ...prev,
          status: "connecting",
          partialText: "",
          finalText: "",
          errorMessage: "",
        }));
      })
    );

    // Cleanup
    return () => {
      unlistenFns.forEach((unlisten) => {
        unlisten.then((fn) => fn());
      });
    };
  }, [mapStateToStatus]);

  // Don't render anything when idle
  if (state.status === "idle") {
    return null;
  }

  return (
    <div className="overlay">
      <div className="overlay-header">
        <StatusIndicator status={state.status} isTranscribing={state.isTranscribing} />
        <Waveform level={state.audioLevel} active={state.status === "recording"} />
      </div>
      <div className="overlay-content">
        <TranscriptDisplay
          partialText={state.partialText}
          finalText={state.finalText}
          status={state.status}
          errorMessage={state.errorMessage}
        />
      </div>
    </div>
  );
}
