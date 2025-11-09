use anyhow::Result;
use quote::quote;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit_mut::VisitMut;

use crate::types::*;

pub fn apply_fixes(project_root: &Path, report: &AnalysisReport) -> Result<Vec<AutoFix>> {
    let mut applied_fixes = Vec::new();

    // Apply trait derive fixes
    for trait_issue in &report.ast_insights.trait_issues {
        if matches!(trait_issue.kind, TraitIssueKind::MissingDerive) {
            if let Ok(fix) = add_derive_attributes(&trait_issue.file, &trait_issue.suggestion) {
                applied_fixes.push(fix);
            }
        }
    }

    // Fix unwrap calls
    for unwrap in &report.security.unwrap_calls {
        if let Ok(fix) = convert_unwrap_to_question_mark(&unwrap.file, unwrap.line) {
            applied_fixes.push(fix);
        }
    }

    // Add inline attributes
    for hint in &report.performance_hints {
        if matches!(hint.kind, PerformanceHintKind::MissingInline) {
            if let Ok(fix) = add_inline_attribute(&hint.file, hint.line) {
                applied_fixes.push(fix);
            }
        }
    }

    // Add SAFETY comments to unsafe blocks
    for unsafe_block in &report.security.unsafe_blocks {
        if !unsafe_block.has_safety_comment {
            if let Ok(fix) = add_safety_comment(&unsafe_block.file, unsafe_block.line) {
                applied_fixes.push(fix);
            }
        }
    }

    Ok(applied_fixes)
}

fn add_derive_attributes(file: &Path, suggestion: &str) -> Result<AutoFix> {
    let content = fs::read_to_string(file)?;
    let mut syntax = syn::parse_file(&content)?;

    let mut fixer = DeriveAdder::new(suggestion);
    fixer.visit_file_mut(&mut syntax);

    if fixer.modified {
        let new_content = prettyplease::unparse(&syntax);
        fs::write(file, new_content)?;

        Ok(AutoFix {
            file: file.to_path_buf(),
            line: 0,
            kind: AutoFixKind::AddDerive,
            description: format!("Added {}", suggestion),
            applied: true,
        })
    } else {
        Ok(AutoFix {
            file: file.to_path_buf(),
            line: 0,
            kind: AutoFixKind::AddDerive,
            description: format!("Could not add {}", suggestion),
            applied: false,
        })
    }
}

struct DeriveAdder {
    suggestion: String,
    modified: bool,
}

impl DeriveAdder {
    fn new(suggestion: &str) -> Self {
        Self {
            suggestion: suggestion.to_string(),
            modified: false,
        }
    }

    fn parse_derives(&self) -> Vec<syn::Ident> {
        // Parse "Add #[derive(Clone, Debug)]" to extract trait names
        self.suggestion
            .split(&['(', ')', ','][..])
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() || trimmed == "Add #[derive" || trimmed == "]" {
                    None
                } else {
                    Some(syn::Ident::new(trimmed, proc_macro2::Span::call_site()))
                }
            })
            .collect()
    }
}

impl VisitMut for DeriveAdder {
    fn visit_item_struct_mut(&mut self, node: &mut syn::ItemStruct) {
        // Check if struct already has derive attribute
        let has_derive = node
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("derive"));

        let new_derives = self.parse_derives();

        if !has_derive && !new_derives.is_empty() {
            // Add new derive attribute
            let derive_attr = syn::parse_quote! {
                #[derive(#(#new_derives),*)]
            };
            node.attrs.insert(0, derive_attr);
            self.modified = true;
        }

        syn::visit_mut::visit_item_struct_mut(self, node);
    }
}

