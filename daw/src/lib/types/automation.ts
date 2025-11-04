// automation.ts
// Trusty Module: TypeScript type definitions for automation system
// These types match the Rust backend types for proper serialization

/**
 * Automation point in time
 */
export interface AutomationPoint {
  id: number;
  time: number;  // MIDI ticks
  value: number; // Normalized 0.0-1.0
}

/**
 * Curve interpolation type
 */
export type CurveType = 'Linear' | 'Bezier' | 'Exponential' | 'Step';

/**
 * Parameter type for automation
 */
export type ParameterType =
  | 'Volume'
  | 'Pan'
  | { CC: number }
  | { Custom: number };

/**
 * Automation curve with points and interpolation
 */
export interface AutomationCurve {
  points: AutomationPoint[];
  curve_type: CurveType;
}

/**
 * Automation lane for a single parameter
 */
export interface AutomationLane {
  id: number;
  track_id: number;
  parameter_type: ParameterType;
  curve: AutomationCurve;
  enabled: boolean;
  name?: string;
}

/**
 * Helper to convert ParameterType to display string
 */
export function parameterTypeToString(param: ParameterType): string {
  if (param === 'Volume') return 'Volume';
  if (param === 'Pan') return 'Pan';
  if (typeof param === 'object' && 'CC' in param) return `CC${param.CC}`;
  if (typeof param === 'object' && 'Custom' in param) return `Custom${param.Custom}`;
  return 'Unknown';
}

/**
 * Helper to get color for parameter type
 */
export function parameterTypeColor(param: ParameterType): string {
  if (param === 'Volume') return '#4ade80';     // green
  if (param === 'Pan') return '#60a5fa';        // blue
  if (typeof param === 'object' && 'CC' in param) return '#a78bfa';  // purple
  if (typeof param === 'object' && 'Custom' in param) return '#fbbf24';  // yellow
  return '#888888';
}

/**
 * Helper to convert value (0.0-1.0) to display value for parameter type
 */
export function normalizedToDisplay(value: number, param: ParameterType): number {
  // Volume and CC: 0-127
  if (param === 'Volume' || (typeof param === 'object' && 'CC' in param)) {
    return Math.round(value * 127);
  }
  // Pan: -64 to +63 (64 = center)
  if (param === 'Pan') {
    return Math.round((value * 127) - 64);
  }
  // Custom: 0-100
  return Math.round(value * 100);
}

/**
 * Helper to convert display value back to normalized (0.0-1.0)
 */
export function displayToNormalized(display: number, param: ParameterType): number {
  // Volume and CC: 0-127
  if (param === 'Volume' || (typeof param === 'object' && 'CC' in param)) {
    return Math.max(0, Math.min(127, display)) / 127;
  }
  // Pan: -64 to +63
  if (param === 'Pan') {
    return (Math.max(-64, Math.min(63, display)) + 64) / 127;
  }
  // Custom: 0-100
  return Math.max(0, Math.min(100, display)) / 100;
}
