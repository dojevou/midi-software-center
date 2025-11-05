#!/usr/bin/env python3
"""
Enhanced Grok 4 Project Reviewer for MIDI Software Center
Targets: Rust backends, Tauri apps, Svelte frontends, PostgreSQL database
"""

import os
import sys
import json
import time
import subprocess
from pathlib import Path
import httpx
from datetime import datetime


class MIDIProjectReviewer:
    """Enhanced code reviewer specifically for MIDI Software Center"""

    def __init__(self, api_key=None):
        self.api_key = api_key or os.getenv("GROK_API_KEY")
        self.project_root = Path("/home/dojevou/projects/midi-software-center")
        self.analysis_history = []

        self.client = httpx.Client(timeout=120.0, http2=True)

        if not self.api_key:
            print("❌ GROK_API_KEY not set")
            print("   export GROK_API_KEY='xai-...'")
            sys.exit(1)

    # 🚀 IMPROVEMENT 1: TARGETED MIDI PROJECT ANALYSIS
    def gather_project_context(self):
        """Collect specific data for MIDI software project"""
        print("🎵 Gathering MIDI Software Center context...")

        context = {
            "timestamp": datetime.now().isoformat(),
            "project_phase": self.detect_project_phase(),
            "architecture": self.analyze_midi_architecture(),
            "build_errors": self.collect_midi_build_errors(),
            "test_status": self.analyze_test_health(),
            "database_state": self.analyze_database(),
            "key_components": self.extract_critical_components(),
            "recent_changes": self.get_recent_changes(),
        }
        return context

    def detect_project_phase(self):
        """Detect current project phase from documentation"""
        phase_files = [
            "PHASE-9-EXECUTION-PLAN.md",
            "PHASE-9-100-PERCENT-QUALITY-PLAN.md",
            "PHASE-9-FINAL-STATUS.md",
        ]

        phase_info = {}
        for phase_file in phase_files:
            full_path = self.project_root / phase_file
            if full_path.exists():
                try:
                    content = full_path.read_text()[:1000]
                    phase_info[phase_file] = {"exists": True, "summary": content[:500]}
                except:
                    phase_info[phase_file] = {"exists": True, "error": "read_failed"}
            else:
                phase_info[phase_file] = {"exists": False}

        return phase_info

    def analyze_midi_architecture(self):
        """Analyze the specific MIDI software architecture"""
        architecture = {
            "backends": self.analyze_rust_backends(),
            "frontends": self.analyze_tauri_frontends(),
            "database": self.analyze_database_layer(),
            "shared_components": self.analyze_shared_components(),
            "build_system": self.analyze_build_system(),
        }
        return architecture

    def analyze_rust_backends(self):
        """Analyze the three Rust backends"""
        backends = {
            "midi-library-shared": self.analyze_backend("midi-library-shared"),
            "midi-pipeline": self.analyze_backend("midi-pipeline"),
            "midi-daw": self.analyze_backend("midi-daw"),
        }
        return backends

    def analyze_backend(self, backend_name):
        """Analyze a specific Rust backend"""
        backend_path = self.project_root / backend_name
        if not backend_path.exists():
            return {"exists": False}

        try:
            cargo_file = backend_path / "Cargo.toml"
            src_dir = backend_path / "src"

            rust_files = list(backend_path.glob("**/*.rs"))
            test_files = list(backend_path.glob("**/*test*.rs"))

            return {
                "exists": True,
                "rust_files": len(rust_files),
                "test_files": len(test_files),
                "has_cargo": cargo_file.exists(),
                "has_src": src_dir.exists(),
                "main_files": [f.name for f in rust_files[:5]],
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_tauri_frontends(self):
        """Analyze Tauri frontend applications"""
        frontends = {
            "daw": self.analyze_frontend("daw"),
            "pipeline": self.analyze_frontend("pipeline"),
        }
        return frontends

    def analyze_frontend(self, frontend_name):
        """Analyze a specific Tauri frontend"""
        frontend_path = self.project_root / frontend_name
        if not frontend_path.exists():
            return {"exists": False}

        try:
            tauri_dir = frontend_path / "src-tauri"
            src_dir = frontend_path / "src"

            svelte_files = list(frontend_path.glob("**/*.svelte"))
            typescript_files = list(frontend_path.glob("**/*.ts"))
            rust_files = list(tauri_dir.glob("**/*.rs")) if tauri_dir.exists() else []

            return {
                "exists": True,
                "svelte_files": len(svelte_files),
                "typescript_files": len(typescript_files),
                "rust_files": len(rust_files),
                "has_tauri": tauri_dir.exists(),
                "has_package_json": (frontend_path / "package.json").exists(),
                "components": [f.name for f in (src_dir / "Components").iterdir()]
                if (src_dir / "Components").exists()
                else [],
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_database_layer(self):
        """Analyze database and migrations"""
        db_path = self.project_root / "database"

        try:
            migrations = (
                list((db_path / "migrations").glob("*.sql"))
                if (db_path / "migrations").exists()
                else []
            )
            seeds = (
                list((db_path / "seeds").glob("*.sql"))
                if (db_path / "seeds").exists()
                else []
            )

            return {
                "exists": db_path.exists(),
                "migrations": len(migrations),
                "seeds": len(seeds),
                "migration_files": [f.name for f in migrations],
                "has_docker_compose": (db_path / "docker-compose.yml").exists(),
                "has_scripts": len(list(db_path.glob("scripts/*.sh"))) > 0,
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_shared_components(self):
        """Analyze shared libraries and components"""
        shared_path = self.project_root / "shared"

        try:
            rust_shared = shared_path / "rust"
            types_shared = shared_path / "types"
            ui_shared = shared_path / "ui"

            return {
                "exists": shared_path.exists(),
                "rust_shared": rust_shared.exists(),
                "types_shared": types_shared.exists(),
                "ui_shared": ui_shared.exists(),
                "rust_files": len(list(rust_shared.glob("**/*.rs")))
                if rust_shared.exists()
                else 0,
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_build_system(self):
        """Analyze build and deployment configuration"""
        build_info = {
            "makefile": (self.project_root / "Makefile").exists(),
            "cargo_workspace": (self.project_root / "Cargo.toml").exists(),
            "scripts": len(list((self.project_root / "scripts").glob("*.sh"))),
            "launch_scripts": len(
                list((self.project_root / "scripts" / "launch").glob("*.sh"))
            ),
            "docker_config": (self.project_root / "infrastructure" / "docker").exists(),
        }
        return build_info

    def collect_midi_build_errors(self):
        """Collect build errors across all components"""
        print("🔧 Running MIDI-specific builds...")

        errors = {
            "rust_workspace": self.build_rust_workspace(),
            "frontend_daw": self.build_frontend("daw"),
            "frontend_pipeline": self.build_frontend("pipeline"),
            "type_check": self.check_typescript(),
            "test_compilation": self.check_test_compilation(),
        }

        return errors

    def build_rust_workspace(self):
        """Build the entire Rust workspace"""
        try:
            result = subprocess.run(
                ["cargo", "build", "--workspace", "--message-format=json"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=180,
            )

            errors = []
            if result.stderr:
                for line in result.stderr.splitlines():
                    if "error[" in line or "ERROR" in line.upper():
                        errors.append(line.strip())

            return {
                "success": result.returncode == 0,
                "error_count": len(errors),
                "sample_errors": errors[:10],
                "raw_output": result.stderr[:4000] if result.stderr else "No errors",
            }
        except Exception as e:
            return {"error": f"Rust workspace build failed: {str(e)}"}

    def build_frontend(self, frontend_name):
        """Build a specific frontend"""
        frontend_path = self.project_root / frontend_name

        try:
            # Try TypeScript compilation first
            ts_result = subprocess.run(
                ["npx", "tsc", "--noEmit"],
                cwd=frontend_path,
                capture_output=True,
                text=True,
                timeout=60,
            )

            # Try Svelte check
            svelte_result = subprocess.run(
                ["npx", "svelte-check"],
                cwd=frontend_path,
                capture_output=True,
                text=True,
                timeout=60,
            )

            return {
                "typescript_ok": ts_result.returncode == 0,
                "svelte_ok": svelte_result.returncode == 0,
                "ts_errors": ts_result.stderr[:2000] if ts_result.stderr else None,
                "svelte_errors": svelte_result.stdout[:2000]
                if svelte_result.stdout
                else None,
            }
        except Exception as e:
            return {"error": f"Frontend {frontend_name} check failed: {str(e)}"}

    def check_typescript(self):
        """Check TypeScript across project"""
        try:
            result = subprocess.run(
                [
                    "find",
                    ".",
                    "-name",
                    "*.ts",
                    "-not",
                    "-path",
                    "*/node_modules/*",
                    "-exec",
                    "npx",
                    "tsc",
                    "--noEmit",
                    "{}",
                    ";",
                ],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=30,
            )
            return {
                "output": result.stderr[:1000]
                if result.stderr
                else "No TypeScript errors"
            }
        except:
            return {"error": "TypeScript check skipped"}

    def check_test_compilation(self):
        """Check test compilation status"""
        try:
            # Check disabled tests
            disabled_tests = list((self.project_root / "_disabled_tests").glob("*.rs"))

            # Try compiling tests
            result = subprocess.run(
                ["cargo", "test", "--no-run", "--workspace"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=120,
            )

            return {
                "disabled_tests_count": len(disabled_tests),
                "test_compilation_ok": result.returncode == 0,
                "compilation_errors": result.stderr[:3000] if result.stderr else None,
            }
        except Exception as e:
            return {"error": f"Test compilation check failed: {str(e)}"}

    def analyze_test_health(self):
        """Analyze test suite health"""
        try:
            # Count test files
            rust_tests = list(self.project_root.glob("**/*test*.rs"))
            frontend_tests = list(self.project_root.glob("**/*test*.ts"))

            # Check test directories
            test_dirs = {
                "e2e": (self.project_root / "tests" / "e2e").exists(),
                "integration": (self.project_root / "tests" / "integration").exists(),
                "fixtures": (self.project_root / "tests" / "fixtures").exists(),
            }

            return {
                "rust_tests": len(rust_tests),
                "frontend_tests": len(frontend_tests),
                "test_directories": test_dirs,
                "coverage_report": (
                    self.project_root / "coverage_error" / "tarpaulin-report.html"
                ).exists(),
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_database(self):
        """Analyze database state"""
        db_path = self.project_root / "database"

        try:
            # Check if database is running
            docker_result = subprocess.run(
                ["docker", "compose", "ps", "--services", "--filter", "status=running"],
                cwd=db_path,
                capture_output=True,
                text=True,
            )

            return {
                "docker_services_running": docker_result.stdout.strip().split("\n")
                if docker_result.returncode == 0
                else [],
                "migrations_applied": len(list((db_path / "migrations").glob("*.sql"))),
                "has_sample_data": (db_path / "seeds").exists(),
            }
        except Exception as e:
            return {"error": f"Database analysis failed: {str(e)}"}

    def extract_critical_components(self):
        """Extract critical project components"""
        critical_files = {}

        # Rust backend critical files
        rust_critical = [
            "Cargo.toml",
            "midi-library-shared/Cargo.toml",
            "midi-pipeline/Cargo.toml",
            "midi-daw/Cargo.toml",
            "shared/rust/Cargo.toml",
        ]

        # Frontend critical files
        frontend_critical = [
            "daw/package.json",
            "pipeline/package.json",
            "daw/src-tauri/tauri.conf.json",
            "pipeline/src-tauri/tauri.conf.json",
        ]

        # Config critical files
        config_critical = ["database/docker-compose.yml", "rustfmt.toml", "Makefile"]

        all_critical = rust_critical + frontend_critical + config_critical

        for file_path in all_critical:
            full_path = self.project_root / file_path
            if full_path.exists():
                try:
                    content = full_path.read_text()[:2000]
                    critical_files[file_path] = content
                except Exception as e:
                    critical_files[file_path] = f"Error reading: {str(e)}"

        return critical_files

    def get_recent_changes(self):
        """Get recent git changes for context"""
        try:
            # Get recent commits
            commits = subprocess.run(
                ["git", "log", "--oneline", "-10", "--graph"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
            )

            # Get current branch
            branch = subprocess.run(
                ["git", "branch", "--show-current"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
            )

            return {
                "recent_commits": commits.stdout
                if commits.returncode == 0
                else "No git history",
                "current_branch": branch.stdout.strip()
                if branch.returncode == 0
                else "Unknown",
            }
        except Exception as e:
            return {"error": f"Git check failed: {str(e)}"}

    # 🔄 IMPROVEMENT 3: ITERATIVE MIDI-SPECIFIC PIPELINE
    def run_midi_analysis_pipeline(self, context):
        """MIDI-specific analysis pipeline"""
        print("\n🔄 Starting MIDI-specific analysis pipeline...")

        stages = [
            ("🎵 MIDI Architecture Review", self.midi_architecture_review),
            ("🔧 Build System Analysis", self.build_system_analysis),
            ("🧪 Test Suite Health Check", self.test_health_analysis),
            ("🚀 Deployment Readiness", self.deployment_readiness),
        ]

        results = {}
        for stage_name, stage_func in stages:
            print(f"\n{stage_name}")
            print("=" * len(stage_name))
            results[stage_name] = stage_func(context)
            self.save_analysis_stage(stage_name, results[stage_name])
            time.sleep(1)

        return results

    def midi_architecture_review(self, context):
        """Stage 1: MIDI-specific architecture review"""
        prompt = f"""
MIDI SOFTWARE CENTER - ARCHITECTURE REVIEW

PROJECT STRUCTURE:
- Backends: {context["architecture"]["backends"]}
- Frontends: {context["architecture"]["frontends"]}
- Database: {context["architecture"]["database"]}
- Shared: {context["architecture"]["shared_components"]}

CURRENT PHASE: {context["project_phase"]}

ANALYZE:
1. Architecture cohesion between 3 backends and 2 frontends
2. MIDI data flow through the system
3. Shared code usage and dependencies
4. Database schema alignment with MIDI domain
5. Tauri desktop app architecture

Focus on MIDI-specific architectural patterns and potential integration issues.
"""
        return self.call_grok(prompt)

    def build_system_analysis(self, context):
        """Stage 2: Build system and compilation analysis"""
        prompt = f"""
MIDI SOFTWARE CENTER - BUILD SYSTEM ANALYSIS

BUILD STATUS:
- Rust Workspace: {context["build_errors"]["rust_workspace"].get("success", False)}
- DAW Frontend: {context["build_errors"]["frontend_daw"]}
- Pipeline Frontend: {context["build_errors"]["frontend_pipeline"]}
- Test Compilation: {context["build_errors"]["test_compilation"]}

SPECIFIC ERRORS:
{context["build_errors"]["rust_workspace"].get("sample_errors", [])}

ANALYZE:
1. Root causes of compilation failures
2. Dependency conflicts between backends
3. Frontend-backend integration issues
4. Test compilation blockers
5. Build system optimization opportunities

Provide specific fix recommendations for MIDI project structure.
"""
        return self.call_grok(prompt)

    def test_health_analysis(self, context):
        """Stage 3: Test suite health analysis"""
        prompt = f"""
MIDI SOFTWARE CENTER - TEST SUITE HEALTH

TEST STATUS:
- Rust Tests: {context["test_status"].get("rust_tests", 0)}
- Frontend Tests: {context["test_status"].get("frontend_tests", 0)}
- Disabled Tests: {context["build_errors"]["test_compilation"].get("disabled_tests_count", 0)}
- Test Directories: {context["test_status"].get("test_directories", {})}

TEST COMPILATION:
{context["build_errors"]["test_compilation"]}

ANALYZE:
1. Test coverage gaps for MIDI functionality
2. Integration test strategy
3. Performance testing needs
4. MIDI file parsing tests
5. Database transaction tests

Recommend test strategy for music production software.
"""
        return self.call_grok(prompt)

    def deployment_readiness(self, context):
        """Stage 4: Deployment readiness assessment"""
        prompt = f"""
MIDI SOFTWARE CENTER - DEPLOYMENT READINESS

CURRENT STATE:
- Project Phase: {context["project_phase"]}
- Build Status: {context["build_errors"]["rust_workspace"].get("success", False)}
- Database: {context["database_state"]}
- Recent Changes: {context["recent_changes"]}

ASSESS READINESS FOR:
1. Development deployment
2. Testing environment
3. Production release

PROVIDE:
- Critical blockers to address
- Recommended deployment strategy
- Performance considerations for MIDI apps
- Desktop app distribution plan

Focus on music production software deployment specifics.
"""
        return self.call_grok(prompt)

    # 🔍 IMPROVEMENT 4: ERROR PATTERN DETECTION
    def analyze_error_patterns(self, context):
        """Advanced error pattern detection for MIDI project"""
        prompt = f"""
MIDI SOFTWARE CENTER - ERROR PATTERN DETECTION

COMPILATION ERRORS:
{context["build_errors"]["rust_workspace"].get("raw_output", "No errors")}

PROJECT CONTEXT:
- 3 Rust backends, 2 Tauri frontends
- MIDI processing, database, real-time audio
- Complex type system with shared libraries

CATEGORIZE ERRORS BY:
1. Type system issues (traits, generics, lifetimes)
2. Module and visibility problems
3. Database and async conflicts
4. MIDI-specific domain errors
5. Frontend-backend type mismatches
6. Build dependency conflicts

For each category:
- Identify root cause patterns
- Provide specific code fixes
- Suggest architectural improvements

Focus on patterns that affect music software reliability.
"""
        return self.call_grok(prompt)

    # 📋 IMPROVEMENT 9: PRIORITIZED MIDI ACTION PLAN
    def generate_midi_action_plan(self, context, pipeline_results, error_patterns):
        """Generate MIDI-specific prioritized action plan"""
        prompt = f"""
MIDI SOFTWARE CENTER - PRIORITIZED ACTION PLAN

CURRENT STATUS:
- Phase: {context["project_phase"]}
- Architecture: {context["architecture"]}
- Build: {"✅" if context["build_errors"]["rust_workspace"].get("success") else "❌"}
- Tests: {context["test_status"]}

ANALYSIS INSIGHTS:
{pipeline_results}

ERROR PATTERNS:
{error_patterns}

CREATE TIME-BOXED ACTION PLAN:

🎯 IMMEDIATE (Next 8 hours):
[Critical fixes for MIDI functionality]

🚀 SHORT TERM (This week):
[Architecture improvements, test enablement]

🏗️ MEDIUM TERM (Next 2 weeks):
[Performance optimization, feature completion]

🎊 LONG TERM (Release ready):
[Polish, documentation, deployment]

INCLUDE:
- Specific file paths to fix
- Exact commands to run
- Success metrics for music software
- Risk mitigation for real-time audio

Prioritize MIDI core functionality and audio reliability.
"""
        return self.call_grok(prompt)

    def save_analysis_stage(self, stage_name, result):
        """Save analysis stage results"""
        self.analysis_history.append(
            {
                "stage": stage_name,
                "timestamp": datetime.now().isoformat(),
                "result": result[:1000] + "..." if len(result) > 1000 else result,
            }
        )

    def call_grok(self, prompt: str, model: str = "grok-4-fast-reasoning") -> str:
        """Call Grok API"""
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            "max_tokens": 4000,
        }

        try:
            response = self.client.post(
                "https://api.x.ai/v1/chat/completions", json=payload, headers=headers
            )
            response.raise_for_status()
            result = response.json()
            return result["choices"][0]["message"]["content"]
        except Exception as e:
            return f"❌ API Error: {str(e)}"

    def run_enhanced_midi_analysis(self):
        """Main enhanced analysis for MIDI project"""
        print("🎵 ENHANCED MIDI SOFTWARE CENTER ANALYSIS")
        print("=" * 50)

        # 1. Gather project context
        context = self.gather_project_context()

        # 2. Run MIDI-specific pipeline
        pipeline_results = self.run_midi_analysis_pipeline(context)

        # 3. Advanced error pattern detection
        print("\n🔍 Analyzing error patterns...")
        error_patterns = self.analyze_error_patterns(context)

        # 4. Generate prioritized action plan
        print("\n📋 Generating MIDI-specific action plan...")
        action_plan = self.generate_midi_action_plan(
            context, pipeline_results, error_patterns
        )

        # 5. Display comprehensive results
        self.display_midi_results(pipeline_results, error_patterns, action_plan)

        # 6. Save analysis
        self.save_analysis_history()

        print(
            f"\n✅ MIDI analysis complete! {len(self.analysis_history)} stages executed."
        )

    def display_midi_results(self, pipeline_results, error_patterns, action_plan):
        """Display comprehensive MIDI analysis results"""
        print("\n" + "=" * 60)
        print("🎵 MIDI SOFTWARE CENTER - COMPREHENSIVE ANALYSIS")
        print("=" * 60)

        print("\n📊 PIPELINE RESULTS:")
        for stage_name, result in pipeline_results.items():
            print(f"\n{stage_name}")
            print("-" * len(stage_name))
            print(result)

        print("\n🔍 ERROR PATTERNS:")
        print("=" * 20)
        print(error_patterns)

        print("\n📋 ACTION PLAN:")
        print("=" * 15)
        print(action_plan)

    def save_analysis_history(self):
        """Save analysis history"""
        history_file = self.project_root / "grok_midi_analysis.json"
        try:
            history_data = {
                "timestamp": datetime.now().isoformat(),
                "analysis_stages": self.analysis_history,
            }
            history_file.write_text(json.dumps(history_data, indent=2))
            print(f"💾 Analysis saved to: {history_file}")
        except Exception as e:
            print(f"⚠️ Could not save history: {e}")

    def close(self):
        """Cleanup"""
        self.client.close()


def main():
    """Main entry point"""
    reviewer = MIDIProjectReviewer()

    try:
        if len(sys.argv) > 1:
            if sys.argv[1] in ["midi", "full", "enhanced"]:
                reviewer.run_enhanced_midi_analysis()
            elif sys.argv[1] == "quick":
                context = reviewer.gather_project_context()
                result = reviewer.midi_architecture_review(context)
                print(result)
            elif sys.argv[1] == "errors":
                context = reviewer.gather_project_context()
                result = reviewer.analyze_error_patterns(context)
                print(result)
            else:
                print("Usage: python3 midi_grok_reviewer.py [command]")
                print("\nCommands:")
                print("  midi     - Full MIDI-specific analysis (recommended)")
                print("  quick    - Quick architecture review")
                print("  errors   - Error pattern analysis only")
        else:
            reviewer.run_enhanced_midi_analysis()
    finally:
        reviewer.close()


if __name__ == "__main__":
    main()
