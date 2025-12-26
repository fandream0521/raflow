import { OverlayStatus } from "./index";

interface StatusIndicatorProps {
  status: OverlayStatus;
  isTranscribing: boolean;
}

const statusConfig: Record<OverlayStatus, { label: string; className: string }> = {
  idle: { label: "Ready", className: "status-idle" },
  connecting: { label: "Connecting", className: "status-connecting" },
  recording: { label: "Recording", className: "status-recording" },
  processing: { label: "Processing", className: "status-processing" },
  injecting: { label: "Injecting", className: "status-injecting" },
  error: { label: "Error", className: "status-error" },
};

export function StatusIndicator({ status, isTranscribing }: StatusIndicatorProps) {
  const config = statusConfig[status];

  // Show "Transcribing" when actively transcribing
  const displayLabel = status === "recording" && isTranscribing ? "Transcribing" : config.label;

  return (
    <div className={`status-indicator ${config.className}`}>
      <span className="status-dot" />
      <span className="status-label">{displayLabel}</span>
    </div>
  );
}
