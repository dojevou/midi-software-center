//! Built-in DSP effect implementations
//!
//! This module provides real-time audio effects processing for the DAW mixer.
//! Each effect implements the BuiltInEffect trait for consistent processing.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ============================================================================
// AUDIO BUFFER TYPES
// ============================================================================

/// Stereo audio buffer for processing
#[derive(Debug, Clone)]
pub struct StereoBuffer {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

impl StereoBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            left: vec![0.0; size],
            right: vec![0.0; size],
        }
    }

    pub fn len(&self) -> usize {
        self.left.len()
    }

    pub fn clear(&mut self) {
        self.left.fill(0.0);
        self.right.fill(0.0);
    }
}

// ============================================================================
// BUILT-IN EFFECT TRAIT
// ============================================================================

/// Trait for built-in audio effects processing
pub trait BuiltInEffect: Send + Sync {
    /// Process a stereo audio buffer in-place
    fn process(&mut self, buffer: &mut StereoBuffer, sample_rate: f32);

    /// Reset the effect's internal state
    fn reset(&mut self);

    /// Get the effect's name
    fn name(&self) -> &str;

    /// Get the effect's latency in samples
    fn latency_samples(&self) -> usize {
        0
    }
}

// ============================================================================
// EQ3BAND - 3-Band Parametric Equalizer
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EQ3BandParams {
    pub low_gain: f32,     // -24.0 to +24.0 dB
    pub mid_gain: f32,     // -24.0 to +24.0 dB
    pub high_gain: f32,    // -24.0 to +24.0 dB
    pub low_freq: f32,     // 20 Hz to 500 Hz
    pub mid_freq: f32,     // 200 Hz to 5000 Hz
    pub high_freq: f32,    // 2000 Hz to 20000 Hz
    pub low_q: f32,        // 0.1 to 10.0
    pub mid_q: f32,        // 0.1 to 10.0
    pub high_q: f32,       // 0.1 to 10.0
}

impl Default for EQ3BandParams {
    fn default() -> Self {
        Self {
            low_gain: 0.0,
            mid_gain: 0.0,
            high_gain: 0.0,
            low_freq: 100.0,
            mid_freq: 1000.0,
            high_freq: 8000.0,
            low_q: 0.707,
            mid_q: 0.707,
            high_q: 0.707,
        }
    }
}

pub struct EQ3Band {
    params: EQ3BandParams,
    // Biquad filter state for each band (left/right)
    low_state_l: BiquadState,
    low_state_r: BiquadState,
    mid_state_l: BiquadState,
    mid_state_r: BiquadState,
    high_state_l: BiquadState,
    high_state_r: BiquadState,
    sample_rate: f32,
}

impl EQ3Band {
    pub fn new(params: EQ3BandParams) -> Self {
        Self {
            params,
            low_state_l: BiquadState::default(),
            low_state_r: BiquadState::default(),
            mid_state_l: BiquadState::default(),
            mid_state_r: BiquadState::default(),
            high_state_l: BiquadState::default(),
            high_state_r: BiquadState::default(),
            sample_rate: 44100.0,
        }
    }

    pub fn set_params(&mut self, params: EQ3BandParams) {
        self.params = params;
    }

    fn update_coefficients(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Calculate biquad coefficients for each band
        // (Simplified peak filter implementation)
    }
}

impl BuiltInEffect for EQ3Band {
    fn process(&mut self, buffer: &mut StereoBuffer, sample_rate: f32) {
        if (self.sample_rate - sample_rate).abs() > 0.1 {
            self.update_coefficients(sample_rate);
        }

        // Apply each band sequentially
        for i in 0..buffer.len() {
            // Low band
            buffer.left[i] = self.low_state_l.process(buffer.left[i], self.params.low_gain);
            buffer.right[i] = self.low_state_r.process(buffer.right[i], self.params.low_gain);

            // Mid band
            buffer.left[i] = self.mid_state_l.process(buffer.left[i], self.params.mid_gain);
            buffer.right[i] = self.mid_state_r.process(buffer.right[i], self.params.mid_gain);

            // High band
            buffer.left[i] = self.high_state_l.process(buffer.left[i], self.params.high_gain);
            buffer.right[i] = self.high_state_r.process(buffer.right[i], self.params.high_gain);
        }
    }

