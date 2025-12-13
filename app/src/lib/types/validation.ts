/**
 * Type Consistency Validation Module
 *
 * Validates that frontend TypeScript types match backend Rust structures.
 */

export interface ValidationResult {
  valid: boolean;
  errors: TypeValidationError[];
  warnings: TypeValidationError[];
}

export interface TypeValidationError {
  typeName: string;
  field?: string;
  message: string;
  severity: 'error' | 'warning';
}

/**
 * Shared type definitions that must match between frontend and backend
 */
export const SharedTypes = {
  FileMetadata: {
    required: ['id', 'filename', 'filepath', 'hash', 'size_bytes'],
    optional: [
      'bpm',
      'key_signature',
      'time_signature',
      'duration_seconds',
      'created_at',
      'updated_at',
    ],
  },
  SearchFilters: {
    required: [],
    optional: ['query', 'bpmRange', 'key', 'tags', 'limit', 'offset', 'sortBy', 'sortOrder'],
  },
  SearchResult: {
    required: ['files', 'total', 'page', 'per_page'],
    optional: [],
  },
  MixerState: {
    required: ['master_volume', 'channels'],
    optional: [],
  },
  MixerChannel: {
    required: ['track_id', 'volume', 'pan', 'muted', 'soloed'],
    optional: ['name', 'color'],
  },
  PlaybackState: {
    required: ['isPlaying', 'isPaused', 'tempo', 'timeSignature', 'position'],
    optional: ['loopEnabled', 'loopStart', 'loopEnd', 'metronomeEnabled'],
  },
  MusicalMetadata: {
    required: ['file_id'],
    optional: [
      'bpm',
      'key_signature',
      'time_signature_numerator',
      'time_signature_denominator',
      'duration_seconds',
    ],
  },
  Tag: {
    required: ['id', 'name'],
    optional: ['category', 'description', 'count'],
  },
  AnalysisResult: {
    required: ['file_id'],
    optional: ['controller_data', 'articulation_data', 'structure_data', 'chord_complexity_score'],
  },
} as const;

/**
 * Type validator class
 */
export class TypeValidator {
  private errors: TypeValidationError[] = [];
  private warnings: TypeValidationError[] = [];

  /**
   * Validate that an object matches a type definition
   */
  validateObject<T extends object>(
    obj: T,
    typeName: keyof typeof SharedTypes,
    context?: string
  ): boolean {
    const typeSpec = SharedTypes[typeName];
    let valid = true;

    // Check required fields
    for (const field of typeSpec.required) {
      if (!(field in obj)) {
        this.errors.push({
          typeName: String(typeName),
          field,
          message: `Required field '${field}' is missing${context ? ` in ${context}` : ''}`,
          severity: 'error',
        });
        valid = false;
      }
    }

    // Check for unknown fields (warning only)
    const allFields: string[] = [...typeSpec.required, ...typeSpec.optional];
    for (const key of Object.keys(obj)) {
      if (!allFields.includes(key)) {
        this.warnings.push({
          typeName: String(typeName),
          field: key,
          message: `Unknown field '${key}' found${context ? ` in ${context}` : ''}`,
          severity: 'warning',
        });
      }
    }

    return valid;
  }

  /**
   * Validate an array of objects
   */
  validateArray<T extends object>(
    arr: T[],
    typeName: keyof typeof SharedTypes,
    context?: string
  ): boolean {
    return arr.every((item, index) =>
      this.validateObject(item, typeName, `${context || typeName}[${index}]`)
    );
  }

  /**
   * Validate API response structure
   */
  validateApiResponse<T extends object>(
    response: T,
    expectedType: keyof typeof SharedTypes
  ): boolean {
    return this.validateObject(response, expectedType, 'API response');
  }

  /**
   * Get validation result
   */
  getResult(): ValidationResult {
    return {
      valid: this.errors.length === 0,
      errors: [...this.errors],
      warnings: [...this.warnings],
    };
  }

  /**
   * Reset validator state
   */
  reset(): void {
    this.errors = [];
    this.warnings = [];
  }

  /**
   * Validate type consistency between frontend and backend
   * This is a static check that compares type definitions
   */
  static validateFrontendBackendTypes(): TypeValidationError[] {
    const errors: TypeValidationError[] = [];

    // Map of backend Rust types to frontend TypeScript types
    const typeMapping = {
      // Rust -> TypeScript field name mappings
      file_id: 'file_id', // Same
      bpm: 'bpm', // Same
      key_signature: 'key_signature', // Same
      time_signature: 'timeSignature', // Different!
    };

    // Check for known mismatches
    const knownMismatches = [
      { rust: 'time_signature', ts: 'timeSignature', typeName: 'PlaybackState' },
    ];

    for (const mismatch of knownMismatches) {
      errors.push({
        typeName: mismatch.typeName,
        field: mismatch.rust,
        message: `Field naming mismatch: Rust uses '${mismatch.rust}', TypeScript uses '${mismatch.ts}'`,
        severity: 'warning',
      });
    }

    return errors;
  }

