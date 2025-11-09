use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use walkdir::WalkDir;

use crate::types::*;

pub struct AstAnalyzer {
    project_root: PathBuf,
}

impl AstAnalyzer {
    pub fn new(project_root: &Path) -> Result<Self> {
        Ok(Self {
            project_root: project_root.to_path_buf(),
        })
    }

    pub async fn analyze_all(&self) -> Result<AstInsights> {
        let rust_files: Vec<PathBuf> = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
            .map(|e| e.path().to_path_buf())
            .collect();

        // Parallel analysis of all files
        let results: Vec<_> = rust_files
            .par_iter()
            .map(|file| self.analyze_file(file))
            .collect();

        let mut trait_issues = Vec::new();
        let mut lifetime_issues = Vec::new();
        let mut generic_issues = Vec::new();
        let mut ownership_patterns = Vec::new();
        let mut macro_usage_map: HashMap<String, usize> = HashMap::new();
        let mut unsafe_macros = Vec::new();

        for result in results {
            if let Ok(file_analysis) = result {
                trait_issues.extend(file_analysis.trait_issues);
                lifetime_issues.extend(file_analysis.lifetime_issues);
                generic_issues.extend(file_analysis.generic_issues);
                ownership_patterns.extend(file_analysis.ownership_patterns);

                for (macro_name, count) in file_analysis.macro_usage {
                    *macro_usage_map.entry(macro_name).or_insert(0) += count;
                }

                unsafe_macros.extend(file_analysis.unsafe_macros);
            }
        }

        let macro_usage: Vec<MacroUsage> = macro_usage_map
            .into_iter()
            .map(|(name, count)| MacroUsage {
                name,
                count,
                should_inline: count > 10,
            })
            .collect();

        Ok(AstInsights {
            trait_issues,
            lifetime_issues,
            generic_issues,
            ownership_patterns,
            macro_analysis: MacroAnalysis {
                macro_usage,
                unsafe_macros,
            },
        })
    }

    fn analyze_file(&self, file: &Path) -> Result<FileAstAnalysis> {
        let content = std::fs::read_to_string(file)?;
        let syntax = syn::parse_file(&content)?;

        let mut visitor = AstVisitor::new(file);
        visitor.visit_file(&syntax);

        Ok(FileAstAnalysis {
            trait_issues: visitor.trait_issues,
            lifetime_issues: visitor.lifetime_issues,
            generic_issues: visitor.generic_issues,
            ownership_patterns: visitor.ownership_patterns,
            macro_usage: visitor.macro_usage,
            unsafe_macros: visitor.unsafe_macros,
        })
    }
}

struct FileAstAnalysis {
    trait_issues: Vec<TraitIssue>,
    lifetime_issues: Vec<LifetimeIssue>,
    generic_issues: Vec<GenericIssue>,
    ownership_patterns: Vec<OwnershipPattern>,
    macro_usage: HashMap<String, usize>,
    unsafe_macros: Vec<UnsafeMacro>,
}

struct AstVisitor<'a> {
    file: &'a Path,
    trait_issues: Vec<TraitIssue>,
    lifetime_issues: Vec<LifetimeIssue>,
    generic_issues: Vec<GenericIssue>,
    ownership_patterns: Vec<OwnershipPattern>,
    macro_usage: HashMap<String, usize>,
    unsafe_macros: Vec<UnsafeMacro>,
}

impl<'a> AstVisitor<'a> {
    fn new(file: &'a Path) -> Self {
        Self {
            file,
            trait_issues: Vec::new(),
            lifetime_issues: Vec::new(),
            generic_issues: Vec::new(),
            ownership_patterns: Vec::new(),
            macro_usage: HashMap::new(),
            unsafe_macros: Vec::new(),
        }
    }
}

impl<'ast, 'a> Visit<'ast> for AstVisitor<'a> {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        // Check for missing derives
        let has_clone = node.attrs.iter().any(|attr| {
            attr.path().is_ident("derive")
                && attr.parse_args::<syn::Ident>()
                    .map(|id| id == "Clone")
                    .unwrap_or(false)
        });

        let has_debug = node.attrs.iter().any(|attr| {
            attr.path().is_ident("derive")
                && attr.parse_args::<syn::Ident>()
                    .map(|id| id == "Debug")
                    .unwrap_or(false)
        });

