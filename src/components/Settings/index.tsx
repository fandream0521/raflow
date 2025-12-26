import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AppConfig {
  api: ApiConfig;
  audio: AudioConfig;
  hotkeys: HotkeyConfig;
  behavior: BehaviorConfig;
}

interface ApiConfig {
  api_key: string;
  model_id: string;
  language_code: string | null;
  include_timestamps: boolean;
  vad_commit_strategy: string | null;
}

interface AudioConfig {
  input_device_id: string | null;
  input_device_name: string | null;
  gain: number;
  noise_suppression: boolean;
  silence_threshold: number;
}

interface HotkeyConfig {
  push_to_talk: string;
  cancel: string;
  toggle_mode: string | null;
}

interface BehaviorConfig {
  injection_strategy: string;
  auto_threshold: number;
  paste_delay_ms: number;
  pre_injection_delay_ms: number;
  auto_inject: boolean;
  show_overlay: boolean;
  auto_start: boolean;
  minimize_to_tray: boolean;
  processing_timeout_secs: number;
}

type TabId = "api" | "audio" | "hotkeys" | "behavior";

interface TabConfig {
  id: TabId;
  label: string;
}

const tabs: TabConfig[] = [
  { id: "api", label: "API" },
  { id: "audio", label: "Audio" },
  { id: "hotkeys", label: "Hotkeys" },
  { id: "behavior", label: "Behavior" },
];

