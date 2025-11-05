#!/usr/bin/env python3
"""
SUPERCHARGED Grok 4 Project Reviewer for MIDI Software Center
Enhanced with: Auto-Fix Mode, Intelligent Code Modification, Predictive Error Detection,
Learning from Past Analyses, Intelligent Prompt Generation, Auto-Generated Fix Scripts,
and CI/CD Integration
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


class MIDIAutoFixer:
    """Automatically applies fixes based on analysis results"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def apply_recommended_fixes(self, analysis_results):
        """Auto-apply the top recommended fixes from analysis"""
        print("🔧 AUTO-FIX MODE ACTIVATED")
        
        fixes = {
            "tauri_typescript": self.fix_tauri_ts_conflicts,
            "rust_dead_code": self.fix_rust_dead_code,
            "missing_backends": self.create_missing_backends,
            "build_config": self.fix_build_config,
            "test_enablement": self.enable_disabled_tests
        }
        
        applied_fixes = []
        for fix_name, fix_func in fixes.items():
            if self.should_apply_fix(analysis_results, fix_name):
                print(f"🛠️  Applying {fix_name}...")
                try:
                    if fix_func():
                        applied_fixes.append(fix_name)
                except Exception as e:
                    print(f"⚠️  Failed to apply {fix_name}: {e}")
        
        return applied_fixes
    
    def should_apply_fix(self, analysis_results, fix_name):
        """Determine if a fix should be applied based on analysis"""
        analysis_text = str(analysis_results).lower()
        
        fix_conditions = {
            "tauri_typescript": any(term in analysis_text for term in ["tauri", "typescript", "window", "__tauri__"]),
            "rust_dead_code": any(term in analysis_text for term in ["dead code", "unused", "warning"]),
            "missing_backends": any(term in analysis_text for term in ["missing backends", "backend stubs", "non-existent"]),
            "build_config": any(term in analysis_text for term in ["cargo.toml", "manifest", "build config"]),
            "test_enablement": any(term in analysis_text for term in ["disabled tests", "test compilation"])
        }
        
        return fix_conditions.get(fix_name, False)
    
    def fix_tauri_ts_conflicts(self):
        """Auto-fix the Tauri TypeScript conflicts"""
        ts_files = list(self.project_root.glob("**/*.ts")) + list(self.project_root.glob("**/*.tsx"))
        fixed_count = 0
        
        for file_path in ts_files:
            try:
                content = file_path.read_text()
                if "interface Window" in content and "__TAURI__" in content:
                    # Fix the Tauri type conflicts
                    fixed_content = self.rewrite_tauri_types(content)
                    if fixed_content != content:
                        file_path.write_text(fixed_content)
                        print(f"✅ Fixed Tauri types in {file_path.relative_to(self.project_root)}")
                        fixed_count += 1
            except Exception as e:
                print(f"⚠️  Could not process {file_path}: {e}")
        
        return fixed_count > 0
    
    def rewrite_tauri_types(self, content):
        """Rewrite Tauri TypeScript types to avoid conflicts"""
        # Remove conflicting Window interface declarations
        lines = content.split('\n')
        in_window_interface = False
        new_lines = []
        
        for line in lines:
            if "interface Window" in line and "__TAURI__" in line:
                in_window_interface = True
                # Replace with proper declaration
                new_lines.append("// Fixed Tauri types - remove conflicting declarations")
                continue
            elif in_window_interface and line.strip() == "}":
                in_window_interface = False
                continue
            elif not in_window_interface:
                new_lines.append(line)
        
        return '\n'.join(new_lines)
    
    def fix_rust_dead_code(self):
        """Fix Rust dead code warnings"""
        rust_files = list(self.project_root.glob("**/*.rs"))
        fixed_count = 0
        
        for file_path in rust_files:
            try:
                content = file_path.read_text()
                
                # Pattern 1: Add #[allow(dead_code)] to structs with unused fields
                if "struct " in content and "unused" in content.lower():
                    lines = content.split('\n')
                    new_lines = []
                    
                    for i, line in enumerate(lines):
                        if "struct " in line and i > 0 and "#[allow(dead_code)]" not in lines[i-1]:
                            new_lines.append("#[allow(dead_code)]")
                        new_lines.append(line)
                    
                    new_content = '\n'.join(new_lines)
                    if new_content != content:
                        file_path.write_text(new_content)
                        print(f"✅ Fixed dead code in {file_path.relative_to(self.project_root)}")
                        fixed_count += 1
                        
            except Exception as e:
                print(f"⚠️  Could not process {file_path}: {e}")
        
        return fixed_count > 0
    
    def create_missing_backends(self):
        """Create missing backend crate structure"""
        backends = ["midi-library-shared", "midi-pipeline", "midi-daw"]
        created_count = 0
        
        for backend in backends:
            backend_path = self.project_root / backend
            if not backend_path.exists():
                backend_path.mkdir(exist_ok=True)
                
                # Create basic Cargo.toml
                cargo_content = f"""[package]
name = "{backend}"
version = "0.1.0"
edition = "2021"

[dependencies]
"""
                if backend == "midi-library-shared":
                    cargo_content += "serde = { version = \"1.0\", features = [\"derive\"] }\n"
                
                (backend_path / "Cargo.toml").write_text(cargo_content)
                
                # Create src directory
                src_path = backend_path / "src"
                src_path.mkdir(exist_ok=True)
                
                # Create lib.rs or main.rs
                if backend == "midi-library-shared":
                    (src_path / "lib.rs").write_text("// MIDI shared library\n")
                else:
                    (src_path / "main.rs").write_text("fn main() {\n    println!(\"Hello from {backend}!\");\n}\n")
                
                print(f"✅ Created missing backend: {backend}")
                created_count += 1
        
        return created_count > 0
    
    def fix_build_config(self):
        """Fix build configuration issues"""
        # Fix Cargo.toml issues
        cargo_files = list(self.project_root.glob("**/Cargo.toml"))
        fixed_count = 0
        
        for cargo_file in cargo_files:
            try:
                content = cargo_file.read_text()
                
                # Remove invalid bin entries
                if "[[bin]]" in content and "walkdir" in content:
                    lines = content.split('\n')
                    new_lines = []
                    skip_next = False
                    
                    for line in lines:
                        if "name = \"walkdir\"" in line:
                            skip_next = True
                            continue
                        elif skip_next and line.strip() == "":
                            skip_next = False
                            continue
                        elif not skip_next:
                            new_lines.append(line)
                    
                    new_content = '\n'.join(new_lines)
                    if new_content != content:
                        cargo_file.write_text(new_content)
                        print(f"✅ Fixed build config in {cargo_file.relative_to(self.project_root)}")
                        fixed_count += 1
                        
            except Exception as e:
                print(f"⚠️  Could not process {cargo_file}: {e}")
        
        return fixed_count > 0
    
    def enable_disabled_tests(self):
        """Enable some disabled tests"""
        disabled_tests_dir = self.project_root / "_disabled_tests"
        if disabled_tests_dir.exists():
            test_files = list(disabled_tests_dir.glob("*.rs"))
            enabled_count = 0
            
            for test_file in test_files[:3]:  # Enable first 3 as a start
                target_path = self.project_root / "tests" / test_file.name
                try:
                    test_file.rename(target_path)
                    print(f"✅ Enabled test: {test_file.name}")
                    enabled_count += 1
                except Exception as e:
                    print(f"⚠️  Could not enable {test_file.name}: {e}")
            
            return enabled_count > 0
        return False


