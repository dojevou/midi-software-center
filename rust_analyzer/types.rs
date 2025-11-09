use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub project_root: PathBuf,
    pub timestamp: String,
    pub project_structure: ProjectStructure,
    pub build_errors: Vec<BuildError>,
    pub code_quality: CodeQualityReport,
    pub security: SecurityReport,
    pub dependencies: DependencyReport,
    pub workspace_config: WorkspaceConfig,
    pub ast_insights: AstInsights,
    pub midi_analysis: MidiAnalysis,
    pub performance_hints: Vec<PerformanceHint>,
    pub auto_fixes: Vec<AutoFix>,
}

impl AnalysisReport {
    pub fn has_critical_issues(&self) -> bool {
        !self.build_errors.is_empty() || self.security.has_critical_issues()
    }

    pub fn issue_count(&self) -> usize {
        self.build_errors.len()
            + self.code_quality.warnings
            + self.security.total_issues()
            + self.dependencies.vulnerabilities.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub cargo_files: usize,
    pub rust_files: usize,
    pub total_lines: usize,
    pub test_files: usize,
    pub bench_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildError {
    pub message: String,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub severity: ErrorSeverity,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityReport {
    pub warnings: usize,
    pub errors: usize,
    pub complexity_issues: Vec<ComplexityIssue>,
    pub style_issues: Vec<StyleIssue>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityIssue {
    pub function: String,
    pub file: PathBuf,
    pub line: usize,
    pub complexity: usize,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleIssue {
    pub message: String,
    pub file: PathBuf,
    pub line: usize,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub unsafe_blocks: Vec<UnsafeBlock>,
    pub panic_calls: Vec<PanicCall>,
    pub unwrap_calls: Vec<UnwrapCall>,
    pub vulnerabilities: Vec<Vulnerability>,
}

impl SecurityReport {
    pub fn has_critical_issues(&self) -> bool {
        self.vulnerabilities.iter().any(|v| v.severity == VulnerabilitySeverity::Critical)
    }

    pub fn total_issues(&self) -> usize {
        self.unsafe_blocks.len()
            + self.panic_calls.len()
            + self.unwrap_calls.len()
            + self.vulnerabilities.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeBlock {
    pub file: PathBuf,
    pub line: usize,
    pub has_safety_comment: bool,
    pub operations: Vec<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicCall {
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnwrapCall {
    pub file: PathBuf,
    pub line: usize,
    pub expression: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub package: String,
    pub version: String,
    pub advisory_id: String,
    pub title: String,
    pub severity: VulnerabilitySeverity,
    pub solution: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyReport {
    pub total_dependencies: usize,
    pub outdated: Vec<OutdatedDependency>,
    pub duplicates: Vec<DuplicateDependency>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub feature_analysis: FeatureAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedDependency {
    pub name: String,
    pub current: String,
    pub latest: String,
    pub compatible: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDependency {
    pub name: String,
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAnalysis {
    pub unused_features: Vec<String>,
    pub circular_features: Vec<Vec<String>>,
    pub optional_always_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub has_workspace: bool,
    pub members: Vec<String>,
    pub has_resolver_2: bool,
    pub has_release_profile: bool,
    pub has_lto: bool,
    pub optimization_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstInsights {
    pub trait_issues: Vec<TraitIssue>,
    pub lifetime_issues: Vec<LifetimeIssue>,
    pub generic_issues: Vec<GenericIssue>,
    pub ownership_patterns: Vec<OwnershipPattern>,
    pub macro_analysis: MacroAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitIssue {
    pub kind: TraitIssueKind,
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitIssueKind {
    MissingImplementation,
    MissingDerive,
    OrphanRule,
    BoundConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeIssue {
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub can_use_elision: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericIssue {
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipPattern {
    pub kind: OwnershipPatternKind,
    pub file: PathBuf,
    pub line: usize,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipPatternKind {
    ExcessiveClone,
    InteriorMutabilityOveruse,
    MissingCow,
    ArcRefCellPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroAnalysis {
    pub macro_usage: Vec<MacroUsage>,
    pub unsafe_macros: Vec<UnsafeMacro>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroUsage {
    pub name: String,
    pub count: usize,
    pub should_inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeMacro {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiAnalysis {
    pub real_time_issues: Vec<RealTimeIssue>,
    pub audio_thread_violations: Vec<AudioThreadViolation>,
    pub latency_concerns: Vec<LatencyConcern>,
    pub buffer_analysis: BufferAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeIssue {
    pub file: PathBuf,
    pub line: usize,
    pub kind: RealTimeIssueKind,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealTimeIssueKind {
    HeapAllocation,
    MutexLock,
    BlockingIO,
    SystemCall,
    PotentialPanic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioThreadViolation {
    pub function: String,
    pub file: PathBuf,
    pub line: usize,
    pub violation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConcern {
    pub file: PathBuf,
    pub line: usize,
    pub estimated_latency_us: f64,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferAnalysis {
    pub buffer_sizes: Vec<BufferSize>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSize {
    pub name: String,
    pub size: usize,
    pub is_optimal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHint {
    pub file: PathBuf,
    pub line: usize,
    pub kind: PerformanceHintKind,
    pub message: String,
    pub impact: PerformanceImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceHintKind {
    MissingInline,
    LargeStackAllocation,
    UnoptimizedLoop,
    MissingSimd,
    SuboptimalAlgorithm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PerformanceImpact {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFix {
    pub file: PathBuf,
    pub line: usize,
    pub kind: AutoFixKind,
    pub description: String,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoFixKind {
    AddDerive,
    RemoveUnusedImport,
    AddInlineAttribute,
    AddSafetyComment,
    ConvertUnwrapToQuestionMark,
    SimplifyLifetime,
}