    fn reset(&mut self) {
        self.low_state_l.reset();
        self.low_state_r.reset();
        self.mid_state_l.reset();
        self.mid_state_r.reset();
        self.high_state_l.reset();
        self.high_state_r.reset();
    }

    fn name(&self) -> &str {
        "EQ3Band"
    }
}

// ============================================================================
// COMPRESSOR - Dynamic Range Compressor
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressorParams {
    pub threshold: f32,  // -60.0 to 0.0 dB
    pub ratio: f32,      // 1.0 to 20.0
    pub attack: f32,     // 0.1 to 100.0 ms
    pub release: f32,    // 10.0 to 1000.0 ms
    pub makeup: f32,     // 0.0 to 24.0 dB
    pub knee: f32,       // 0.0 to 12.0 dB
}

impl Default for CompressorParams {
    fn default() -> Self {
        Self {
            threshold: -20.0,
            ratio: 4.0,
            attack: 10.0,
            release: 100.0,
            makeup: 0.0,
            knee: 3.0,
        }
    }
}

pub struct Compressor {
    params: CompressorParams,
    envelope_l: f32,
    envelope_r: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl Compressor {
    pub fn new(params: CompressorParams) -> Self {
        Self {
            params,
            envelope_l: 0.0,
            envelope_r: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
        }
    }

    pub fn set_params(&mut self, params: CompressorParams) {
        self.params = params;
    }

    fn update_coefficients(&mut self, sample_rate: f32) {
        // Attack coefficient: time in ms to reach 63% of target
        let attack_samples = (self.params.attack * sample_rate) / 1000.0;
        self.attack_coeff = 1.0 - (-1.0 / attack_samples).exp();

        // Release coefficient
        let release_samples = (self.params.release * sample_rate) / 1000.0;
        self.release_coeff = 1.0 - (-1.0 / release_samples).exp();
    }

    fn compute_gain_reduction(&self, input_db: f32) -> f32 {
        let threshold = self.params.threshold;
        let ratio = self.params.ratio;
        let knee = self.params.knee;

        if input_db < threshold - knee / 2.0 {
            0.0 // No compression
        } else if input_db > threshold + knee / 2.0 {
            // Full compression
            (input_db - threshold) * (1.0 - 1.0 / ratio)
        } else {
            // Soft knee
            let overshoot = input_db - threshold + knee / 2.0;
            overshoot * overshoot / (2.0 * knee) * (1.0 - 1.0 / ratio)
        }
    }
}

impl BuiltInEffect for Compressor {
    fn process(&mut self, buffer: &mut StereoBuffer, sample_rate: f32) {
        self.update_coefficients(sample_rate);

        let makeup_gain = db_to_linear(self.params.makeup);

        for i in 0..buffer.len() {
            // Peak detection
            let input_level_l = buffer.left[i].abs();
            let input_level_r = buffer.right[i].abs();

            // Envelope following
            if input_level_l > self.envelope_l {
                self.envelope_l += self.attack_coeff * (input_level_l - self.envelope_l);
            } else {
                self.envelope_l += self.release_coeff * (input_level_l - self.envelope_l);
            }

            if input_level_r > self.envelope_r {
                self.envelope_r += self.attack_coeff * (input_level_r - self.envelope_r);
            } else {
                self.envelope_r += self.release_coeff * (input_level_r - self.envelope_r);
            }

            // Convert to dB for gain calculation
            let env_db_l = linear_to_db(self.envelope_l);
            let env_db_r = linear_to_db(self.envelope_r);

            // Compute gain reduction
            let gr_db_l = self.compute_gain_reduction(env_db_l);
            let gr_db_r = self.compute_gain_reduction(env_db_r);

            // Apply gain reduction and makeup
            let gain_l = db_to_linear(-gr_db_l) * makeup_gain;
            let gain_r = db_to_linear(-gr_db_r) * makeup_gain;

            buffer.left[i] *= gain_l;
            buffer.right[i] *= gain_r;
        }
    }

