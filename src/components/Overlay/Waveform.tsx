import { useMemo, useEffect, useState } from "react";

interface WaveformProps {
  level: number; // 0-1
  active: boolean;
}

export function Waveform({ level, active }: WaveformProps) {
  const [animatedLevel, setAnimatedLevel] = useState(0);

  // Smooth animation for audio level
  useEffect(() => {
    if (!active) {
      setAnimatedLevel(0);
      return;
    }

    // Add some randomness to make the animation more natural
    const targetLevel = level * (0.8 + Math.random() * 0.4);
    setAnimatedLevel((prev) => prev + (targetLevel - prev) * 0.3);
  }, [level, active]);

  const bars = useMemo(() => {
    const count = 5;
    return Array.from({ length: count }, (_, i) => {
      const baseHeight = 0.2;
      // Create a wave pattern - center bars are higher
      const centerFactor = 1 - Math.abs(i - (count - 1) / 2) / ((count - 1) / 2);
      const variation = centerFactor * animatedLevel;
      // Add slight phase offset for each bar
      const phaseOffset = Math.sin((Date.now() / 150 + i * 0.5)) * 0.1 * animatedLevel;
      return Math.max(baseHeight, Math.min(1, variation + phaseOffset + baseHeight));
    });
  }, [animatedLevel]);

  return (
    <div className={`waveform ${active ? "waveform-active" : ""}`}>
      {bars.map((height, i) => (
        <div
          key={i}
          className="waveform-bar"
          style={{
            height: `${height * 100}%`,
            animationDelay: `${i * 0.1}s`,
            opacity: active ? 1 : 0.3,
          }}
        />
      ))}
    </div>
  );
}
