import { writable, type Writable } from 'svelte/store';
import type { MidiDevice } from '../types';

/**
 * Available MIDI devices
 */
export const availableDevices: Writable<MidiDevice[]> = writable([]);

/**
 * Currently connected device
 */
export const currentDevice: Writable<MidiDevice | null> = writable(null);

/**
 * MIDI connection status
 */
export const isConnected: Writable<boolean> = writable(false);

/**
 * Connection in progress
 */
export const isConnecting: Writable<boolean> = writable(false);

/**
 * MIDI connection error
 */
export const midiError: Writable<string | null> = writable(null);
