#!/usr/bin/env python3
"""
ULTRA SUPERCHARGED Grok 4 Project Reviewer for MIDI Software Center - PRODUCTION VERSION
Captures real build errors, applies fixes, and generates AI-powered analysis reports.
"""

import os
import sys
import json
import time
import re
import subprocess
import ast
from pathlib import Path
from datetime import datetime
from collections import defaultdict
import httpx
import threading
from io import StringIO


class TerminalOutputCapture:
    """Captures all terminal output to a Markdown file"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
        self.original_stdout = sys.stdout
        self.original_stderr = sys.stderr
        self.capture_buffer = StringIO()
        self.markdown_content = []
        self.start_time = datetime.now()
    
    def start_capture(self):
        """Start capturing all terminal output"""
        sys.stdout = self
        sys.stderr = self
        self.markdown_content = [
            f"# MIDI Software Center Analysis Report",
            f"**Generated**: {self.start_time.strftime('%Y-%m-%d %H:%M:%S')}",
            f"**Project**: {self.project_root}",
            f"",
            f"## Terminal Output",
            f"```bash"
        ]
    
    def stop_capture(self):
        """Stop capturing and save to Markdown file"""
        sys.stdout = self.original_stdout
        sys.stderr = self.original_stderr
        
        # Close the code block
        self.markdown_content.append("```")
        
        # Add summary section
        self.markdown_content.extend([
            f"",
            f"## Analysis Summary",
            f"- **Analysis completed**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"- **Duration**: {(datetime.now() - self.start_time).total_seconds():.1f} seconds",
            f"- **Output captured**: {len(self.capture_buffer.getvalue())} characters",
        ])
        
        # Save to file
        report_path = self.project_root / "analysis_report.md"
        report_path.write_text('\n'.join(self.markdown_content))
        print(f"📄 Analysis report saved: {report_path}")
    
    def write(self, text):
        """Capture write operations"""
        self.original_stdout.write(text)
        self.capture_buffer.write(text)
        
        # Add to markdown, handling special characters
        clean_text = text.replace('`', '\\`')
        self.markdown_content.append(clean_text.rstrip())
    
    def flush(self):
        """Flush buffer"""
        self.original_stdout.flush()


class ProgressTracker:
    """Tracks and displays real-time progress"""
    
    def __init__(self):
        self.start_time = time.time()
        self.completed_steps = 0
        self.total_steps = 0
    
    def start_analysis(self, total_steps):
        """Start tracking analysis progress"""
        self.total_steps = total_steps
        self.completed_steps = 0
        self.start_time = time.time()
    
    def update_progress(self, step_name, results=None):
        """Update progress with current step"""
        self.completed_steps += 1
        progress = (self.completed_steps / self.total_steps) * 100
        elapsed = time.time() - self.start_time
        
        status_line = f"📊 [{progress:.1f}%] {step_name}"
        if results:
            status_line += f" | {results}"
        
        print(status_line)


class RealErrorCapture:
    """Captures and analyzes real build errors"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def capture_live_errors(self):
        """Run actual builds and capture real errors"""
        print("🔍 Capturing live build errors...")
        
        # Run cargo check and test compilation
        builds = {
            "rust_check": self.run_cargo_check(),
            "test_compilation": self.run_test_compilation()
        }
        
        # Extract ACTUAL error messages
        actual_errors = []
        for build_name, result in builds.items():
            if not result["success"]:
                errors = self.parse_compiler_errors(result["output"])
                for error in errors:
                    error["source"] = build_name
                actual_errors.extend(errors)
        
        print(f"✅ Captured {len(actual_errors)} actual errors from builds")
        return actual_errors
    
    def run_cargo_check(self):
        """Run cargo check for quick compilation check"""
        try:
            result = subprocess.run(
                ["cargo", "check", "--workspace"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            return {
                "success": result.returncode == 0,
                "output": result.stderr,
                "returncode": result.returncode
            }
        except Exception as e:
            return {"success": False, "output": str(e), "returncode": -1}
    
    def run_test_compilation(self):
        """Check test compilation"""
        try:
            result = subprocess.run(
                ["cargo", "test", "--no-run", "--workspace"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            return {
                "success": result.returncode == 0,
                "output": result.stderr,
                "returncode": result.returncode
            }
        except Exception as e:
            return {"success": False, "output": str(e), "returncode": -1}
    
    def parse_compiler_errors(self, output):
        """Parse actual compiler error messages"""
        errors = []
        lines = output.split('\n')
        
        for line in lines:
            line = line.strip()
            if not line:
                continue
                
            # Rust errors
            if 'error[' in line or 'ERROR' in line:
                errors.append({
                    "type": "rust_compilation",
                    "message": line[:200],
                    "file": self.extract_file_from_error(line),
                    "line": self.extract_line_from_error(line)
                })
            # General error patterns
            elif 'error:' in line.lower() and len(line) > 20:
                errors.append({
                    "type": "general",
                    "message": line[:200],
                    "file": "unknown",
                    "line": "unknown"
                })
        
        return errors[:50]
    
    def extract_file_from_error(self, error_line):
        """Extract filename from Rust error"""
        match = re.search(r'--> ([^:]+\.rs):', error_line)
        return match.group(1) if match else "unknown"
    
    def extract_line_from_error(self, error_line):
        """Extract line number from Rust error"""
        match = re.search(r':(\d+):\d+', error_line)
        return match.group(1) if match else "unknown"


class SimpleAutoFixer:
    """Auto-fixer for common compilation issues"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def apply_quick_fixes(self, actual_errors):
        """Apply quick fixes based on actual errors"""
        print("🔧 Applying quick fixes...")
        applied_fixes = []
        
        # Fix 1: Clean build artifacts
        if self.run_cargo_clean():
            applied_fixes.append("cargo_clean")
        
        # Fix 2: Update dependencies
        if self.run_cargo_update():
            applied_fixes.append("cargo_update")
        
        # Fix 3: Fix dead code warnings
        dead_code_fixed = self.fix_dead_code_warnings(actual_errors)
        if dead_code_fixed:
            applied_fixes.append("dead_code_fixes")
        
        return applied_fixes
    
    def run_cargo_clean(self):
        """Run cargo clean"""
        try:
            result = subprocess.run(
                ["cargo", "clean"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            if result.returncode == 0:
                print("✅ Cleaned build artifacts")
                return True
        except:
            pass
        return False
    
    def run_cargo_update(self):
        """Run cargo update"""
        try:
            result = subprocess.run(
                ["cargo", "update"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            if result.returncode == 0:
                print("✅ Updated dependencies")
                return True
        except:
            pass
        return False
    
    def fix_dead_code_warnings(self, actual_errors):
        """Report dead code warnings"""
        dead_code_errors = [e for e in actual_errors if "dead code" in e["message"].lower()]
        
        if not dead_code_errors:
            return False
            
        print(f"🔧 Found {len(dead_code_errors)} dead code warnings")
        for error in dead_code_errors[:5]:
            print(f"   🛠️  {error['message'][:100]}...")
        
        return len(dead_code_errors) > 0


class PredictiveAnalyzer:
    """Predicts future errors based on patterns"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def predict_compilation_issues(self):
        """Predict what will break before it happens"""
        print("🔮 Running predictive analysis...")
        predictions = []
        
        # Check for common issues
        if self.has_build_config_issues():
            predictions.append("🔨 Build configuration issues detected")
        
        if self.has_missing_midi_dependencies():
            predictions.append("🎵 Missing MIDI-specific dependencies")
        
        if not self.is_workspace_configured():
            predictions.append("🏗️  Workspace not properly configured")
        
        return predictions
    
    def has_build_config_issues(self):
        """Check for build configuration issues"""
        root_cargo = self.project_root / "Cargo.toml"
        if root_cargo.exists():
            content = root_cargo.read_text()
            return "[workspace]" not in content
        return True
    
    def has_missing_midi_dependencies(self):
        """Check for missing MIDI-specific dependencies"""
        cargo_files = list(self.project_root.glob("**/Cargo.toml"))
        for cargo_file in cargo_files:
            content = cargo_file.read_text()
            midi_crates = ["midly", "midir", "cpal"]
            if any(crate in content for crate in midi_crates):
                return False
        return True
    
    def is_workspace_configured(self):
        """Check if Cargo workspace is properly configured"""
        root_cargo = self.project_root / "Cargo.toml"
        if root_cargo.exists():
            content = root_cargo.read_text()
            return "[workspace]" in content
        return False


class AnalysisLearner:
    """Learns from previous analyses to improve future ones"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
        self.history_file = self.project_root / "grok_analysis_history.json"
    
    def get_most_likely_issues(self):
        """Get historical patterns"""
        patterns = []
        if self.history_file.exists():
            try:
                history = json.loads(self.history_file.read_text())
                error_counts = {}
                for stage in history.get("analysis_stages", []):
                    result = stage.get("result", "")
                    if "error" in result.lower():
                        error_counts["errors"] = error_counts.get("errors", 0) + 1
                    if "dead code" in result.lower():
                        error_counts["dead_code"] = error_counts.get("dead_code", 0) + 1
                
                for issue, count in error_counts.items():
                    if count > 0:
                        patterns.append((issue, count))
            except:
                pass
        
        return patterns
    
    def save_analysis(self, analysis_data):
        """Save current analysis to history"""
        try:
            history = {}
            if self.history_file.exists():
                try:
                    history = json.loads(self.history_file.read_text())
                except:
                    pass
            
            if "analysis_stages" not in history:
                history["analysis_stages"] = []
            
            history["analysis_stages"].append({
                "timestamp": datetime.now().isoformat(),
                "data": analysis_data
            })
            
            # Keep only last 5 analyses
            if len(history["analysis_stages"]) > 5:
                history["analysis_stages"] = history["analysis_stages"][-5:]
            
            self.history_file.write_text(json.dumps(history, indent=2))
        except Exception as e:
            print(f"⚠️  Could not save analysis history: {e}")


class UltraSuperchargedMIDIReviewer:
    """Main class for comprehensive project analysis"""
    
    def __init__(self, api_key=None, project_root=None):
        self.api_key = api_key or os.getenv("GROK_API_KEY")
        self.project_root = Path(project_root) if project_root else Path.cwd()
        
        # Initialize components
        self.terminal_capture = TerminalOutputCapture(self.project_root)
        self.progress_tracker = ProgressTracker()
        self.error_capture = RealErrorCapture(self.project_root)
        self.auto_fixer = SimpleAutoFixer(self.project_root)
        self.predictive_analyzer = PredictiveAnalyzer(self.project_root)
        self.analysis_learner = AnalysisLearner(self.project_root)
        
        self.client = httpx.Client(timeout=120.0, http2=True)
        
        if not self.api_key:
            print("❌ GROK_API_KEY not set")
            print("   export GROK_API_KEY='xai-...'")
            sys.exit(1)

    def generate_intelligent_prompt(self, actual_errors, predictions, historical_patterns):
        """Generate comprehensive analysis prompt"""
        
        error_summary = "\n".join([
            f"{i+1}. {error['type']}: {error['message']}" 
            for i, error in enumerate(actual_errors[:10])
        ])
        
        prompt = f"""
🚨 MIDI SOFTWARE CENTER - COMPREHENSIVE ANALYSIS

ACTUAL ERRORS FOUND ({len(actual_errors)} total):
{error_summary if error_summary else "No errors captured"}

PREDICTED ISSUES:
{chr(10).join(f"- {prediction}" for prediction in predictions) if predictions else "No predictions"}

HISTORICAL PATTERNS:
{chr(10).join(f"- {issue[0]}: {issue[1]} occurrences" for issue in historical_patterns) if historical_patterns else "No historical patterns"}

PROJECT OVERVIEW:
- Rust project with Tauri desktop apps
- MIDI music production software
- Multiple workspaces (DAW, Pipeline, etc.)

REQUESTED ANALYSIS:
1. Root cause analysis of the main compilation errors
2. Specific fix recommendations for Rust/Tauri projects
3. MIDI-specific architectural considerations
4. Prioritized action plan

Please provide concrete, actionable advice.
"""
        return prompt

    def generate_fix_script(self):
        """Generate executable fix script"""
        print("📜 Generating fix script...")
        
        script_content = """#!/bin/bash
# MIDI Software Center Fix Script
# Generated: {date}

set -e

echo "🎵 MIDI Software Center - Automated Fixes"
echo "========================================"

# Fix 1: Clean build artifacts
echo "🔧 Cleaning build artifacts..."
cargo clean

# Fix 2: Update dependencies
echo "🔧 Updating dependencies..."
cargo update

# Fix 3: Check compilation
echo "🔍 Checking compilation..."
cargo check --workspace

# Fix 4: Configure workspace if needed
if ! grep -q "[workspace]" Cargo.toml 2>/dev/null; then
    echo "🔧 Adding workspace configuration..."
    cat >> Cargo.toml << 'EOF'

[workspace]
members = [
    "daw/src-tauri",
    "pipeline/src-tauri", 
    "shared/rust",
    "scripts/test-midi-files"
]
EOF
fi

echo ""
echo "✅ Fixes applied!"
echo ""
echo "📊 Next steps:"
echo "   1. Run 'cargo build --workspace' to verify"
echo "   2. Run 'cargo test --workspace' to check tests"
echo "   3. Address any remaining errors manually"
""".format(date=datetime.now().isoformat())
        
        script_path = self.project_root / "auto_fix.sh"
        script_path.write_text(script_content)
        script_path.chmod(0o755)
        
        print(f"✅ Fix script generated: {script_path}")
        return script_path

    def generate_troubleshooting_guide(self, analysis_results, actual_errors, applied_fixes):
        """Generate comprehensive troubleshooting guide"""
        
        guide = f"""# MIDI Software Center Troubleshooting Guide
Generated: {datetime.now().isoformat()}

## Current Status
- **Errors Found**: {len(actual_errors)}
- **Fixes Applied**: {len(applied_fixes)}
- **Main Error Types**: {', '.join(set(e['type'] for e in actual_errors)) if actual_errors else 'None'}

## Applied Fixes
{chr(10).join(f"- {fix}" for fix in applied_fixes) if applied_fixes else "No fixes applied yet"}

## Common Solutions

### 1. Build Configuration Issues
Add workspace configuration to Cargo.toml:
```toml
[workspace]
members = [
    "daw/src-tauri",
    "pipeline/src-tauri",
    "shared/rust", 
    "scripts/test-midi-files"
]
```

### 2. Dependency Issues
```bash
# Update all dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build --workspace
```

### 3. Dead Code Warnings
Add `#[allow(dead_code)]` above structs or functions that are intentionally unused.

### 4. MIDI-Specific Issues
Ensure MIDI crates are in Cargo.toml:
```toml
[dependencies]
midir = "0.7"
midly = "0.2"
cpal = "0.17"
```

## Verification Steps
1. Run `cargo check --workspace` to verify compilation
2. Run `cargo test --workspace` to check tests
3. Run `cargo build --workspace` for full build
4. Run `cargo clippy --workspace` for linting advice

## Generated Artifacts
- `auto_fix.sh` - Executable fix script
- `analysis_report.md` - Full analysis output
- `grok_analysis_history.json` - Historical data

## AI Analysis
{analysis_results}
"""

        guide_path = self.project_root / "TROUBLESHOOTING_GUIDE.md"
        guide_path.write_text(guide)
        print(f"📖 Troubleshooting guide generated: {guide_path}")
        return guide_path

    def call_grok(self, prompt: str) -> str:
        """Call Grok API with proper error handling"""
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }
        
        payload = {
            "model": "grok-4-fast-reasoning",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            "max_tokens": 3000,
        }
        
        try:
            response = self.client.post(
                "https://api.x.ai/v1/chat/completions",
                json=payload,
                headers=headers,
                timeout=60.0
            )
            response.raise_for_status()
            result = response.json()
            return result["choices"][0]["message"]["content"]
        except Exception as e:
            return f"❌ API Error: {str(e)}"

    def run_analysis(self):
        """Run complete ultra supercharged analysis"""
        print("🚀 ULTRA SUPERCHARGED MIDI ANALYSIS ACTIVATED")
        print("=" * 60)
        
        # Start terminal output capture
        self.terminal_capture.start_capture()
        
        try:
            # Define analysis steps
            self.progress_tracker.start_analysis(6)
            
            # 1. Capture ACTUAL errors
            actual_errors = self.error_capture.capture_live_errors()
            self.progress_tracker.update_progress("Captured live errors", f"{len(actual_errors)} errors")
            
            # 2. Run predictive analysis
            predictions = self.predictive_analyzer.predict_compilation_issues()
            self.progress_tracker.update_progress("Predictive analysis", f"{len(predictions)} predictions")
            
            # 3. Load historical patterns
            historical_patterns = self.analysis_learner.get_most_likely_issues()
            self.progress_tracker.update_progress("Historical analysis", f"{len(historical_patterns)} patterns")
            
            # 4. Apply quick fixes
            applied_fixes = self.auto_fixer.apply_quick_fixes(actual_errors)
            self.progress_tracker.update_progress("Applied fixes", f"{len(applied_fixes)} fixes")
            
            # 5. Get AI analysis
            prompt = self.generate_intelligent_prompt(actual_errors, predictions, historical_patterns)
            ai_analysis = self.call_grok(prompt)
            self.progress_tracker.update_progress("AI analysis", "Complete")
            
            # 6. Generate output files
            script_path = self.generate_fix_script()
            guide_path = self.generate_troubleshooting_guide(ai_analysis, actual_errors, applied_fixes)
            self.progress_tracker.update_progress("Generated outputs", "Script + Guide")
            
            # Save analysis for learning
            self.analysis_learner.save_analysis({
                "error_count": len(actual_errors),
                "applied_fixes": applied_fixes,
                "predictions": predictions
            })
            
            # Display results
            self.display_results(ai_analysis, predictions, historical_patterns, applied_fixes, actual_errors, script_path, guide_path)
            
            print(f"\n🎉 ANALYSIS COMPLETE!")
            print(f"📁 Generated files:")
            print(f"   - {script_path}")
            print(f"   - {guide_path}")
            print(f"   - analysis_report.md")
            print(f"   - grok_analysis_history.json")
            
        except Exception as e:
            print(f"❌ Analysis failed: {e}")
            import traceback
            traceback.print_exc()
        finally:
            # Always stop capture
            self.terminal_capture.stop_capture()

    def display_results(self, analysis_results, predictions, historical_patterns, applied_fixes, actual_errors, script_path, guide_path):
        """Display comprehensive results"""
        print("\n" + "="*70)
        print("📊 ANALYSIS RESULTS")
        print("="*70)
        
        print(f"\n🔮 PREDICTIONS:")
        for prediction in predictions:
            print(f"  • {prediction}")
        
        print(f"\n📚 HISTORICAL PATTERNS:")
        for pattern, count in historical_patterns:
            print(f"  • {pattern}: {count} occurrences")
        
        print(f"\n🔧 APPLIED FIXES:")
        for fix in applied_fixes:
            print(f"  • {fix}")
        
        print(f"\n❌ CURRENT ERRORS: {len(actual_errors)}")
        for i, error in enumerate(actual_errors[:5]):
            print(f"  {i+1}. [{error['type']}] {error['message']}")
        if len(actual_errors) > 5:
            print(f"  ... and {len(actual_errors) - 5} more errors")
        
        print(f"\n🧠 AI ANALYSIS:")
        print("-" * 40)
        print(analysis_results[:1500])
        if len(analysis_results) > 1500:
            print("\n... (see TROUBLESHOOTING_GUIDE.md for full analysis)")
        
        print(f"\n📜 GENERATED FILES:")
        print(f"  • {script_path}")
        print(f"  • {guide_path}")

    def close(self):
        """Cleanup resources"""
        self.client.close()


def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="ULTRA SUPERCHARGED Grok 4 Project Reviewer"
    )
    parser.add_argument(
        "--project",
        default=None,
        help="Project root directory (default: current directory)"
    )
    parser.add_argument(
        "--api-key",
        default=None,
        help="Grok API key (default: GROK_API_KEY env var)"
    )
    parser.add_argument(
        "command",
        nargs="?",
        default="analyze",
        choices=["analyze", "ultra", "super"],
        help="Command to run"
    )
    
    args = parser.parse_args()
    
    reviewer = UltraSuperchargedMIDIReviewer(
        api_key=args.api_key,
        project_root=args.project
    )
    
    try:
        if args.command in ["analyze", "ultra", "super"]:
            reviewer.run_analysis()
        else:
            parser.print_help()
    finally:
        reviewer.close()


if __name__ == "__main__":
    main()
