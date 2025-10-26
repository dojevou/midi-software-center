/**
 * Base API client with error handling and type safety
 *
 * FEATURES:
 * - Automatic retry with exponential backoff
 * - Request timeout handling (10 seconds)
 * - Connection status monitoring
 * - User-friendly error messages
 */

import { invoke } from '@tauri-apps/api/core';
import { writable, type Readable } from 'svelte/store';

/**
 * API Error class for consistent error handling
 */
export class ApiError extends Error {
  constructor(
    message: string,
    public code?: string,
    public details?: unknown,
    public isRetryable: boolean = false
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/**
 * Connection status type
 */
export type ConnectionStatus = 'connected' | 'disconnected' | 'reconnecting' | 'error';

/**
 * Connection status store - monitors backend connection health
 */
const connectionStatusStore = writable<ConnectionStatus>('connected');
export const connectionStatus: Readable<ConnectionStatus> = {
  subscribe: connectionStatusStore.subscribe
};

/**
 * Update connection status
 */
function setConnectionStatus(status: ConnectionStatus): void {
  connectionStatusStore.set(status);
  console.log(`🔌 Connection status: ${status}`);
}

/**
 * Sleep helper for retry delays
 */
function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Check if error is retryable
 */
function isRetryableError(error: unknown): boolean {
  if (typeof error === 'string') {
    const errorLower = error.toLowerCase();

    // Connection errors - retryable
    if (errorLower.includes('connection') ||
        errorLower.includes('timeout') ||
        errorLower.includes('network') ||
        errorLower.includes('pool') ||
        errorLower.includes('busy') ||
        errorLower.includes('unavailable')) {
      return true;
    }

    // Client errors (4xx) - not retryable
    if (errorLower.includes('not found') ||
        errorLower.includes('permission denied') ||
        errorLower.includes('invalid') ||
        errorLower.includes('duplicate')) {
      return false;
    }
  }

  if (error instanceof Error) {
    return isRetryableError(error.message);
  }

  // Default: retry on unknown errors
  return true;
}

/**
 * Create user-friendly error message from raw error
 */
function createUserFriendlyError(error: unknown, command: string): ApiError {
  let message = 'An unexpected error occurred';
  let isRetryable = false;

  if (typeof error === 'string') {
    message = error;
    isRetryable = isRetryableError(error);
  } else if (error instanceof Error) {
    message = error.message;
    isRetryable = isRetryableError(error);
  } else if (error && typeof error === 'object' && 'message' in error) {
    message = String(error.message);
    isRetryable = isRetryableError(message);
  }

  // Enhance specific error messages
  if (message.toLowerCase().includes('connection')) {
    message = 'Unable to connect to the backend. Please check that the application is running properly.';
    isRetryable = true;
  } else if (message.toLowerCase().includes('timeout')) {
    message = 'The request took too long to complete. The server may be busy or unresponsive.';
    isRetryable = true;
  } else if (message.toLowerCase().includes('pool')) {
    message = 'The database is currently busy. Please try again in a moment.';
    isRetryable = true;
  } else if (message.toLowerCase().includes('not found')) {
    message = 'The requested item was not found.';
    isRetryable = false;
  } else if (message.toLowerCase().includes('permission')) {
    message = 'You do not have permission to perform this action.';
    isRetryable = false;
  }

  return new ApiError(message, command, error, isRetryable);
}

/**
 * Invokes a Tauri command with automatic retry and timeout handling
 *
 * FEATURES:
 * - Automatic retry: Up to 3 attempts with exponential backoff (1s, 2s, 4s)
 * - Timeout: 10 seconds per request
 * - Connection monitoring: Updates connection status
 * - Smart retry: Only retries transient errors (connection, timeout, server errors)
 * - User-friendly errors: Converts technical errors to readable messages
 *
 * RETRY STRATEGY:
 * 1. Attempt 1: Immediate
 * 2. Attempt 2: After 1 second delay
 * 3. Attempt 3: After 2 seconds delay
 * 4. Attempt 4: After 4 seconds delay (capped at 3 retries)
 *
 * TIMEOUT HANDLING:
 * - Each request has 10 second timeout
 * - Timeout errors are retryable
 * - User sees clear timeout message
 *
 * @param command - Tauri command name
 * @param args - Command arguments
 * @returns Promise with command result
 * @throws ApiError with user-friendly message
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  const MAX_RETRIES = 3;
  const TIMEOUT_MS = 10000; // 10 seconds
  const INITIAL_RETRY_DELAY_MS = 1000; // 1 second

  let lastError: unknown = null;

  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    try {
      // Add delay for retries (exponential backoff)
      if (attempt > 0) {
        const delayMs = INITIAL_RETRY_DELAY_MS * Math.pow(2, attempt - 1);
        console.log(`⏳ Retry attempt ${attempt}/${MAX_RETRIES} after ${delayMs}ms delay...`);
        setConnectionStatus('reconnecting');
        await sleep(delayMs);
      }

      // Create timeout promise
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => {
          reject(new Error('Request timeout: The server took too long to respond.'));
        }, TIMEOUT_MS);
      });

      // Race between actual request and timeout
      const result = await Promise.race([
        invoke<T>(command, args),
        timeoutPromise
      ]);

      // Success! Update connection status and return
      if (attempt > 0) {
        console.log(`✓ Request succeeded on attempt ${attempt + 1}/${MAX_RETRIES + 1}`);
      }
      setConnectionStatus('connected');
      return result;

    } catch (error) {
      lastError = error;
      console.error(`API Error [${command}] attempt ${attempt + 1}/${MAX_RETRIES + 1}:`, error);

      // Check if we should retry
      const shouldRetry = attempt < MAX_RETRIES && isRetryableError(error);

      if (!shouldRetry) {
        // Non-retryable error or max retries exhausted
        if (attempt === MAX_RETRIES) {
          console.error(`❌ All retry attempts exhausted for [${command}]`);
          setConnectionStatus('error');
        } else {
          setConnectionStatus('error');
        }

        throw createUserFriendlyError(error, command);
      }

      // Will retry - continue loop
      console.warn(`⚠️  Retryable error detected. Will retry...`);
    }
  }

  // Should never reach here, but just in case
  setConnectionStatus('error');
  throw createUserFriendlyError(lastError, command);
}

/**
 * Type guard to check if error is ApiError
 */
export function isApiError(error: unknown): error is ApiError {
  return error instanceof ApiError;
}

/**
 * Extracts user-friendly error message
 */
export function getErrorMessage(error: unknown): string {
  if (isApiError(error)) {
    return error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  return 'An unexpected error occurred';
}

/**
 * Check if error is retryable (public API)
 */
export function canRetry(error: unknown): boolean {
  if (isApiError(error)) {
    return error.isRetryable;
  }
  return isRetryableError(error);
}

/**
 * Manually set connection status (useful for testing or manual updates)
 */
export function updateConnectionStatus(status: ConnectionStatus): void {
  setConnectionStatus(status);
}

/**
 * Test connection to backend
 *
 * Sends a lightweight test request to verify backend is reachable.
 * Updates connection status based on result.
 *
 * @returns Promise<boolean> - true if connected, false otherwise
 */
export async function testConnection(): Promise<boolean> {
  try {
    await invokeCommand('test_db_connection');
    setConnectionStatus('connected');
    return true;
  } catch (error) {
    console.error('Connection test failed:', error);
    setConnectionStatus('disconnected');
    return false;
  }
}

/**
 * Start periodic connection monitoring
 *
 * Polls backend every 30 seconds to verify connection health.
 * Updates connection status automatically.
 *
 * @param intervalMs - Polling interval in milliseconds (default: 30000 = 30s)
 * @returns Function to stop monitoring
 */
export function startConnectionMonitoring(intervalMs: number = 30000): () => void {
  console.log(`🔄 Starting connection monitoring (every ${intervalMs}ms)`);

  // Initial test
  testConnection();

  // Periodic testing
  const intervalId = setInterval(() => {
    testConnection();
  }, intervalMs);

  // Return cleanup function
  return () => {
    console.log('🛑 Stopping connection monitoring');
    clearInterval(intervalId);
  };
}