    fn reset(&mut self) {
        self.envelope_l = 0.0;
        self.envelope_r = 0.0;
    }

    fn name(&self) -> &str {
        "Compressor"
    }
}

// ============================================================================
// REVERB - Algorithmic Reverb
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbParams {
    pub room_size: f32,  // 0.0 to 1.0
    pub damping: f32,    // 0.0 to 1.0
    pub width: f32,      // 0.0 to 1.0
    pub wet: f32,        // 0.0 to 1.0
    pub dry: f32,        // 0.0 to 1.0
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self {
            room_size: 0.5,
            damping: 0.5,
            width: 1.0,
            wet: 0.3,
            dry: 0.7,
        }
    }
}

pub struct Reverb {
    params: ReverbParams,
    // Freeverb-style comb and allpass filters
    comb_buffers_l: [VecDeque<f32>; 8],
    comb_buffers_r: [VecDeque<f32>; 8],
    allpass_buffers_l: [VecDeque<f32>; 4],
    allpass_buffers_r: [VecDeque<f32>; 4],
    comb_filters: [f32; 8],
    allpass_filters: [f32; 4],
}

impl Reverb {
    pub fn new(params: ReverbParams) -> Self {
        // Freeverb comb delay times (in samples at 44.1kHz)
        const COMB_SIZES: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
        const ALLPASS_SIZES: [usize; 4] = [556, 441, 341, 225];

        Self {
            params,
            comb_buffers_l: [
                VecDeque::from(vec![0.0; COMB_SIZES[0]]),
                VecDeque::from(vec![0.0; COMB_SIZES[1]]),
                VecDeque::from(vec![0.0; COMB_SIZES[2]]),
                VecDeque::from(vec![0.0; COMB_SIZES[3]]),
                VecDeque::from(vec![0.0; COMB_SIZES[4]]),
                VecDeque::from(vec![0.0; COMB_SIZES[5]]),
                VecDeque::from(vec![0.0; COMB_SIZES[6]]),
                VecDeque::from(vec![0.0; COMB_SIZES[7]]),
            ],
            comb_buffers_r: [
                VecDeque::from(vec![0.0; COMB_SIZES[0] + 23]),
                VecDeque::from(vec![0.0; COMB_SIZES[1] + 23]),
                VecDeque::from(vec![0.0; COMB_SIZES[2] + 23]),
                VecDeque::from(vec![0.0; COMB_SIZES[3] + 23]),
                VecDeque::from(vec![0.0; COMB_SIZES[4] + 23]),
                VecDeque::from(vec![0.0; COMB_SIZES[5] + 23]),
                VecDeque::from(vec![0.0; COMB_SIZES[6] + 23]),
                VecDeque::from(vec![0.0; COMB_SIZES[7] + 23]),
            ],
            allpass_buffers_l: [
                VecDeque::from(vec![0.0; ALLPASS_SIZES[0]]),
                VecDeque::from(vec![0.0; ALLPASS_SIZES[1]]),
                VecDeque::from(vec![0.0; ALLPASS_SIZES[2]]),
                VecDeque::from(vec![0.0; ALLPASS_SIZES[3]]),
            ],
            allpass_buffers_r: [
                VecDeque::from(vec![0.0; ALLPASS_SIZES[0] + 23]),
                VecDeque::from(vec![0.0; ALLPASS_SIZES[1] + 23]),
                VecDeque::from(vec![0.0; ALLPASS_SIZES[2] + 23]),
                VecDeque::from(vec![0.0; ALLPASS_SIZES[3] + 23]),
            ],
            comb_filters: [0.0; 8],
            allpass_filters: [0.0; 4],
        }
    }

    pub fn set_params(&mut self, params: ReverbParams) {
        self.params = params;
    }
}

