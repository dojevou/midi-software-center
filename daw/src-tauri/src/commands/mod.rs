pub mod analysis;
pub mod automation;
pub mod database;
pub mod daw;
pub mod effect;
pub mod export;
pub mod gear;
pub mod logging;
pub mod midi;
pub mod midi_clock;
pub mod midi_io;
pub mod mixer;
pub mod piano_roll;
pub mod pipeline;
pub mod preferences;
pub mod presets;
pub mod project;
pub mod repair;
// pub mod scripting;  // Temporarily disabled - depends on scripting module
pub mod search;
pub mod sequencer;
pub mod settings;
pub mod system;
pub mod tags;
pub mod trim;
pub mod window;
// pub mod status;

// Re-export types used by app crate (commands can't be re-exported due to Tauri's __cmd__ symbols)
#[allow(unused_imports)] // Used by external crates
pub use automation::AutomationState;
#[allow(unused_imports)] // Used by external crates
pub use daw::DawState;
#[allow(unused_imports)] // Used by external crates
pub use midi_clock::MidiClockState;
#[allow(unused_imports)] // Used by external crates
pub use window::DAWState;

use sqlx::PgPool;
use tauri::{command, State};

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: Option<PgPool>,
}

#[command]
pub async fn initialize_database(state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.db_pool.as_ref().ok_or("Database pool not initialized".to_string())?;
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| format!("Database test failed: {}", e))?;
    Ok(())
}