        if !has_clone || !has_debug {
            let mut missing = Vec::new();
            if !has_clone {
                missing.push("Clone");
            }
            if !has_debug {
                missing.push("Debug");
            }

            self.trait_issues.push(TraitIssue {
                kind: TraitIssueKind::MissingDerive,
                file: self.file.to_path_buf(),
                line: node.struct_token.span.start().line,
                message: format!("Struct '{}' missing common derives", node.ident),
                suggestion: format!("Add #[derive({})]", missing.join(", ")),
            });
        }

        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Analyze lifetimes
        if !node.sig.generics.params.is_empty() {
            let lifetime_count = node
                .sig
                .generics
                .params
                .iter()
                .filter(|p| matches!(p, syn::GenericParam::Lifetime(_)))
                .count();

            if lifetime_count > 0 {
                // Check if lifetime elision could be used
                if can_use_lifetime_elision(&node.sig) {
                    self.lifetime_issues.push(LifetimeIssue {
                        file: self.file.to_path_buf(),
                        line: node.sig.fn_token.span.start().line,
                        message: format!("Function '{}' has explicit lifetimes", node.sig.ident),
                        can_use_elision: true,
                    });
                }
            }
        }

        // Analyze generics
        let generic_count = node
            .sig
            .generics
            .params
            .iter()
            .filter(|p| matches!(p, syn::GenericParam::Type(_)))
            .count();

        if generic_count > 3 {
            self.generic_issues.push(GenericIssue {
                file: self.file.to_path_buf(),
                line: node.sig.fn_token.span.start().line,
                message: format!(
                    "Function '{}' has {} generic parameters",
                    node.sig.ident, generic_count
                ),
                suggestion: "Consider refactoring to reduce generic complexity".to_string(),
            });
        }

        // Count clone calls
        let clone_count = count_clone_calls(&node.block);
        if clone_count > 5 {
            self.ownership_patterns.push(OwnershipPattern {
                kind: OwnershipPatternKind::ExcessiveClone,
                file: self.file.to_path_buf(),
                line: node.sig.fn_token.span.start().line,
                suggestion: format!(
                    "Function '{}' calls .clone() {} times - consider using references or Arc/Rc",
                    node.sig.ident, clone_count
                ),
            });
        }

        // Check for missing inline on small functions
        if should_be_inline(&node) && !has_inline_attr(&node) {
            // This could be a performance hint rather than ownership pattern
            // But we'll track it here for now
        }

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        // Check for trait implementations
        if let Some((_, trait_path, _)) = &node.trait_ {
            let trait_name = quote::quote!(#trait_path).to_string();

            // Check for common MIDI traits
            if trait_name.contains("MidiDevice") || trait_name.contains("AudioProcessor") {
                // Could add specific checks for MIDI trait implementations
            }
        }

        syn::visit::visit_item_impl(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let macro_name = node
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        *self.macro_usage.entry(macro_name.clone()).or_insert(0) += 1;

        // Check for unsafe macros
        if macro_name.contains("unsafe") {
            self.unsafe_macros.push(UnsafeMacro {
                name: macro_name,
                file: self.file.to_path_buf(),
                line: 0, // TODO: Get actual line number
            });
        }

        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        // Unsafe blocks are already handled in security analysis
        syn::visit::visit_expr_unsafe(self, node);
    }
}

fn can_use_lifetime_elision(sig: &syn::Signature) -> bool {
    // Simplified check - in reality this is more complex
    let lifetime_params = sig
        .generics
        .params
        .iter()
        .filter(|p| matches!(p, syn::GenericParam::Lifetime(_)))
        .count();

    let input_refs = sig
        .inputs
        .iter()
        .filter(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                matches!(&*pat_type.ty, syn::Type::Reference(_))
            } else {
                false
            }
        })
        .count();

    // If there's only one input reference, lifetime elision can likely be used
    lifetime_params == 1 && input_refs == 1
}

fn count_clone_calls(block: &syn::Block) -> usize {
    struct CloneCounter {
        count: usize,
    }

    impl<'ast> Visit<'ast> for CloneCounter {
        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            if node.method == "clone" {
                self.count += 1;
            }
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    let mut counter = CloneCounter { count: 0 };
    counter.visit_block(block);
    counter.count
}

fn should_be_inline(func: &syn::ItemFn) -> bool {
    // Heuristic: small functions should be inline
    let stmt_count = func.block.stmts.len();
    stmt_count < 10 && !func.sig.asyncness.is_some()
}

fn has_inline_attr(func: &syn::ItemFn) -> bool {
    func.attrs
        .iter()
        .any(|attr| attr.path().is_ident("inline"))
}
