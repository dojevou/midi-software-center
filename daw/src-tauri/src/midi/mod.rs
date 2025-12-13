/// MIDI hardware management
///
/// Grown-up Scripts: Handle MIDI device I/O and state management.
/// Delegates business logic to Trusty Modules in core/midi.
pub mod export;
pub mod input;
pub mod learn;
pub mod link;
pub mod manager;
pub mod output;
pub mod output_manager;
pub mod playback;
pub mod recording;
pub mod thru;

pub use manager::MidiManager;
#[allow(unused_imports)]
pub use export::{
    ExportOptions, ExportResult, KeyMode, KeySignature, MidiExporter, SmfFormat, TimeSignature,
    TrackMetadata,
};
#[allow(unused_imports)]
pub use input::{MidiInputHandle, MidiInputMessage, MidiInputState, MidiInputDevice, InputDeviceStatus};
#[allow(unused_imports)]
pub use learn::{MidiLearn, MidiMapping, ScalingMode};
#[allow(unused_imports)]
pub use link::{AbletonLink, LinkState};
#[allow(unused_imports)]
pub use output::{
    BankSelectMode, MessagePriority, MidiOutputDevice, MidiOutputManager, MidiOutputMessageType,
    MidiOutputSendState, OutputDeviceStatus, ScheduledMessage,
};
#[allow(unused_imports)]
pub use recording::{
    MidiRecorder, MidiRecordingState, PunchPoint, QuantizeValue, RecordMode, RecordedEvent,
    RecordingState, RecordingStatus,
};
#[allow(unused_imports)]
pub use thru::{
    MidiThru, MidiThruState, ThruConfig, ThruEvent, ThruMode, ThruStatus, VelocityCurve,
};
#[allow(unused_imports)]
pub use output_manager::{
    MidiOutputManager as OutputManager, MidiOutputState as OutputManagerState,
    OutputPriority, TrackOutputAssignment, VirtualOutputPort, OutputQueueStatus,
    DeviceChangeEvent,
};
#[allow(unused_imports)]
pub use playback::{
    PlaybackEngine, PlaybackPosition, PlaybackState, PlaybackState_, LoopRegion,
    // Tauri commands
    play_midi_file, stop_playback, pause_playback, resume_playback,
    set_loop_region, clear_loop, enable_loop, enable_chase,
    set_playback_outputs, add_playback_output, remove_playback_output,
    seek_to_position, get_midi_playback_position, get_midi_playback_state,
    set_playback_bpm, get_playback_bpm, list_playback_outputs,
};