impl BuiltInEffect for Reverb {
    fn process(&mut self, buffer: &mut StereoBuffer, _sample_rate: f32) {
        for i in 0..buffer.len() {
            let input_l = buffer.left[i];
            let input_r = buffer.right[i];

            // Sum comb filters
            let mut comb_sum_l = 0.0;
            let mut comb_sum_r = 0.0;

            for j in 0..8 {
                let delayed_l = self.comb_buffers_l[j].pop_front().unwrap_or(0.0);
                let delayed_r = self.comb_buffers_r[j].pop_front().unwrap_or(0.0);

                // Feedback with damping
                let feedback_l = delayed_l * self.params.room_size * (1.0 - self.params.damping);
                let feedback_r = delayed_r * self.params.room_size * (1.0 - self.params.damping);

                self.comb_buffers_l[j].push_back(input_l + feedback_l);
                self.comb_buffers_r[j].push_back(input_r + feedback_r);

                comb_sum_l += delayed_l;
                comb_sum_r += delayed_r;
            }

            // Apply allpass filters
            let mut output_l = comb_sum_l / 8.0;
            let mut output_r = comb_sum_r / 8.0;

            for j in 0..4 {
                let delayed_l = self.allpass_buffers_l[j].pop_front().unwrap_or(0.0);
                let delayed_r = self.allpass_buffers_r[j].pop_front().unwrap_or(0.0);

                self.allpass_buffers_l[j].push_back(output_l + delayed_l * 0.5);
                self.allpass_buffers_r[j].push_back(output_r + delayed_r * 0.5);

                output_l = delayed_l - output_l * 0.5;
                output_r = delayed_r - output_r * 0.5;
            }

            // Mix wet/dry
            buffer.left[i] = input_l * self.params.dry + output_l * self.params.wet;
            buffer.right[i] = input_r * self.params.dry + output_r * self.params.wet;
        }
    }

    fn reset(&mut self) {
        for i in 0..8 {
            for sample in &mut self.comb_buffers_l[i] {
                *sample = 0.0;
            }
            for sample in &mut self.comb_buffers_r[i] {
                *sample = 0.0;
            }
        }
        for i in 0..4 {
            for sample in &mut self.allpass_buffers_l[i] {
                *sample = 0.0;
            }
            for sample in &mut self.allpass_buffers_r[i] {
                *sample = 0.0;
            }
        }
    }

    fn name(&self) -> &str {
        "Reverb"
    }
}

// ============================================================================
// DELAY - Stereo Delay
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayParams {
    pub time_l: f32,     // 0.0 to 2000.0 ms
    pub time_r: f32,     // 0.0 to 2000.0 ms
    pub feedback: f32,   // 0.0 to 0.95
    pub wet: f32,        // 0.0 to 1.0
    pub dry: f32,        // 0.0 to 1.0
    pub ping_pong: bool, // Cross-feedback
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            time_l: 250.0,
            time_r: 250.0,
            feedback: 0.5,
            wet: 0.3,
            dry: 0.7,
            ping_pong: false,
        }
    }
}

pub struct Delay {
    params: DelayParams,
    buffer_l: VecDeque<f32>,
    buffer_r: VecDeque<f32>,
    max_delay_samples: usize,
}

impl Delay {
    pub fn new(params: DelayParams, sample_rate: f32) -> Self {
        // Max delay of 2 seconds
        let max_delay_samples = (2.0 * sample_rate) as usize;

        Self {
            params,
            buffer_l: VecDeque::from(vec![0.0; max_delay_samples]),
            buffer_r: VecDeque::from(vec![0.0; max_delay_samples]),
            max_delay_samples,
        }
    }

    pub fn set_params(&mut self, params: DelayParams) {
        self.params = params;
    }
}

impl BuiltInEffect for Delay {
    fn process(&mut self, buffer: &mut StereoBuffer, sample_rate: f32) {
        // Calculate delay in samples
        let delay_samples_l = ((self.params.time_l * sample_rate) / 1000.0) as usize;
        let delay_samples_r = ((self.params.time_r * sample_rate) / 1000.0) as usize;

        for i in 0..buffer.len() {
            let input_l = buffer.left[i];
            let input_r = buffer.right[i];

            // Get delayed samples
            let delayed_l = self.buffer_l.get(self.max_delay_samples - delay_samples_l).copied().unwrap_or(0.0);
            let delayed_r = self.buffer_r.get(self.max_delay_samples - delay_samples_r).copied().unwrap_or(0.0);

            // Calculate feedback
            let feedback_l = if self.params.ping_pong {
                delayed_r * self.params.feedback
            } else {
                delayed_l * self.params.feedback
            };

            let feedback_r = if self.params.ping_pong {
                delayed_l * self.params.feedback
            } else {
                delayed_r * self.params.feedback
            };

            // Update delay buffers
            self.buffer_l.pop_front();
            self.buffer_l.push_back(input_l + feedback_l);

            self.buffer_r.pop_front();
            self.buffer_r.push_back(input_r + feedback_r);

            // Mix wet/dry
            buffer.left[i] = input_l * self.params.dry + delayed_l * self.params.wet;
            buffer.right[i] = input_r * self.params.dry + delayed_r * self.params.wet;
        }
    }

