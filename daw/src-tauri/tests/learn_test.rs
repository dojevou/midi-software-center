//! MIDI Learn tests

use midi_software_center_daw::midi::learn::{
    MappingTarget, MidiLearn, MidiMapping, MidiMessageType, MidiSource, PickupMode, ScalingMode,
};

#[test]
fn test_learn_creation() {
    let (learn, _event_rx, _param_rx) = MidiLearn::new();
    assert!(!learn.is_learning());
    assert!(learn.list_mappings().is_empty());
}

#[test]
fn test_learn_mode() {
    let (learn, _event_rx, _param_rx) = MidiLearn::new();
    let target = MappingTarget::Parameter { path: "mixer/channel/1/volume".to_string() };
    learn.start_learn(target);
    assert!(learn.is_learning());
    learn.cancel_learn();
    assert!(!learn.is_learning());
}

#[test]
fn test_scaling_linear() {
    let mode = ScalingMode::Linear { min: 0.0, max: 1.0 };
    assert_eq!(mode.scale(0, None), 0.0);
    assert_eq!(mode.scale(127, None), 1.0);
    assert!((mode.scale(64, None) - 0.504).abs() < 0.01);
}

#[test]
fn test_scaling_linear_range() {
    let mode = ScalingMode::Linear { min: 0.0, max: 100.0 };
    assert_eq!(mode.scale(0, None), 0.0);
    assert_eq!(mode.scale(127, None), 100.0);
    assert!((mode.scale(64, None) - 50.4).abs() < 0.5);
}

#[test]
fn test_scaling_toggle() {
    let mode = ScalingMode::Toggle;
    assert_eq!(mode.scale(0, None), 0.0);
    assert_eq!(mode.scale(63, None), 0.0);
    assert_eq!(mode.scale(64, None), 1.0);
    assert_eq!(mode.scale(127, None), 1.0);
}

#[test]
fn test_scaling_momentary() {
    let mode = ScalingMode::Momentary;
    assert_eq!(mode.scale(0, None), 0.0);
    assert_eq!(mode.scale(1, None), 1.0);
    assert_eq!(mode.scale(127, None), 1.0);
}

#[test]
fn test_scaling_stepped() {
    let mode = ScalingMode::Stepped { values: vec![0.0, 0.5, 1.0] };
    assert_eq!(mode.scale(0, None), 0.0);
    assert_eq!(mode.scale(64, None), 0.5);
    assert_eq!(mode.scale(127, None), 1.0);
}

#[test]
fn test_scaling_logarithmic() {
    let mode = ScalingMode::Logarithmic { min: 1.0, max: 100.0 };
    let low = mode.scale(0, None);
    let mid = mode.scale(64, None);
    let high = mode.scale(127, None);

    // Log scaling: low should be ~1, high should be ~100
    assert!((low - 1.0).abs() < 0.1);
    assert!((high - 100.0).abs() < 0.1);
    // Mid should be geometric mean-ish (sqrt(1 * 100) = 10)
    assert!(mid > 5.0 && mid < 20.0);
}

#[test]
fn test_scaling_exponential() {
    let mode = ScalingMode::Exponential { min: 0.0, max: 1.0, curve: 2.0 };
    let low = mode.scale(0, None);
    let mid = mode.scale(64, None);
    let high = mode.scale(127, None);

    assert_eq!(low, 0.0);
    assert!((high - 1.0).abs() < 0.01);
    // With curve 2.0, mid should be less than 0.5 (curved)
    assert!(mid < 0.5);
}

#[test]
fn test_export_import() {
    let (learn, _event_rx, _param_rx) = MidiLearn::new();
    let json = learn.export_mappings();
    assert!(json.starts_with('['));
    assert!(json.ends_with(']'));

    // Import empty array
    let count = learn.import_mappings(&json).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_add_and_list_mapping() {
    let (learn, _event_rx, _param_rx) = MidiLearn::new();

    let source = MidiSource {
        device_id: "test-device".to_string(),
        message_type: MidiMessageType::ControlChange,
        channel: Some(0),
        data1: Some(1),
    };
    let target = MappingTarget::Parameter { path: "test/param".to_string() };
    let mapping = MidiMapping::new("Test Mapping", source, target);

    learn.add_mapping(mapping);

    let mappings = learn.list_mappings();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].name, "Test Mapping");
}

#[test]
fn test_remove_mapping() {
    let (learn, _event_rx, _param_rx) = MidiLearn::new();

    let source = MidiSource {
        device_id: "test".to_string(),
        message_type: MidiMessageType::ControlChange,
        channel: Some(0),
        data1: Some(1),
    };
    let target = MappingTarget::Parameter { path: "test".to_string() };
    let mapping = MidiMapping::new("Test", source, target);
    let id = mapping.id.clone();

    learn.add_mapping(mapping);
    assert_eq!(learn.list_mappings().len(), 1);

    learn.remove_mapping(&id);
    assert!(learn.list_mappings().is_empty());
}

#[test]
fn test_clear_mappings() {
    let (learn, _event_rx, _param_rx) = MidiLearn::new();

    for i in 0..3 {
        let source = MidiSource {
            device_id: "test".to_string(),
            message_type: MidiMessageType::ControlChange,
            channel: Some(0),
            data1: Some(i),
        };
        let target = MappingTarget::Parameter { path: format!("test/{}", i) };
        learn.add_mapping(MidiMapping::new(&format!("Test {}", i), source, target));
    }

    assert_eq!(learn.list_mappings().len(), 3);
    learn.clear_mappings();
    assert!(learn.list_mappings().is_empty());
}

#[test]
fn test_mapping_creation() {
    let source = MidiSource {
        device_id: "device".to_string(),
        message_type: MidiMessageType::NoteOn,
        channel: Some(0),
        data1: Some(60),
    };
    let target = MappingTarget::Parameter { path: "path".to_string() };
    let mapping = MidiMapping::new("Test", source.clone(), target);

    assert!(mapping.enabled);
    assert!(!mapping.id.is_empty());
    assert_eq!(mapping.name, "Test");
    assert!(matches!(mapping.pickup_mode, PickupMode::Jump));
}

#[test]
fn test_event_subscription() {
    let (learn, _event_rx, _param_rx) = MidiLearn::new();
    let _rx = learn.subscribe();
    // Verify subscription works
}