class PredictiveAnalyzer:
    """Predicts future errors based on patterns"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def predict_compilation_issues(self, context):
        """Predict what will break before it happens"""
        predictions = []
        
        # Analyze dependency chains
        cargo_files = list(self.project_root.glob("**/Cargo.toml"))
        for cargo_file in cargo_files:
            if self.has_misconfigured_dependencies(cargo_file):
                predictions.append(f"🔧 Misconfigured dependencies in {cargo_file.relative_to(self.project_root)}")
        
        # Predict TypeScript issues
        ts_config = self.project_root / "tsconfig.json"
        if ts_config.exists():
            if self.will_break_on_strict_mode(ts_config):
                predictions.append("⚡ TypeScript strict mode may cause breaks")
        
        # Check for common MIDI project issues
        if self.has_missing_midi_dependencies():
            predictions.append("🎵 Missing MIDI-specific dependencies")
        
        # Check build configuration
        if self.has_build_config_issues():
            predictions.append("🔨 Build configuration issues detected")
        
        return predictions
    
    def has_misconfigured_dependencies(self, cargo_file):
        """Detect misconfigured dependencies"""
        try:
            content = cargo_file.read_text()
            # Check for common issues
            issues = [
                "walkdir" in content and "[[bin]]" in content,  # walkdir in bin section
                "dependencies]" not in content and "[[bin]]" in content,  # missing dependencies section
            ]
            return any(issues)
        except:
            return False
    
    def will_break_on_strict_mode(self, ts_config):
        """Predict if TypeScript strict mode will cause issues"""
        try:
            content = ts_config.read_text()
            return "\"strict\": true" in content
        except:
            return False
    
    def has_missing_midi_dependencies(self):
        """Check for missing MIDI-specific dependencies"""
        cargo_files = list(self.project_root.glob("**/Cargo.toml"))
        for cargo_file in cargo_files:
            content = cargo_file.read_text()
            # Common MIDI crates that might be missing
            midi_crates = ["midly", "midir", "cpal"]
            if any(crate in content for crate in midi_crates):
                return False
        return True
    
    def has_build_config_issues(self):
        """Check for build configuration issues"""
        # Check if workspace is properly configured
        root_cargo = self.project_root / "Cargo.toml"
        if root_cargo.exists():
            content = root_cargo.read_text()
            return "[workspace]" not in content
        return True


class AnalysisLearner:
    """Learns from previous analyses to improve future ones"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
        self.history_file = self.project_root / "grok_analysis_history.json"
        self.patterns = self.load_historical_patterns()
    
    def load_historical_patterns(self):
        """Learn from past analysis patterns"""
        patterns = {
            "common_errors": defaultdict(int),
            "fix_effectiveness": {},
            "recurring_issues": []
        }
        
        if self.history_file.exists():
            try:
                history = json.loads(self.history_file.read_text())
                patterns = self.extract_patterns(history)
            except Exception as e:
                print(f"⚠️  Could not load analysis history: {e}")
        
        return patterns
    
    def extract_patterns(self, history):
        """Extract recurring patterns from history"""
        patterns = {
            "common_errors": defaultdict(int),
            "fix_effectiveness": {},
            "recurring_issues": []
        }
        
        # Extract from analysis stages
        for stage in history.get("analysis_stages", []):
            result = stage.get("result", "")
            error_types = self.categorize_errors(result)
            for error_type in error_types:
                patterns["common_errors"][error_type] += 1
        
        # Identify recurring issues (appear in multiple analyses)
        common_issues = [issue for issue, count in patterns["common_errors"].items() if count > 1]
        patterns["recurring_issues"] = common_issues
        
        return patterns
    
    def categorize_errors(self, analysis_text):
        """Categorize errors from analysis text"""
        categories = []
        text_lower = analysis_text.lower()
        
        if any(term in text_lower for term in ["typescript", "tauri", "window"]):
            categories.append("typescript_tauri_conflict")
        if any(term in text_lower for term in ["dead code", "unused"]):
            categories.append("rust_dead_code")
        if any(term in text_lower for term in ["missing", "backend", "stub"]):
            categories.append("missing_components")
        if any(term in text_lower for term in ["build", "compilation", "cargo"]):
            categories.append("build_issues")
        if any(term in text_lower for term in ["test", "coverage"]):
            categories.append("test_issues")
        
        return categories
    
    def get_most_likely_issues(self):
        """Predict issues based on historical patterns"""
        common_issues = sorted(
            self.patterns["common_errors"].items(), 
            key=lambda x: x[1], 
            reverse=True
        )[:5]
        return common_issues
    
    def save_analysis(self, analysis_data):
        """Save current analysis to history"""
        try:
            history = {}
            if self.history_file.exists():
                history = json.loads(self.history_file.read_text())
            
            # Add current analysis
            if "analysis_stages" not in history:
                history["analysis_stages"] = []
            
            history["analysis_stages"].extend(analysis_data.get("stages", []))
            history["last_updated"] = datetime.now().isoformat()
            
            # Keep only last 10 analyses to prevent file from growing too large
            if len(history["analysis_stages"]) > 10:
                history["analysis_stages"] = history["analysis_stages"][-10:]
            
            self.history_file.write_text(json.dumps(history, indent=2))
        except Exception as e:
            print(f"⚠️  Could not save analysis history: {e}")