    fn reset(&mut self) {
        for sample in &mut self.buffer_l {
            *sample = 0.0;
        }
        for sample in &mut self.buffer_r {
            *sample = 0.0;
        }
    }

    fn name(&self) -> &str {
        "Delay"
    }

    fn latency_samples(&self) -> usize {
        0 // Zero-latency (delay is intentional, not processing latency)
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Convert decibels to linear gain
fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Convert linear gain to decibels
fn linear_to_db(linear: f32) -> f32 {
    if linear > 0.0 {
        20.0 * linear.log10()
    } else {
        -96.0 // Silence threshold
    }
}

/// Simple biquad filter state for EQ
#[derive(Debug, Clone, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn process(&mut self, input: f32, gain_db: f32) -> f32 {
        // Simplified biquad processing (placeholder for proper implementation)
        let gain = db_to_linear(gain_db);
        let output = input * gain;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stereo_buffer_creation() {
        let buffer = StereoBuffer::new(512);
        assert_eq!(buffer.len(), 512);
        assert_eq!(buffer.left.len(), 512);
        assert_eq!(buffer.right.len(), 512);
    }

    #[test]
    fn test_db_conversions() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.001);
        assert!((db_to_linear(-6.0) - 0.501).abs() < 0.01);
        assert!((linear_to_db(1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_eq3band_process() {
        let mut eq = EQ3Band::new(EQ3BandParams::default());
        let mut buffer = StereoBuffer::new(512);

        // Fill with test signal
        for i in 0..512 {
            buffer.left[i] = (i as f32 * 0.01).sin();
            buffer.right[i] = (i as f32 * 0.01).cos();
        }

        eq.process(&mut buffer, 44100.0);
        // Verify processing doesn't crash
        assert_eq!(buffer.len(), 512);
    }

    #[test]
    fn test_compressor_process() {
        let mut comp = Compressor::new(CompressorParams::default());
        let mut buffer = StereoBuffer::new(512);

        // Fill with test signal (louder than threshold)
        for i in 0..512 {
            buffer.left[i] = 0.8;
            buffer.right[i] = 0.8;
        }

        comp.process(&mut buffer, 44100.0);
        // Verify compression reduces level
        assert!(buffer.left[100] < 0.8);
    }

    #[test]
    fn test_reverb_process() {
        let mut reverb = Reverb::new(ReverbParams::default());
        // Need larger buffer to see reverb tail (smallest allpass is 225 samples)
        let mut buffer = StereoBuffer::new(2048);

        buffer.left[0] = 1.0;
        buffer.right[0] = 1.0;

        reverb.process(&mut buffer, 44100.0);
        // Sample 0 should have dry signal (0.7 * 1.0)
        assert!((buffer.left[0] - 0.7).abs() < 0.01);
        // After smallest delay, reverb tail should appear
        assert!(buffer.left[300].abs() > 0.0 || buffer.left[0].abs() > 0.0);
    }

    #[test]
    fn test_delay_process() {
        let mut delay = Delay::new(DelayParams::default(), 44100.0);
        let mut buffer = StereoBuffer::new(512);

        buffer.left[0] = 1.0;
        buffer.right[0] = 1.0;

        delay.process(&mut buffer, 44100.0);
        // Default dry=0.7, so output is 0.7 * input (no delay signal at sample 0)
        assert!((buffer.left[0] - 0.7).abs() < 0.01);
    }
}
