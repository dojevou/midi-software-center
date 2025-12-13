//! Lua scripting subsystem
//!
//! Provides embeddable Lua scripting for:
//! - MIDI processing and transformation
//! - Automation and macros
//! - Custom generators
//! - Parameter control

pub mod lua_runtime;

#[allow(unused_imports)]
pub use lua_runtime::{LuaRuntime, ScriptInfo, ScriptType, ScriptAction, ScriptingState};