export function Settings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [activeTab, setActiveTab] = useState<TabId>("api");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  // Load config on mount
  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const cfg = await invoke<AppConfig>("get_config");
      setConfig(cfg);
      setError(null);
    } catch (e) {
      setError(`Failed to load config: ${e}`);
    }
  };

  const saveConfig = async () => {
    if (!config) return;

    setSaving(true);
    setError(null);
    setSuccess(false);

    try {
      await invoke("save_config", { config });
      setSuccess(true);
      setTimeout(() => setSuccess(false), 2000);
    } catch (e) {
      setError(`Failed to save config: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const updateConfig = useCallback(
    <K extends keyof AppConfig>(section: K, key: keyof AppConfig[K], value: AppConfig[K][keyof AppConfig[K]]) => {
      setConfig((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          [section]: {
            ...prev[section],
            [key]: value,
          },
        };
      });
    },
    []
  );

  if (!config) {
    return (
      <div className="settings-loading">
        <p>Loading settings...</p>
      </div>
    );
  }

  return (
    <div className="settings">
      <header className="settings-header">
        <h1>RaFlow Settings</h1>
        <p className="settings-subtitle">Configure your speech-to-text experience</p>
      </header>

      <nav className="settings-tabs">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`tab-button ${activeTab === tab.id ? "active" : ""}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </nav>

      <main className="settings-content">
        {activeTab === "api" && (
          <ApiSettings
            config={config.api}
            onChange={(key, value) => updateConfig("api", key, value)}
          />
        )}
        {activeTab === "audio" && (
          <AudioSettings
            config={config.audio}
            onChange={(key, value) => updateConfig("audio", key, value)}
          />
        )}
        {activeTab === "hotkeys" && (
          <HotkeySettings
            config={config.hotkeys}
            onChange={(key, value) => updateConfig("hotkeys", key, value)}
          />
        )}
        {activeTab === "behavior" && (
          <BehaviorSettings
            config={config.behavior}
            onChange={(key, value) => updateConfig("behavior", key, value)}
          />
        )}
      </main>

      <footer className="settings-footer">
        {error && <p className="error-message">{error}</p>}
        {success && <p className="success-message">Settings saved!</p>}
        <button className="save-button" onClick={saveConfig} disabled={saving}>
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </footer>
    </div>
  );
}

interface SectionProps<T> {
  config: T;
  onChange: <K extends keyof T>(key: K, value: T[K]) => void;
}

function ApiSettings({ config, onChange }: SectionProps<ApiConfig>) {
  return (
    <section className="settings-section">
      <h2>API Configuration</h2>

      <div className="form-group">
        <label htmlFor="api-key">ElevenLabs API Key</label>
        <input
          id="api-key"
          type="password"
          value={config.api_key}
          onChange={(e) => onChange("api_key", e.target.value)}
          placeholder="Enter your API key"
        />
        <p className="form-help">
          Get your API key from{" "}
          <a
            href="https://elevenlabs.io/app/subscription"
            target="_blank"
            rel="noopener noreferrer"
          >
            ElevenLabs Dashboard
          </a>
        </p>
      </div>

      <div className="form-group">
        <label htmlFor="language">Language</label>
        <select
          id="language"
          value={config.language_code || ""}
          onChange={(e) => onChange("language_code", e.target.value || null)}
        >
          <option value="">Auto-detect</option>
          <option value="zh">Chinese (zh)</option>
          <option value="en">English (en)</option>
          <option value="ja">Japanese (ja)</option>
          <option value="ko">Korean (ko)</option>
          <option value="de">German (de)</option>
          <option value="fr">French (fr)</option>
          <option value="es">Spanish (es)</option>
        </select>
      </div>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={config.include_timestamps}
            onChange={(e) => onChange("include_timestamps", e.target.checked)}
          />
          <span>Include word timestamps</span>
        </label>
      </div>
    </section>
  );
}

function AudioSettings({ config, onChange }: SectionProps<AudioConfig>) {
  return (
    <section className="settings-section">
      <h2>Audio Settings</h2>

      <div className="form-group">
        <label htmlFor="gain">Volume Gain</label>
        <div className="range-group">
          <input
            id="gain"
            type="range"
            min="0.5"
            max="2"
            step="0.1"
            value={config.gain}
            onChange={(e) => onChange("gain", parseFloat(e.target.value))}
          />
          <span className="range-value">{config.gain.toFixed(1)}x</span>
        </div>
      </div>

      <div className="form-group">
        <label htmlFor="silence-threshold">Silence Threshold</label>
        <div className="range-group">
          <input
            id="silence-threshold"
            type="range"
            min="0"
            max="0.1"
            step="0.005"
            value={config.silence_threshold}
            onChange={(e) => onChange("silence_threshold", parseFloat(e.target.value))}
          />
          <span className="range-value">{(config.silence_threshold * 100).toFixed(1)}%</span>
        </div>
      </div>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={config.noise_suppression}
            onChange={(e) => onChange("noise_suppression", e.target.checked)}
          />
          <span>Enable noise suppression (experimental)</span>
        </label>
      </div>
    </section>
  );
}

function HotkeySettings({ config, onChange }: SectionProps<HotkeyConfig>) {
  return (
    <section className="settings-section">
      <h2>Hotkey Settings</h2>

      <div className="form-group">
        <label htmlFor="ptt-hotkey">Push-to-Talk</label>
        <input
          id="ptt-hotkey"
          type="text"
          value={config.push_to_talk}
          onChange={(e) => onChange("push_to_talk", e.target.value)}
          placeholder="e.g., CommandOrControl+Shift+."
        />
        <p className="form-help">Hold this key to start recording, release to transcribe</p>
      </div>

      <div className="form-group">
        <label htmlFor="cancel-hotkey">Cancel</label>
        <input
          id="cancel-hotkey"
          type="text"
          value={config.cancel}
          onChange={(e) => onChange("cancel", e.target.value)}
          placeholder="e.g., Escape"
        />
        <p className="form-help">Press to cancel current recording</p>
      </div>
    </section>
  );
}

function BehaviorSettings({ config, onChange }: SectionProps<BehaviorConfig>) {
  return (
    <section className="settings-section">
      <h2>Behavior Settings</h2>

      <div className="form-group">
        <label htmlFor="injection-strategy">Text Injection Strategy</label>
        <select
          id="injection-strategy"
          value={config.injection_strategy}
          onChange={(e) => onChange("injection_strategy", e.target.value)}
        >
          <option value="Auto">Auto (keyboard for short, clipboard for long)</option>
          <option value="Keyboard">Always use keyboard simulation</option>
          <option value="Clipboard">Always use clipboard paste</option>
          <option value="ClipboardOnly">Copy to clipboard only</option>
        </select>
      </div>

      <div className="form-group">
        <label htmlFor="auto-threshold">Auto Strategy Threshold</label>
        <div className="range-group">
          <input
            id="auto-threshold"
            type="range"
            min="5"
            max="100"
            step="5"
            value={config.auto_threshold}
            onChange={(e) => onChange("auto_threshold", parseInt(e.target.value))}
          />
          <span className="range-value">{config.auto_threshold} chars</span>
        </div>
        <p className="form-help">
          Text shorter than this will use keyboard, longer will use clipboard
        </p>
      </div>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={config.show_overlay}
            onChange={(e) => onChange("show_overlay", e.target.checked)}
          />
          <span>Show overlay window during transcription</span>
        </label>
      </div>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={config.auto_inject}
            onChange={(e) => onChange("auto_inject", e.target.checked)}
          />
          <span>Automatically inject text after transcription</span>
        </label>
      </div>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={config.minimize_to_tray}
            onChange={(e) => onChange("minimize_to_tray", e.target.checked)}
          />
          <span>Minimize to system tray</span>
        </label>
      </div>
    </section>
  );
}
