use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FontSize {
    Small,
    Medium,
    Large,
}

impl Default for FontSize {
    fn default() -> Self {
        FontSize::Medium
    }
}

impl FontSize {
    pub fn as_pixels(&self) -> u32 {
        match self {
            FontSize::Small => 12,
            FontSize::Medium => 14,
            FontSize::Large => 16,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GridSnapOption {
    Off,
    Bar,
    HalfNote,
    QuarterNote,
    EighthNote,
    SixteenthNote,
    ThirtySecondNote,
}

impl Default for GridSnapOption {
    fn default() -> Self {
        GridSnapOption::SixteenthNote
    }
}

impl GridSnapOption {
    pub fn ticks_per_beat(&self, ppq: u32) -> u32 {
        match self {
            GridSnapOption::Off => 1,
            GridSnapOption::Bar => ppq.saturating_mul(4),
            GridSnapOption::HalfNote => ppq.saturating_mul(2),
            GridSnapOption::QuarterNote => ppq,
            GridSnapOption::EighthNote => ppq / 2,
            GridSnapOption::SixteenthNote => ppq / 4,
            GridSnapOption::ThirtySecondNote => ppq / 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub window_scale: f32,
    pub font_size: FontSize,
    pub grid_snap_default: GridSnapOption,
    pub timeline_zoom_default: f32,
    pub show_toolbar: bool,
    pub show_transport: bool,
    pub show_mixer: bool,
    pub show_browser: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            window_scale: 1.0,
            font_size: FontSize::default(),
            grid_snap_default: GridSnapOption::default(),
            timeline_zoom_default: 1.0,
            show_toolbar: true,
            show_transport: true,
            show_mixer: true,
            show_browser: true,
        }
    }
}

impl DisplaySettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.window_scale = scale;
        self
    }

    pub fn with_font_size(mut self, font_size: FontSize) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.window_scale < 1.0 || self.window_scale > 4.0 {
            return Err("Window scale must be between 1.0 and 4.0".to_string());
        }

        if self.timeline_zoom_default < 0.1 || self.timeline_zoom_default > 10.0 {
            return Err("Timeline zoom must be between 0.1 and 10.0".to_string());
        }

        Ok(())
    }

    pub fn set_window_scale(&mut self, scale: f32) -> Result<(), String> {
        if scale < 1.0 || scale > 4.0 {
            return Err("Window scale must be between 1.0 and 4.0".to_string());
        }
        self.window_scale = scale;
        Ok(())
    }

    pub fn set_font_size(&mut self, font_size: FontSize) {
        self.font_size = font_size;
    }

    pub fn set_grid_snap_default(&mut self, snap: GridSnapOption) {
        self.grid_snap_default = snap;
    }

    pub fn set_timeline_zoom_default(&mut self, zoom: f32) -> Result<(), String> {
        if zoom < 0.1 || zoom > 10.0 {
            return Err("Timeline zoom must be between 0.1 and 10.0".to_string());
        }
        self.timeline_zoom_default = zoom;
        Ok(())
    }

    pub fn set_show_toolbar(&mut self, show: bool) {
        self.show_toolbar = show;
    }

    pub fn set_show_transport(&mut self, show: bool) {
        self.show_transport = show;
    }

    pub fn set_show_mixer(&mut self, show: bool) {
        self.show_mixer = show;
    }

    pub fn set_show_browser(&mut self, show: bool) {
        self.show_browser = show;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_display_settings() {
        let settings = DisplaySettings::default();

        assert_eq!(settings.window_scale, 1.0);
        assert_eq!(settings.font_size, FontSize::Medium);
        assert_eq!(settings.grid_snap_default, GridSnapOption::SixteenthNote);
        assert_eq!(settings.timeline_zoom_default, 1.0);
        assert!(settings.show_toolbar);
        assert!(settings.show_transport);
        assert!(settings.show_mixer);
        assert!(settings.show_browser);
    }

    #[test]
    fn test_font_size_pixels() {
        assert_eq!(FontSize::Small.as_pixels(), 12);
        assert_eq!(FontSize::Medium.as_pixels(), 14);
        assert_eq!(FontSize::Large.as_pixels(), 16);
    }

    #[test]
    fn test_grid_snap_ticks() {
        let ppq = 480;

        assert_eq!(GridSnapOption::Off.ticks_per_beat(ppq), 1);
        assert_eq!(GridSnapOption::Bar.ticks_per_beat(ppq), 1920);
        assert_eq!(GridSnapOption::HalfNote.ticks_per_beat(ppq), 960);
        assert_eq!(GridSnapOption::QuarterNote.ticks_per_beat(ppq), 480);
        assert_eq!(GridSnapOption::EighthNote.ticks_per_beat(ppq), 240);
        assert_eq!(GridSnapOption::SixteenthNote.ticks_per_beat(ppq), 120);
        assert_eq!(GridSnapOption::ThirtySecondNote.ticks_per_beat(ppq), 60);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = DisplaySettings::new()
            .with_scale(2.0)
            .with_font_size(FontSize::Large);

        assert_eq!(settings.window_scale, 2.0);
        assert_eq!(settings.font_size, FontSize::Large);
    }

    #[test]
    fn test_validate_success() {
        let settings = DisplaySettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_scale_too_small() {
        let mut settings = DisplaySettings::default();
        settings.window_scale = 0.5;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_scale_too_large() {
        let mut settings = DisplaySettings::default();
        settings.window_scale = 5.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_zoom_too_small() {
        let mut settings = DisplaySettings::default();
        settings.timeline_zoom_default = 0.05;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_zoom_too_large() {
        let mut settings = DisplaySettings::default();
        settings.timeline_zoom_default = 11.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_set_window_scale_valid() {
        let mut settings = DisplaySettings::default();
        assert!(settings.set_window_scale(2.5).is_ok());
        assert_eq!(settings.window_scale, 2.5);
    }

    #[test]
    fn test_set_window_scale_invalid() {
        let mut settings = DisplaySettings::default();
        assert!(settings.set_window_scale(5.0).is_err());
    }

    #[test]
    fn test_set_font_size() {
        let mut settings = DisplaySettings::default();
        settings.set_font_size(FontSize::Large);
        assert_eq!(settings.font_size, FontSize::Large);
    }

    #[test]
    fn test_set_grid_snap_default() {
        let mut settings = DisplaySettings::default();
        settings.set_grid_snap_default(GridSnapOption::QuarterNote);
        assert_eq!(settings.grid_snap_default, GridSnapOption::QuarterNote);
    }

    #[test]
    fn test_set_timeline_zoom_valid() {
        let mut settings = DisplaySettings::default();
        assert!(settings.set_timeline_zoom_default(2.5).is_ok());
        assert_eq!(settings.timeline_zoom_default, 2.5);
    }

    #[test]
    fn test_set_timeline_zoom_invalid() {
        let mut settings = DisplaySettings::default();
        assert!(settings.set_timeline_zoom_default(15.0).is_err());
    }

    #[test]
    fn test_toggle_panels() {
        let mut settings = DisplaySettings::default();

        settings.set_show_toolbar(false);
        assert!(!settings.show_toolbar);

        settings.set_show_transport(false);
        assert!(!settings.show_transport);

        settings.set_show_mixer(false);
        assert!(!settings.show_mixer);

        settings.set_show_browser(false);
        assert!(!settings.show_browser);
    }
}
