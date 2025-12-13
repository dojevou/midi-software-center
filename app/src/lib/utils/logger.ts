/**
 * Production-ready logging utility for MIDI Software Center
 *
 * Features:
 * - Structured logging with context
 * - Performance measurement
 * - Session tracking
 * - Development/production mode switching
 * - Error categorization
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
  level: LogLevel;
  message: string;
  context?: Record<string, unknown>;
  timestamp: string;
  sessionId: string;
}

interface LoggerConfig {
  /** Enable debug level logs (development only) */
  enableDebug: boolean;
  /** Log to console */
  enableConsole: boolean;
  /** Maximum context size in characters */
  maxContextSize: number;
}

/**
 * Production-ready logger with structured output and performance tracking
 */
class Logger {
  private readonly sessionId: string;
  private readonly config: LoggerConfig;
  private readonly isDevelopment: boolean;

  constructor() {
    this.sessionId = this.generateSessionId();
    this.isDevelopment = import.meta.env.DEV;

    this.config = {
      enableDebug: this.isDevelopment,
      enableConsole: true,
      maxContextSize: 10000,
    };
  }

  private generateSessionId(): string {
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).substring(2, 9);
    return `${timestamp}-${random}`;
  }

  private formatContext(context?: Record<string, unknown>): Record<string, unknown> | undefined {
    if (!context) {
      return undefined;
    }

    // Truncate large context objects
    const stringified = JSON.stringify(context);
    if (stringified.length > this.config.maxContextSize) {
      return {
        _truncated: true,
        _originalSize: stringified.length,
        preview: stringified.substring(0, 1000) + '...',
      };
    }

    return context;
  }

  private log(level: LogLevel, message: string, context?: Record<string, unknown>): void {
    // Skip debug logs in production
    if (level === 'debug' && !this.config.enableDebug) {
      return;
    }

    const entry: LogEntry = {
      level,
      message,
      context: this.formatContext(context),
      timestamp: new Date().toISOString(),
      sessionId: this.sessionId,
    };

    if (this.config.enableConsole) {
      this.logToConsole(entry);
    }
  }

  private logToConsole(entry: LogEntry): void {
    const prefix = `[${entry.level.toUpperCase()}] ${entry.timestamp}`;

    switch (entry.level) {
      case 'debug':
        console.debug(prefix, entry.message, entry.context ?? '');
        break;
      case 'info':
        console.info(prefix, entry.message, entry.context ?? '');
        break;
      case 'warn':
        console.warn(prefix, entry.message, entry.context ?? '');
        break;
      case 'error':
        console.error(prefix, entry.message, entry.context ?? '');
        break;
    }
  }

  /**
   * Log debug information (development only)
   */
  debug(message: string, context?: Record<string, unknown>): void {
    this.log('debug', message, context);
  }

  /**
   * Log informational messages
   */
  info(message: string, context?: Record<string, unknown>): void {
    this.log('info', message, context);
  }

  /**
   * Log warning messages
   */
  warn(message: string, context?: Record<string, unknown>): void {
    this.log('warn', message, context);
  }

  /**
   * Log error messages
   */
  error(message: string, context?: Record<string, unknown>): void {
    this.log('error', message, context);
  }

  /**
   * Log an error object with stack trace
   */
  logError(error: unknown, context?: Record<string, unknown>): void {
    if (error instanceof Error) {
      this.error(error.message, {
        ...context,
        name: error.name,
        stack: error.stack,
      });
    } else {
      this.error('Unknown error', {
        ...context,
        error: String(error),
      });
    }
  }

  /**
   * Measure and log the duration of an async operation
   */
  async measure<T>(
    operationName: string,
    operation: () => Promise<T>,
    context?: Record<string, unknown>
  ): Promise<T> {
    const startTime = performance.now();

    try {
      const result = await operation();
      const duration = performance.now() - startTime;

      this.info(`${operationName} completed`, {
        ...context,
        durationMs: Math.round(duration * 100) / 100,
      });

      // Warn about slow operations (> 1 second)
      if (duration > 1000) {
        this.warn(`Slow operation: ${operationName}`, {
          durationMs: duration,
          threshold: 1000,
        });
      }

      return result;
    } catch (error) {
      const duration = performance.now() - startTime;
      this.logError(error, {
        ...context,
        operation: operationName,
        durationMs: Math.round(duration * 100) / 100,
      });
      throw error;
    }
  }

  /**
   * Measure and log the duration of a sync operation
   */
  measureSync<T>(operationName: string, operation: () => T, context?: Record<string, unknown>): T {
    const startTime = performance.now();

    try {
      const result = operation();
      const duration = performance.now() - startTime;

      this.debug(`${operationName} completed`, {
        ...context,
        durationMs: Math.round(duration * 100) / 100,
      });

      return result;
    } catch (error) {
      const duration = performance.now() - startTime;
      this.logError(error, {
        ...context,
        operation: operationName,
        durationMs: Math.round(duration * 100) / 100,
      });
      throw error;
    }
  }

  /**
   * Create a child logger with additional context
   */
  child(context: Record<string, unknown>): ChildLogger {
    return new ChildLogger(this, context);
  }

  /**
   * Get the current session ID
   */
  getSessionId(): string {
    return this.sessionId;
  }
}

/**
 * Child logger that inherits parent context
 */
class ChildLogger {
  constructor(
    private readonly parent: Logger,
    private readonly baseContext: Record<string, unknown>
  ) {}

  private mergeContext(context?: Record<string, unknown>): Record<string, unknown> {
    return { ...this.baseContext, ...context };
  }

  debug(message: string, context?: Record<string, unknown>): void {
    this.parent.debug(message, this.mergeContext(context));
  }

  info(message: string, context?: Record<string, unknown>): void {
    this.parent.info(message, this.mergeContext(context));
  }

  warn(message: string, context?: Record<string, unknown>): void {
    this.parent.warn(message, this.mergeContext(context));
  }

  error(message: string, context?: Record<string, unknown>): void {
    this.parent.error(message, this.mergeContext(context));
  }

  logError(error: unknown, context?: Record<string, unknown>): void {
    this.parent.logError(error, this.mergeContext(context));
  }

  async measure<T>(
    operationName: string,
    operation: () => Promise<T>,
    context?: Record<string, unknown>
  ): Promise<T> {
    return this.parent.measure(operationName, operation, this.mergeContext(context));
  }
}

// Export singleton instance
export const logger = new Logger();

// Export types for external use
export type { LogLevel, LogEntry, LoggerConfig, ChildLogger };
