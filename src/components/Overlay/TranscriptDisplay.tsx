import { OverlayStatus } from "./index";

interface TranscriptDisplayProps {
  partialText: string;
  finalText: string;
  status: OverlayStatus;
  errorMessage: string;
}

export function TranscriptDisplay({
  partialText,
  finalText,
  status,
  errorMessage,
}: TranscriptDisplayProps) {
  // Show error message
  if (status === "error" && errorMessage) {
    return (
      <div className="transcript-display transcript-error">
        <span className="error-icon">!</span>
        <span className="error-text">{errorMessage}</span>
      </div>
    );
  }

  // Show connecting message
  if (status === "connecting") {
    return (
      <div className="transcript-display transcript-connecting">
        <span className="connecting-dots">
          <span>.</span>
          <span>.</span>
          <span>.</span>
        </span>
        <span className="connecting-text">Connecting</span>
      </div>
    );
  }

  // Show processing message
  if (status === "processing" || status === "injecting") {
    return (
      <div className="transcript-display transcript-processing">
        {finalText ? (
          <span className="final-text">{finalText}</span>
        ) : (
          <span className="processing-text">Processing...</span>
        )}
      </div>
    );
  }

  // Show recording/transcription
  if (status === "recording") {
    if (partialText) {
      return (
        <div className="transcript-display transcript-active">
          <span className="partial-text">{partialText}</span>
          <span className="cursor">|</span>
        </div>
      );
    }
    return (
      <div className="transcript-display transcript-listening">
        <span className="listening-icon">
          <MicrophoneIcon />
        </span>
        <span className="listening-text">Listening...</span>
      </div>
    );
  }

  return null;
}

function MicrophoneIcon() {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
      <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
      <line x1="12" y1="19" x2="12" y2="23" />
      <line x1="8" y1="23" x2="16" y2="23" />
    </svg>
  );
}