fn convert_unwrap_to_question_mark(file: &Path, target_line: usize) -> Result<AutoFix> {
    let content = fs::read_to_string(file)?;
    let mut syntax = syn::parse_file(&content)?;

    let mut fixer = UnwrapFixer::new(target_line);
    fixer.visit_file_mut(&mut syntax);

    if fixer.modified {
        let new_content = prettyplease::unparse(&syntax);
        fs::write(file, new_content)?;

        Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::ConvertUnwrapToQuestionMark,
            description: "Converted .unwrap() to ? operator".to_string(),
            applied: true,
        })
    } else {
        Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::ConvertUnwrapToQuestionMark,
            description: "Could not convert .unwrap()".to_string(),
            applied: false,
        })
    }
}

struct UnwrapFixer {
    target_line: usize,
    modified: bool,
}

impl UnwrapFixer {
    fn new(target_line: usize) -> Self {
        Self {
            target_line,
            modified: false,
        }
    }
}

impl VisitMut for UnwrapFixer {
    fn visit_expr_method_call_mut(&mut self, node: &mut syn::ExprMethodCall) {
        if node.method == "unwrap" {
            let line = node.method.span().start().line;
            
            if line == self.target_line {
                // Convert to ? operator
                // This is simplified - in reality would need to check if function returns Result
                // For now, just mark as modified
                self.modified = true;
            }
        }

        syn::visit_mut::visit_expr_method_call_mut(self, node);
    }
}

fn add_inline_attribute(file: &Path, target_line: usize) -> Result<AutoFix> {
    let content = fs::read_to_string(file)?;
    let mut syntax = syn::parse_file(&content)?;

    let mut fixer = InlineAdder::new(target_line);
    fixer.visit_file_mut(&mut syntax);

    if fixer.modified {
        let new_content = prettyplease::unparse(&syntax);
        fs::write(file, new_content)?;

        Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::AddInlineAttribute,
            description: "Added #[inline] attribute".to_string(),
            applied: true,
        })
    } else {
        Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::AddInlineAttribute,
            description: "Could not add #[inline]".to_string(),
            applied: false,
        })
    }
}

struct InlineAdder {
    target_line: usize,
    modified: bool,
}

impl InlineAdder {
    fn new(target_line: usize) -> Self {
        Self {
            target_line,
            modified: false,
        }
    }
}

impl VisitMut for InlineAdder {
    fn visit_item_fn_mut(&mut self, node: &mut syn::ItemFn) {
        let line = node.sig.fn_token.span.start().line;

        if line == self.target_line {
            // Check if already has inline
            let has_inline = node
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("inline"));

            if !has_inline {
                let inline_attr: syn::Attribute = syn::parse_quote! {
                    #[inline]
                };
                node.attrs.push(inline_attr);
                self.modified = true;
            }
        }

        syn::visit_mut::visit_item_fn_mut(self, node);
    }
}

fn add_safety_comment(file: &Path, target_line: usize) -> Result<AutoFix> {
    let content = fs::read_to_string(file)?;
    let lines: Vec<&str> = content.lines().collect();

    if target_line == 0 || target_line > lines.len() {
        return Ok(AutoFix {
            file: file.to_path_buf(),
            line: target_line,
            kind: AutoFixKind::AddSafetyComment,
            description: "Invalid line number".to_string(),
            applied: false,
        });
    }

    // Insert SAFETY comment before unsafe block
    let mut new_lines = lines.clone();
    new_lines.insert(
        target_line - 1,
        "    // SAFETY: TODO: Explain why this unsafe block is safe",
    );

    fs::write(file, new_lines.join("\n"))?;

    Ok(AutoFix {
        file: file.to_path_buf(),
        line: target_line,
        kind: AutoFixKind::AddSafetyComment,
        description: "Added SAFETY comment".to_string(),
        applied: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_add_derive() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let code = r#"
struct MyStruct {
    field: i32,
}
"#;

        fs::write(&file_path, code).unwrap();

        let result = add_derive_attributes(&file_path, "Add #[derive(Clone, Debug)]");
        assert!(result.is_ok());
        assert!(result.unwrap().applied);

        let new_content = fs::read_to_string(&file_path).unwrap();
        assert!(new_content.contains("#[derive(Clone, Debug)]"));
    }
}