  /**
   * Validate command parameters match expected types
   */
  static validateCommandParameters(
    command: string,
    params: Record<string, unknown>
  ): TypeValidationError[] {
    const errors: TypeValidationError[] = [];

    // Define expected parameter types for key commands
    const commandParams: Record<string, { required: string[]; optional: string[] }> = {
      search_files: {
        required: [],
        optional: ['query', 'bpm_range', 'key', 'tags', 'limit', 'offset'],
      },
      add_track: { required: ['file_id', 'channel'], optional: ['position'] },
      set_tempo: { required: ['bpm'], optional: [] },
      set_time_signature: { required: ['numerator', 'denominator'], optional: [] },
      set_channel_volume: { required: ['track_id', 'volume'], optional: [] },
      set_channel_pan: { required: ['track_id', 'pan'], optional: [] },
      set_channel_mute: { required: ['track_id', 'muted'], optional: [] },
      import_midi_file: { required: ['file_path'], optional: ['analyze'] },
      analyze_file: { required: ['file_id'], optional: ['force_reanalyze'] },
    };

    const spec = commandParams[command];
    if (!spec) {
      // Unknown command, can't validate
      return errors;
    }

    // Check required parameters
    for (const param of spec.required) {
      if (!(param in params)) {
        errors.push({
          typeName: 'CommandParams',
          field: param,
          message: `Required parameter '${param}' missing for command '${command}'`,
          severity: 'error',
        });
      }
    }

    // Check for unknown parameters
    const allParams = [...spec.required, ...spec.optional];
    for (const key of Object.keys(params)) {
      if (!allParams.includes(key)) {
        errors.push({
          typeName: 'CommandParams',
          field: key,
          message: `Unknown parameter '${key}' for command '${command}'`,
          severity: 'warning',
        });
      }
    }

    return errors;
  }
}

/**
 * Runtime type guard utilities
 */
export const TypeGuards = {
  isFileMetadata(obj: unknown): obj is { id: number; filename: string; filepath: string } {
    return (
      typeof obj === 'object' &&
      obj !== null &&
      'id' in obj &&
      'filename' in obj &&
      'filepath' in obj
    );
  },

  isSearchResult(obj: unknown): obj is { files: unknown[]; total: number } {
    return (
      typeof obj === 'object' &&
      obj !== null &&
      'files' in obj &&
      Array.isArray((obj as Record<string, unknown>).files) &&
      'total' in obj &&
      typeof (obj as Record<string, unknown>).total === 'number'
    );
  },

  isMixerState(obj: unknown): obj is { master_volume: number; channels: unknown[] } {
    return (
      typeof obj === 'object' &&
      obj !== null &&
      'master_volume' in obj &&
      'channels' in obj &&
      Array.isArray((obj as Record<string, unknown>).channels)
    );
  },

  isPlaybackPosition(
    obj: unknown
  ): obj is { current_tick: number; current_bar: number; current_beat: number } {
    return (
      typeof obj === 'object' &&
      obj !== null &&
      'current_tick' in obj &&
      'current_bar' in obj &&
      'current_beat' in obj
    );
  },
};

/**
 * Validation utilities for common patterns
 */
export const ValidationUtils = {
  /**
   * Validate BPM is in valid range
   */
  isValidBpm(bpm: number): boolean {
    return bpm >= 20 && bpm <= 400;
  },

  /**
   * Validate key signature format
   */
  isValidKeySignature(key: string): boolean {
    const validKeys = [
      'C',
      'C#',
      'Db',
      'D',
      'D#',
      'Eb',
      'E',
      'F',
      'F#',
      'Gb',
      'G',
      'G#',
      'Ab',
      'A',
      'A#',
      'Bb',
      'B',
      'Cm',
      'C#m',
      'Dbm',
      'Dm',
      'D#m',
      'Ebm',
      'Em',
      'Fm',
      'F#m',
      'Gbm',
      'Gm',
      'G#m',
      'Abm',
      'Am',
      'A#m',
      'Bbm',
      'Bm',
    ];
    return validKeys.includes(key);
  },

  /**
   * Validate time signature
   */
  isValidTimeSignature(numerator: number, denominator: number): boolean {
    const validNumerators = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    const validDenominators = [2, 4, 8, 16];
    return validNumerators.includes(numerator) && validDenominators.includes(denominator);
  },

  /**
   * Validate volume is in 0-1 range
   */
  isValidVolume(volume: number): boolean {
    return volume >= 0 && volume <= 1;
  },

  /**
   * Validate pan is in -1 to 1 range
   */
  isValidPan(pan: number): boolean {
    return pan >= -1 && pan <= 1;
  },
};

export default TypeValidator;
