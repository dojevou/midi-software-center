//! DAW (Digital Audio Workstation) module
//!
//! Contains DSP processing implementations and real-time audio utilities.

pub mod effects;

pub use effects::{
    BuiltInEffect, Compressor, CompressorParams, Delay, DelayParams, EQ3Band, EQ3BandParams,
    Reverb, ReverbParams, StereoBuffer,
};
