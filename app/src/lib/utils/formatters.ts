/**
 * Format bytes to human-readable file size
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Format duration in seconds to MM:SS
 */
export function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Format BPM with decimal places
 */
export function formatBPM(bpm: number | undefined): string {
  if (bpm === undefined) return 'N/A';
  return bpm.toFixed(2) + ' BPM';
}

/**
 * Format tick position to bar:beat:tick
 */
export function formatTick(tick: number, tpqn: number = 480): string {
  const ticksPerBeat = tpqn / 4; // Assuming 4/4
  const bar = Math.floor(tick / (tpqn * 4));
  const beat = Math.floor((tick % (tpqn * 4)) / ticksPerBeat);
  const tickInBeat = tick % ticksPerBeat;
  return `${bar + 1}:${beat + 1}:${Math.floor(tickInBeat)}`;
}

/**
 * Convert MIDI note number to name (e.g., 60 -> "C4")
 */
export function formatMidiNote(note: number): string {
  const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
  const octave = Math.floor(note / 12) - 1;
  const noteName = NOTE_NAMES[note % 12];
  return `${noteName}${octave}`;
}

/**
 * Format MIDI velocity as percentage
 */
export function formatVelocity(velocity: number): string {
  const percentage = Math.round((velocity / 127) * 100);
  return `${percentage}%`;
}

/**
 * Format MIDI channel (0-15 to 1-16)
 */
export function formatChannel(channel: number): string {
  return `Ch ${channel + 1}`;
}

/**
 * Format timestamp to local time
 */
export function formatTimestamp(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleString();
}