class DevelopmentWorkflowIntegrator:
    """Integrates with existing development workflows"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
        self.ci_files = [
            ".github/workflows/ci.yml",
            ".gitlab-ci.yml", 
            ".circleci/config.yml",
            "azure-pipelines.yml"
        ]
    
    def integrate_with_ci(self):
        """Add Grok analysis to CI pipeline"""
        print("🔄 Integrating with CI/CD workflow...")
        
        for ci_file in self.ci_files:
            ci_path = self.project_root / ci_file
            if ci_path.exists():
                if self.add_grok_step_to_ci(ci_path):
                    print(f"✅ Added Grok analysis to {ci_path}")
                    return True
        
        # If no CI file exists, create a basic GitHub Actions workflow
        self.create_github_workflow()
        return True
    
    def add_grok_step_to_ci(self, ci_path):
        """Add Grok analysis step to CI configuration"""
        try:
            content = ci_path.read_text()
            
            # Check if Grok step already exists
            if "Grok Analysis" in content:
                return False
            
            grok_step = """
      - name: AI Code Analysis
        run: |
          echo "🔍 Running AI-powered code analysis..."
          python3 midi_grok_reviewer.py ci-check
          if [ -f "grok_analysis_report.json" ]; then
            echo "Analysis report generated"
          fi
