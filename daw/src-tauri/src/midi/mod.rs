   /// MIDI hardware management
   ///
   /// Grown-up Scripts: Handle MIDI device I/O and state management.
   /// Delegates business logic to Trusty Modules in core/midi.

pub mod manager;

pub use manager::MidiManager;
