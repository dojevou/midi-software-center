// MIDI Latency Benchmark
// Measures round-trip latency and throughput for different backends

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::{Duration, Instant};

// Import from the DAW crate
use midi_software_center_daw::hardware::{
    MidiBackendType, MidiManager, PrecisionTimer, TimestampedMidiEvent,
};

/// Benchmark MIDI message creation (zero-allocation)
fn bench_message_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("midi_message_creation");

    group.bench_function("note_on", |b| {
        b.iter(|| {
            TimestampedMidiEvent::note_on(
                black_box(0),
                black_box(60),
                black_box(100),
                black_box(1000),
                black_box("port".to_string()),
            )
        })
    });

    group.bench_function("note_off", |b| {
        b.iter(|| {
            TimestampedMidiEvent::note_off(
                black_box(0),
                black_box(60),
                black_box(64),
                black_box(1000),
                black_box("port".to_string()),
            )
        })
    });

    group.bench_function("control_change", |b| {
        b.iter(|| {
            TimestampedMidiEvent::control_change(
                black_box(0),
                black_box(1),
                black_box(127),
                black_box(1000),
                black_box("port".to_string()),
            )
        })
    });

    group.finish();
}

/// Benchmark MIDI message parsing (wmidi zero-copy)
fn bench_message_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("midi_message_parsing");

    let note_on = TimestampedMidiEvent::note_on(0, 60, 100, 1000, "port".to_string());
    let control_change = TimestampedMidiEvent::control_change(0, 1, 127, 1000, "port".to_string());

    group.bench_function("parse_note_on", |b| {
        b.iter(|| black_box(note_on.parse().unwrap()))
    });

    group.bench_function("parse_control_change", |b| {
        b.iter(|| black_box(control_change.parse().unwrap()))
    });

    group.finish();
}

/// Benchmark precision timer accuracy
fn bench_precision_timer(c: &mut Criterion) {
    let mut group = c.benchmark_group("precision_timer");
    group.sample_size(50); // Fewer samples for timing tests

    // Standard sleep
    let timer_standard = PrecisionTimer::new(false);
    group.bench_function("standard_sleep_100us", |b| {
        b.iter(|| {
            timer_standard.sleep(Duration::from_micros(100));
        })
    });

    // Spin sleep (more precise)
    let timer_spin = PrecisionTimer::new(true);
    group.bench_function("spin_sleep_100us", |b| {
        b.iter(|| {
            timer_spin.sleep(Duration::from_micros(100));
        })
    });

    group.finish();
}

/// Benchmark timer jitter measurement
fn bench_timer_jitter(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_jitter");
    group.sample_size(100);

    for sleep_us in [100, 500, 1000, 5000].iter() {
        let timer = PrecisionTimer::new(true);
        let target = Duration::from_micros(*sleep_us);

        group.bench_with_input(
            BenchmarkId::new("spin_sleep", sleep_us),
            sleep_us,
            |b, &us| {
                b.iter_custom(|iters| {
                    let mut total_jitter = Duration::ZERO;
                    for _ in 0..iters {
                        let start = Instant::now();
                        timer.sleep(Duration::from_micros(us));
                        let elapsed = start.elapsed();
                        let jitter = elapsed.abs_diff(target);
                        total_jitter += jitter;
                    }
                    total_jitter
                })
            },
        );
    }

    group.finish();
}

/// Benchmark MIDI manager initialization
fn bench_manager_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("midi_manager");
    group.sample_size(20); // Fewer samples as this involves I/O

    group.bench_function("init_midir", |b| {
        b.iter(|| {
            let manager = MidiManager::with_backend(MidiBackendType::Midir);
            black_box(manager)
        })
    });

    group.bench_function("init_auto", |b| {
        b.iter(|| {
            let manager = MidiManager::new();
            black_box(manager)
        })
    });

    group.finish();
}

/// Benchmark port enumeration
fn bench_port_enumeration(c: &mut Criterion) {
    let mut group = c.benchmark_group("port_enumeration");
    group.sample_size(20);

    if let Ok(manager) = MidiManager::new() {
        group.bench_function("list_input_ports", |b| {
            b.iter(|| black_box(manager.list_input_ports()))
        });

        group.bench_function("list_output_ports", |b| {
            b.iter(|| black_box(manager.list_output_ports()))
        });
    }

    group.finish();
}

/// Benchmark high-throughput message generation
fn bench_message_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_throughput");

    // Generate 1000 messages
    group.bench_function("generate_1000_notes", |b| {
        b.iter(|| {
            let mut events = Vec::with_capacity(1000);
            for i in 0..1000u64 {
                events.push(TimestampedMidiEvent::note_on(
                    (i % 16) as u8,
                    (i % 128) as u8,
                    100,
                    i * 10,
                    "port".to_string(),
                ));
            }
            black_box(events)
        })
    });

    // Parse 1000 messages
    let events: Vec<_> = (0..1000u64)
        .map(|i| {
            TimestampedMidiEvent::note_on(
                (i % 16) as u8,
                (i % 128) as u8,
                100,
                i * 10,
                "port".to_string(),
            )
        })
        .collect();

    group.bench_function("parse_1000_notes", |b| {
        b.iter(|| {
            for event in &events {
                black_box(event.parse().ok());
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_message_creation,
    bench_message_parsing,
    bench_precision_timer,
    bench_timer_jitter,
    bench_manager_init,
    bench_port_enumeration,
    bench_message_throughput,
);

criterion_main!(benches);