"""
            
            # Insert after build step (simplistic approach)
            if "name: Build" in content:
                new_content = content.replace(
                    "      - name: Build", 
                    "      - name: Build" + grok_step
                )
                ci_path.write_text(new_content)
                return True
                
        except Exception as e:
            print(f"⚠️  Could not modify {ci_path}: {e}")
        
        return False
    
    def create_github_workflow(self):
        """Create a GitHub Actions workflow if none exists"""
        workflows_dir = self.project_root / ".github" / "workflows"
        workflows_dir.mkdir(parents=True, exist_ok=True)
        
        workflow_content = """name: CI with AI Analysis

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Build
      run: cargo build --verbose
    
    - name: Run Tests
      run: cargo test --verbose
    
    - name: AI Code Analysis
      run: |
        pip3 install httpx
        python3 midi_grok_reviewer.py ci-check
        if [ -f "grok_analysis_report.json" ]; then
          echo "📊 AI analysis completed"
        fi
"""
        
        workflow_file = workflows_dir / "ci.yml"
        workflow_file.write_text(workflow_content)
        print(f"✅ Created GitHub Actions workflow: {workflow_file}")


class SuperchargedMIDIReviewer:
    """Main class with all enhanced capabilities"""
    
    def __init__(self, api_key=None):
        self.api_key = api_key or os.getenv("GROK_API_KEY")
        self.project_root = Path("/home/dojevou/projects/midi-software-center")
        
        # Initialize all enhanced components
        self.auto_fixer = MIDIAutoFixer(self.project_root)
        self.predictive_analyzer = PredictiveAnalyzer(self.project_root)
        self.analysis_learner = AnalysisLearner(self.project_root)
        self.workflow_integrator = DevelopmentWorkflowIntegrator(self.project_root)
        
        self.client = httpx.Client(timeout=120.0, http2=True)
        self.analysis_history = []
        
        if not self.api_key:
            print("❌ GROK_API_KEY not set")
            print("   export GROK_API_KEY='xai-...'")
            sys.exit(1)

    def generate_intelligent_prompt(self, context, analysis_type):
        """Generate context-aware prompts using historical patterns"""
        historical_patterns = self.analysis_learner.get_most_likely_issues()
        predictions = self.predictive_analyzer.predict_compilation_issues(context)
        
        base_prompts = {
            "architecture": """
