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

#[allow(unused_imports)]
pub use export::{
    ExportOptions, ExportResult, KeyMode, KeySignature, MidiExporter, SmfFormat, TimeSignature,
    TrackMetadata,
};
#[allow(unused_imports)]
pub use input::{
    InputDeviceStatus, MidiInputDevice, MidiInputHandle, MidiInputMessage, MidiInputState,
};
#[allow(unused_imports)]
pub use learn::{MidiLearn, MidiMapping, ScalingMode};
#[allow(unused_imports)]
pub use link::{AbletonLink, LinkState};
pub use manager::MidiManager;
#[allow(unused_imports)]
pub use output::{
    BankSelectMode, MessagePriority, MidiOutputDevice, MidiOutputManager, MidiOutputMessageType,
    MidiOutputSendState, OutputDeviceStatus, ScheduledMessage,
};
#[allow(unused_imports)]
pub use output_manager::{
    DeviceChangeEvent, MidiOutputManager as OutputManager, MidiOutputState as OutputManagerState,
    OutputPriority, OutputQueueStatus, TrackOutputAssignment, VirtualOutputPort,
};
#[allow(unused_imports)]
pub use playback::{
    add_playback_output,
    clear_loop,
    enable_chase,
    enable_loop,
    get_midi_playback_position,
    get_midi_playback_state,
    get_playback_bpm,
    list_playback_outputs,
    pause_playback,
    // Tauri commands
    play_midi_file,
    remove_playback_output,
    resume_playback,
    seek_to_position,
    set_loop_region,
    set_playback_bpm,
    set_playback_outputs,
    stop_playback,
    LoopRegion,
    PlaybackEngine,
    PlaybackPosition,
    PlaybackState,
    PlaybackState_,
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
