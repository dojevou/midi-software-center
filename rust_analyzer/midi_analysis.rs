use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use walkdir::WalkDir;

use crate::types::*;

pub struct MidiAnalyzer {
    project_root: PathBuf,
}

impl MidiAnalyzer {
    pub fn new(project_root: &Path) -> Result<Self> {
        Ok(Self {
            project_root: project_root.to_path_buf(),
        })
    }

    pub async fn analyze(&self) -> Result<MidiAnalysis> {
        // Find MIDI-related files
        let midi_files = self.find_midi_files()?;

        // Parallel analysis
        let results: Vec<_> = midi_files
            .par_iter()
            .map(|file| self.analyze_midi_file(file))
            .collect();

        let mut real_time_issues = Vec::new();
        let mut audio_thread_violations = Vec::new();
        let mut latency_concerns = Vec::new();

        for result in results {
            if let Ok(analysis) = result {
                real_time_issues.extend(analysis.real_time_issues);
                audio_thread_violations.extend(analysis.audio_thread_violations);
                latency_concerns.extend(analysis.latency_concerns);
            }
        }

        let buffer_analysis = self.analyze_buffers()?;

        Ok(MidiAnalysis {
            real_time_issues,
            audio_thread_violations,
            latency_concerns,
            buffer_analysis,
        })
    }

    fn find_midi_files(&self) -> Result<Vec<PathBuf>> {
        let files: Vec<PathBuf> = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map(|ext| ext == "rs").unwrap_or(false)
                    && (e.path().to_string_lossy().contains("midi")
                        || e.path().to_string_lossy().contains("audio"))
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        Ok(files)
    }

    fn analyze_midi_file(&self, file: &Path) -> Result<MidiFileAnalysis> {
        let content = std::fs::read_to_string(file)?;
        let syntax = syn::parse_file(&content)?;

        let mut visitor = MidiVisitor::new(file);
        visitor.visit_file(&syntax);

        Ok(MidiFileAnalysis {
            real_time_issues: visitor.real_time_issues,
            audio_thread_violations: visitor.audio_thread_violations,
            latency_concerns: visitor.latency_concerns,
        })
    }

    fn analyze_buffers(&self) -> Result<BufferAnalysis> {
        let mut buffer_sizes = Vec::new();
        let mut recommendations = Vec::new();

        // Scan for buffer size constants
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry
                .path()
                .extension()
                .map(|ext| ext == "rs")
                .unwrap_or(false)
            {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    // Look for buffer size patterns
                    if content.contains("BUFFER_SIZE") || content.contains("buffer_size") {
                        // Parse and analyze
                        if let Some(size) = extract_buffer_size(&content) {
                            let is_optimal = is_optimal_buffer_size(size);
                            buffer_sizes.push(BufferSize {
                                name: "BUFFER_SIZE".to_string(),
                                size,
                                is_optimal,
                            });

                            if !is_optimal {
                                recommendations.push(format!(
                                    "Buffer size {} may not be optimal for low-latency audio",
                                    size
                                ));
                            }
                        }
                    }
                }
            }
        }

        if buffer_sizes.is_empty() {
            recommendations.push("No explicit buffer sizes found - ensure proper buffering for MIDI/audio".to_string());
        }

        Ok(BufferAnalysis {
            buffer_sizes,
            recommendations,
        })
    }
}

struct MidiFileAnalysis {
    real_time_issues: Vec<RealTimeIssue>,
    audio_thread_violations: Vec<AudioThreadViolation>,
    latency_concerns: Vec<LatencyConcern>,
}

struct MidiVisitor<'a> {
    file: &'a Path,
    real_time_issues: Vec<RealTimeIssue>,
    audio_thread_violations: Vec<AudioThreadViolation>,
    latency_concerns: Vec<LatencyConcern>,
    in_audio_callback: bool,
}

impl<'a> MidiVisitor<'a> {
    fn new(file: &'a Path) -> Self {
        Self {
            file,
            real_time_issues: Vec::new(),
            audio_thread_violations: Vec::new(),
            latency_concerns: Vec::new(),
            in_audio_callback: false,
        }
    }