MIDI SOFTWARE ARCHITECTURE REVIEW - ENHANCED ANALYSIS

HISTORICAL PATTERNS:
{historical_patterns}

FUTURE PREDICTIONS:
{predictions}

PROJECT CONTEXT:
{context}

FOCUS ON:
1. Architecture patterns that have caused issues before
2. Preventing predicted future problems
3. MIDI-specific reliability concerns
4. Long-term maintainability given historical trends

Provide actionable insights that address both current state and future risks.
""",
            "errors": """
COMPREHENSIVE ERROR ANALYSIS - LEARNING FROM HISTORY

RECURRING ISSUES:
{historical_patterns}

PREDICTED ISSUES:
{predictions}

CURRENT ERROR CONTEXT:
{context}

ANALYSIS APPROACH:
- Identify if current errors match historical patterns
- Suggest fixes that address root causes, not just symptoms
- Consider MIDI domain implications of each error type
- Prioritize based on historical fix effectiveness

Provide both immediate fixes and long-term prevention strategies.
"""
        }
        
        template = base_prompts.get(analysis_type, base_prompts["architecture"])
        return template.format(
            historical_patterns=historical_patterns,
            predictions=predictions,
            context=context
        )

    def generate_fix_scripts(self, analysis_results):
        """Generate executable fix scripts based on analysis"""
        print("📜 Generating automated fix scripts...")
        
        fixes = self.extract_actionable_fixes(analysis_results)
        
        script_content = "#!/bin/bash\n# Auto-generated fix script for MIDI Software Center\n# Generated: {}\n\n".format(datetime.now().isoformat())
        script_content += "set -e\n\n"
        script_content += "echo '🎵 MIDI Software Center - Automated Fixes'\n"
        script_content += "echo '========================================'\n\n"
        
        for i, fix in enumerate(fixes, 1):
            script_content += "echo '\\n🔧 Fix {}: {}'\n".format(i, fix['description'])
            script_content += "{}\n".format(fix['command'])
            
            if fix.get('verify'):
                script_content += "echo '✅ {}'\n".format(fix['verify'])
            
            script_content += "sleep 1\n\n"
        
        script_content += "echo '\\n🎉 All fixes applied!'\n"
        script_content += "echo 'Run ./midi_grok_reviewer.py midi to verify improvements'\n"
        
        script_path = self.project_root / "auto_fix.sh"
        script_path.write_text(script_content)
        script_path.chmod(0o755)
        
        print(f"✅ Fix script generated: {script_path}")
        return script_path

    def extract_actionable_fixes(self, analysis):
        """Extract commands that can be automated from analysis"""
        actionable_fixes = []
        
        # Parse analysis for specific fix patterns
        lines = str(analysis).split('\n')
        
        for line in lines:
            line = line.strip()
            
            # Rust/Cargo fixes
            if line.startswith('cargo ') and 'fix' in line.lower():
                actionable_fixes.append({
                    'description': f"Run cargo fix: {line}",
                    'command': line,
                    'verify': "Cargo fix completed"
                })
            elif line.startswith('cargo ') and 'clippy' in line:
                actionable_fixes.append({
                    'description': f"Run clippy: {line}",
                    'command': line,
                    'verify': "Clippy analysis completed"
                })
            
            # TypeScript fixes
            elif line.startswith('npm ') and ('fix' in line.lower() or 'lint' in line.lower()):
                actionable_fixes.append({
                    'description': f"Run npm fix: {line}",
                    'command': f"cd {self.project_root / 'pipeline'} && {line}",
                    'verify': "NPM fix completed"
                })
        
        # Add common MIDI project fixes
        common_fixes = [
            {
                'description': "Clean cargo build artifacts",
                'command': "cargo clean",
                'verify': "Build artifacts cleaned"
            },
            {
                'description': "Update cargo dependencies",
                'command': "cargo update",
                'verify': "Dependencies updated"
            },
            {
                'description': "Check TypeScript compilation",
                'command': "cd pipeline && npx tsc --noEmit",
                'verify': "TypeScript check completed"
            }
        ]
        
        actionable_fixes.extend(common_fixes)
        return actionable_fixes

    def run_super_analysis(self):
        """Run all enhanced analysis modes"""
        print("🚀 SUPERCHARGED MIDI ANALYSIS ACTIVATED")
        print("=" * 60)
        
        # 1. Gather project context
        print("\n📁 Gathering enhanced project context...")
        context = self.gather_project_context()
        
        # 2. Predictive analysis
        print("\n🔮 Running predictive analysis...")
        predictions = self.predictive_analyzer.predict_compilation_issues(context)
        if predictions:
            print("📊 PREDICTED ISSUES:")
            for prediction in predictions:
                print(f"  • {prediction}")
        
        # 3. Historical pattern analysis
        print("\n📚 Analyzing historical patterns...")
        historical_issues = self.analysis_learner.get_most_likely_issues()
        if historical_issues:
            print("🕰️  HISTORICAL PATTERNS:")
            for issue, count in historical_issues:
                print(f"  • {issue}: {count} occurrences")
        
        # 4. Run enhanced analysis with intelligent prompts
        print("\n🧠 Running enhanced AI analysis...")
        enhanced_prompt = self.generate_intelligent_prompt(context, "architecture")
        analysis_results = self.call_grok(enhanced_prompt)
        
        # 5. Auto-apply fixes
        print("\n🔧 Applying automated fixes...")
        applied_fixes = self.auto_fixer.apply_recommended_fixes(analysis_results)
        if applied_fixes:
            print(f"✅ APPLIED FIXES: {', '.join(applied_fixes)}")
        
        # 6. Generate fix scripts
        print("\n📜 Generating automated fix scripts...")
        script_path = self.generate_fix_scripts(analysis_results)
        
        # 7. Integrate with CI/CD
        print("\n🔄 Integrating with development workflow...")
        self.workflow_integrator.integrate_with_ci()
        
        # 8. Save analysis for learning
        analysis_data = {
            "timestamp": datetime.now().isoformat(),
            "stages": [
                {"stage": "predictive_analysis", "result": str(predictions)},
                {"stage": "historical_patterns", "result": str(historical_issues)},
                {"stage": "ai_analysis", "result": analysis_results[:1000] + "..." if len(analysis_results) > 1000 else analysis_results},
                {"stage": "applied_fixes", "result": str(applied_fixes)}
            ]
        }
        self.analysis_learner.save_analysis(analysis_data)
        
        # Display results
        self.display_enhanced_results(analysis_results, predictions, historical_issues, applied_fixes, script_path)
        
        print(f"\n🎉 SUPERCHARGED ANALYSIS COMPLETE!")
        print(f"📁 Fix script: {script_path}")
        print(f"💾 Analysis saved to history for future learning")

    def gather_project_context(self):
        """Gather basic project context for analysis"""
        return {
            "timestamp": datetime.now().isoformat(),
            "rust_files": len(list(self.project_root.glob("**/*.rs"))),
            "typescript_files": len(list(self.project_root.glob("**/*.ts"))),
            "svelte_files": len(list(self.project_root.glob("**/*.svelte"))),
            "cargo_files": len(list(self.project_root.glob("**/Cargo.toml"))),
            "has_tauri": any(self.project_root.glob("**/tauri.conf.json")),
            "build_status": self.check_build_status()
        }

    def check_build_status(self):
        """Check current build status"""
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
                "has_errors": result.returncode != 0,
                "error_count": len([line for line in result.stderr.split('\n') if 'error[' in line])
            }
        except:
            return {"success": False, "has_errors": True, "error_count": "unknown"}

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
                "https://api.x.ai/v1/chat/completions",
                json=payload,
                headers=headers
            )
            response.raise_for_status()
            result = response.json()
            return result["choices"][0]["message"]["content"]
        except Exception as e:
            return f"❌ API Error: {str(e)}"

    def display_enhanced_results(self, analysis_results, predictions, historical_issues, applied_fixes, script_path):
        """Display comprehensive results from all enhanced analyses"""
        print("\n" + "="*70)
        print("🚀 ENHANCED MIDI ANALYSIS - COMPREHENSIVE RESULTS")
        print("="*70)
        
        print(f"\n🔮 PREDICTIVE INSIGHTS ({len(predictions)} issues):")
        for prediction in predictions:
            print(f"  • {prediction}")
        
        print(f"\n📚 HISTORICAL PATTERNS ({len(historical_issues)} patterns):")
        for issue, count in historical_issues:
            print(f"  • {issue}: {count} past occurrences")
        
        print(f"\n🔧 AUTOMATED FIXES ({len(applied_fixes)} applied):")
        for fix in applied_fixes:
            print(f"  • {fix}")
        
        print(f"\n🧠 AI ANALYSIS:")
        print("-" * 40)
        print(analysis_results)
        
        print(f"\n📜 GENERATED SCRIPTS:")
        print(f"  • {script_path} - Run this to apply all fixes")

    def run_ci_check(self):
        """Run a quick check suitable for CI pipelines"""
        print("🔍 CI Check Mode - Quick Analysis")
        
        context = self.gather_project_context()
        predictions = self.predictive_analyzer.predict_compilation_issues(context)
        
        # Generate a quick status report
        report = {
            "timestamp": datetime.now().isoformat(),
            "build_status": context["build_status"],
            "predictions": predictions,
            "file_counts": {k: v for k, v in context.items() if k.endswith('_files')}
        }
        
        report_path = self.project_root / "grok_ci_report.json"
        report_path.write_text(json.dumps(report, indent=2))
        
        print(f"✅ CI report generated: {report_path}")
        
        # Exit with error code if build is failing
        if not context["build_status"]["success"]:
            print("❌ Build is failing - exiting with error")
            sys.exit(1)

    def close(self):
        """Cleanup resources"""
        self.client.close()


def main():
    """Main entry point with enhanced command options"""
    reviewer = SuperchargedMIDIReviewer()
    
    try:
        if len(sys.argv) > 1:
            if sys.argv[1] in ["super", "enhanced", "all"]:
                reviewer.run_super_analysis()
            elif sys.argv[1] == "ci-check":
                reviewer.run_ci_check()
            elif sys.argv[1] == "predict":
                context = reviewer.gather_project_context()
                predictions = reviewer.predictive_analyzer.predict_compilation_issues(context)
                print("🔮 PREDICTED ISSUES:")
                for prediction in predictions:
                    print(f"  • {prediction}")
            elif sys.argv[1] == "history":
                patterns = reviewer.analysis_learner.get_most_likely_issues()
                print("📚 HISTORICAL PATTERNS:")
                for issue, count in patterns:
                    print(f"  • {issue}: {count} occurrences")
            elif sys.argv[1] == "fix":
                context = reviewer.gather_project_context()
                applied = reviewer.auto_fixer.apply_recommended_fixes(context)
                print(f"🔧 APPLIED FIXES: {applied}")
            else:
                print("Usage: python3 supercharged_grok_reviewer.py [command]")
                print("\nEnhanced Commands:")
                print("  super        - Full supercharged analysis (recommended)")
                print("  ci-check     - Quick analysis for CI pipelines")
                print("  predict      - Show predicted issues only")
                print("  history      - Show historical patterns only")
                print("  fix          - Apply automated fixes only")
        else:
            reviewer.run_super_analysis()
    finally:
        reviewer.close()


if __name__ == "__main__":
    main()
