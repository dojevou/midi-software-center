//! MIDI Software Center Verification Runner
//!
//! Comprehensive verification tool that runs all verification checks
//! and provides detailed reporting.

use clap::{Arg, Command};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::process::ExitCode;

use verification::{
    BuildVerifier, ConfigVerifier, DependencyVerifier, HealthVerifier, IntegrationVerifier,
    MigrationVerifier, PathVerifier, VerificationOptions, VerificationResult,
};

#[tokio::main]
async fn main() -> ExitCode {
    let matches = Command::new("midi-verification")
        .version("1.0.0")
        .author("MIDI Software Center")
        .about("Comprehensive project verification tool")
        .arg(
            Arg::new("all")
                .long("all")
                .help("Run all verification checks (default)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("migrations")
                .long("migrations")
                .help("Verify database migrations only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("paths")
                .long("paths")
                .help("Verify file paths only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .help("Verify configuration only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dependencies")
                .long("dependencies")
                .help("Verify dependencies only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("build")
                .long("build")
                .help("Verify build environment only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("health")
                .long("health")
                .help("Verify system health only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("integration")
                .long("integration")
                .help("Verify integration only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("skip-tests")
                .long("skip-tests")
                .help("Skip running tests during build verification")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ci")
                .long("ci")
                .help("CI mode - fail fast on any error")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Determine which checks to run
    let run_all = matches.get_flag("all")
        || !matches.get_flag("migrations")
            && !matches.get_flag("paths")
            && !matches.get_flag("config")
            && !matches.get_flag("dependencies")
            && !matches.get_flag("build")
            && !matches.get_flag("health")
            && !matches.get_flag("integration");

    let options = VerificationOptions {
        skip_migrations: !run_all && !matches.get_flag("migrations"),
        skip_paths: !run_all && !matches.get_flag("paths"),
        skip_config: !run_all && !matches.get_flag("config"),
        skip_dependencies: !run_all && !matches.get_flag("dependencies"),
        skip_build: !run_all && !matches.get_flag("build"),
        skip_health: !run_all && !matches.get_flag("health"),
        skip_integration: !run_all && !matches.get_flag("integration"),
        run_tests: !matches.get_flag("skip-tests"),
        verbose: matches.get_flag("verbose"),
    };

    let ci_mode = matches.get_flag("ci");

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     MIDI Software Center - Verification Suite                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Connect to database if needed
    let pool = if !options.skip_migrations || !options.skip_health || !options.skip_integration {
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
        });

        match PgPoolOptions::new().max_connections(5).connect(&database_url).await {
            Ok(pool) => {
                println!("✓ Database connection established");
                Some(pool)
            },
            Err(e) => {
                eprintln!("✗ Failed to connect to database: {}", e);
                if ci_mode {
                    return ExitCode::FAILURE;
                }
                None
            },
        }
    } else {
        None
    };

    println!();

    // Run verifications
    let result = run_verifications(&options, pool.as_ref()).await;

    // Print results
    println!();
    println!("{}", result);

    // Summary
    if result.all_passed() {
        println!("✓ All verification checks passed!");
        ExitCode::SUCCESS
    } else {
        let failed = result.failed_count();
        println!("✗ {} verification check(s) failed", failed);
        ExitCode::FAILURE
    }
}

async fn run_verifications(
    options: &VerificationOptions,
    pool: Option<&sqlx::PgPool>,
) -> VerificationResult {
    // Migration verification
    let migrations = if !options.skip_migrations {
        print_header("Database Migrations");
        if let Some(pool) = pool {
            MigrationVerifier::get_summary(pool).await
        } else {
            println!("  Skipped (no database connection)");
            verification::MigrationSummary {
                schema_valid: false,
                indexes_valid: false,
                foreign_keys_valid: false,
            }
        }
    } else {
        println!("⊘ Skipping migration verification");
        verification::MigrationSummary {
            schema_valid: true,
            indexes_valid: true,
            foreign_keys_valid: true,
        }
    };

    // Path verification
    let paths = if !options.skip_paths {
        print_header("File Paths");
        PathVerifier::get_summary()
    } else {
        println!("⊘ Skipping path verification");
        verification::PathSummary {
            application_paths_valid: true,
            critical_files_present: true,
            data_paths_accessible: true,
            build_outputs_present: true,
        }
    };

    // Configuration verification
    let config = if !options.skip_config {
        print_header("Configuration");
        ConfigVerifier::get_summary()
    } else {
        println!("⊘ Skipping configuration verification");
        verification::ConfigSummary {
            environment_valid: true,
            tauri_valid: true,
            cargo_valid: true,
            database_valid: true,
        }
    };

    // Dependency verification
    let dependencies = if !options.skip_dependencies {
        print_header("Dependencies");
        DependencyVerifier::get_summary()
    } else {
        println!("⊘ Skipping dependency verification");
        verification::DependencySummary {
            workspace_consistent: true,
            frontend_valid: true,
            tauri_compatible: true,
        }
    };

    // Build verification
    let build = if !options.skip_build {
        print_header("Build Environment");
        BuildVerifier::get_summary()
    } else {
        println!("⊘ Skipping build verification");
        verification::BuildSummary {
            environment_valid: true,
            compilation_passes: true,
            frontend_builds: true,
            formatting_correct: true,
        }
    };

    // Health verification
    let health = if !options.skip_health {
        print_header("System Health");
        if let Some(pool) = pool {
            HealthVerifier::get_full_summary(pool).await
        } else {
            HealthVerifier::get_summary()
        }
    } else {
        println!("⊘ Skipping health verification");
        verification::HealthSummary {
            filesystem_healthy: true,
            resources_healthy: true,
            database_healthy: true,
        }
    };

    // Integration verification
    let integration = if !options.skip_integration {
        print_header("Integration");
        if let Some(pool) = pool {
            IntegrationVerifier::get_summary(pool).await
        } else {
            println!("  Skipped (no database connection)");
            verification::IntegrationSummary {
                workflows_integrated: false,
                commands_integrated: false,
            }
        }
    } else {
        println!("⊘ Skipping integration verification");
        verification::IntegrationSummary { workflows_integrated: true, commands_integrated: true }
    };

    VerificationResult { migrations, paths, config, dependencies, build, health, integration }
}

fn print_header(name: &str) {
    println!("┌──────────────────────────────────────────────────────────────┐");
    println!("│ Verifying: {:<50} │", name);
    println!("└──────────────────────────────────────────────────────────────┘");
}