    fn check_for_allocations(&mut self, expr: &syn::Expr, line: usize) {
        let expr_str = quote::quote!(#expr).to_string();

        // Check for heap allocations
        if expr_str.contains("Box::new")
            || expr_str.contains("Vec::new")
            || expr_str.contains("String::from")
            || expr_str.contains(".to_string()")
            || expr_str.contains("format!")
        {
            self.real_time_issues.push(RealTimeIssue {
                file: self.file.to_path_buf(),
                line,
                kind: RealTimeIssueKind::HeapAllocation,
                message: "Heap allocation in real-time code".to_string(),
                suggestion: "Pre-allocate or use stack-based alternatives".to_string(),
            });
        }

        // Check for Mutex locks
        if expr_str.contains("Mutex") || expr_str.contains(".lock()") {
            self.real_time_issues.push(RealTimeIssue {
                file: self.file.to_path_buf(),
                line,
                kind: RealTimeIssueKind::MutexLock,
                message: "Mutex lock in real-time code".to_string(),
                suggestion: "Use lock-free data structures (e.g., crossbeam, atomic)".to_string(),
            });
        }

        // Check for blocking I/O
        if expr_str.contains("std::fs::") || expr_str.contains("std::io::") {
            self.real_time_issues.push(RealTimeIssue {
                file: self.file.to_path_buf(),
                line,
                kind: RealTimeIssueKind::BlockingIO,
                message: "Blocking I/O in real-time code".to_string(),
                suggestion: "Move I/O to separate thread".to_string(),
            });
        }

        // Check for potential panics
        if expr_str.contains("unwrap()") || expr_str.contains("expect(") {
            self.real_time_issues.push(RealTimeIssue {
                file: self.file.to_path_buf(),
                line,
                kind: RealTimeIssueKind::PotentialPanic,
                message: "Potential panic in real-time code".to_string(),
                suggestion: "Use proper error handling without panicking".to_string(),
            });
        }
    }
}

impl<'ast, 'a> Visit<'ast> for MidiVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let func_name = node.sig.ident.to_string();

        // Detect audio/MIDI callback functions
        if func_name.contains("process")
            || func_name.contains("callback")
            || func_name.contains("audio")
            || func_name.contains("midi")
            || func_name.contains("handle_event")
        {
            self.in_audio_callback = true;

            // Check if function has #[inline] attribute for hot paths
            let has_inline = node.attrs.iter().any(|attr| attr.path().is_ident("inline"));

            if !has_inline && is_likely_hot_path(&func_name) {
                self.latency_concerns.push(LatencyConcern {
                    file: self.file.to_path_buf(),
                    line: node.sig.fn_token.span.start().line,
                    estimated_latency_us: 10.0,
                    suggestion: format!("Add #[inline] to hot path function '{}'", func_name),
                });
            }
        }

        syn::visit::visit_item_fn(self, node);

        self.in_audio_callback = false;
    }

    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        if self.in_audio_callback {
            // Get line number
            let line = match node {
                syn::Expr::Call(call) => call.paren_token.span.start().line,
                syn::Expr::MethodCall(call) => call.method.span().start().line,
                _ => 0,
            };

            self.check_for_allocations(node, line);
        }

        syn::visit::visit_expr(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.in_audio_callback {
            let method_name = node.method.to_string();

            // Check for sleep or blocking operations
            if method_name.contains("sleep") || method_name.contains("wait") {
                self.audio_thread_violations.push(AudioThreadViolation {
                    function: "unknown".to_string(),
                    file: self.file.to_path_buf(),
                    line: node.method.span().start().line,
                    violation: format!("Blocking call '{}' in audio thread", method_name),
                });
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

fn extract_buffer_size(content: &str) -> Option<usize> {
    use regex::Regex;
    
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"BUFFER_SIZE\s*=\s*(\d+)").unwrap();
    }

    RE.captures(content)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn is_optimal_buffer_size(size: usize) -> bool {
    // Optimal buffer sizes are typically powers of 2 between 128 and 2048
    size.is_power_of_two() && size >= 128 && size <= 2048
}

fn is_likely_hot_path(func_name: &str) -> bool {
    func_name.contains("process")
        || func_name.contains("handle")
        || func_name.contains("midi_event")
        || func_name.contains("audio_callback")
}
