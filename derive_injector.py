#!/usr/bin/env python3
"""
Derive Macro Injector - Automatically adds missing derive macros
Category 6: 18 errors related to missing trait implementations
"""

import re
from pathlib import Path
from typing import List, Set, Dict, Tuple
import sys

class DeriveMacroInjector:
    def __init__(self, rust_root: str = '.'):
        self.rust_root = Path(rust_root)
        self.fixes_applied = 0
        self.files_modified = 0
        
        # Map error patterns to required derive macros
        self.error_trait_map = {
            "cannot apply '=='": ['PartialEq'],
            "is not satisfied": ['Debug', 'Clone', 'Serialize', 'Deserialize'],
            "binary operation '==' cannot be applied": ['PartialEq'],
            "the trait bound": ['Debug', 'Clone', 'Serialize', 'Deserialize'],
        }
    
    def find_struct_definitions(self, content: str) -> List[Tuple[str, int, str]]:
        """
        Find all struct definitions
        Returns: List of (struct_name, line_number, full_definition)
        """
        pattern = re.compile(
            r'(?:pub\s+)?(?:async\s+)?struct\s+(\w+)[^{]*\{[^}]*\}',
            re.MULTILINE | re.DOTALL
        )
        matches = []
        for match in pattern.finditer(content):
            struct_name = match.group(1)
            line_num = content[:match.start()].count('\n')
            matches.append((struct_name, line_num, match.group(0)))
        
        return matches
    
    def get_existing_derives(self, struct_def: str) -> Set[str]:
        """Extract existing derive macros from struct definition"""
        derive_pattern = r'#\[derive\(([^)]+)\)\]'
        match = re.search(derive_pattern, struct_def)
        
        if match:
            derives = match.group(1)
            return set(d.strip() for d in derives.split(','))
        
        return set()
    
    def inject_derives(self, struct_def: str, required_derives: List[str]) -> Tuple[str, int]:
        """
        Add required derive macros to struct definition
        Returns: (modified_def, number_of_derives_added)
        """
        existing = self.get_existing_derives(struct_def)
        new_derives = set(required_derives) - existing
        
        if not new_derives:
            return struct_def, 0
        
        all_derives = sorted(list(existing | set(required_derives)))
        derives_str = ', '.join(all_derives)
        
        # Check if struct already has #[derive(...)]
        if '#[derive(' in struct_def:
            # Replace existing derive
            new_def = re.sub(
                r'#\[derive\([^)]+\)\]',
                f'#[derive({derives_str})]',
                struct_def
            )
        else:
            # Add new derive before struct
            new_def = f'#[derive({derives_str})]\n{struct_def}'
        
        return new_def, len(new_derives)
    
    def fix_tagged_response(self, content: str) -> Tuple[str, int]:
        """Fix TagResponse struct - needs PartialEq"""
        fixes = 0
        
        # Find TagResponse struct
        pattern = re.compile(
            r'(#\[derive\(([^)]*)\)\]\s*)?(?:pub\s+)?struct\s+TagResponse\s*\{[^}]*\}',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_tag_response(match):
            nonlocal fixes
            struct_def = match.group(0)
            existing_derives = match.group(2) or ""
            
            required = {'Debug', 'Clone', 'Serialize', 'Deserialize', 'PartialEq'}
            existing_set = set(d.strip() for d in existing_derives.split(',') if d.strip())
            needed = required - existing_set
            
            if needed:
                all_derives = sorted(list(existing_set | required))
                derives_str = ', '.join(all_derives)
                new_struct = re.sub(
                    r'(#\[derive\([^)]*\)\]\s*)?(?=(?:pub\s+)?struct)',
                    f'#[derive({derives_str})]\n',
                    struct_def
                )
                fixes += len(needed)
                return new_struct
            
            return struct_def
        
        new_content = pattern.sub(replace_tag_response, content)
        return new_content, fixes
    
    def fix_import_progress(self, content: str) -> Tuple[str, int]:
        """Fix ImportProgress struct - needs Deserialize"""
        fixes = 0
        
        pattern = re.compile(
            r'(#\[derive\(([^)]*)\)\]\s*)?(?:pub\s+)?struct\s+ImportProgress\s*\{[^}]*\}',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_import_progress(match):
            nonlocal fixes
            struct_def = match.group(0)
            existing_derives = match.group(2) or ""
            
            required = {'Debug', 'Clone', 'Serialize', 'Deserialize'}
            existing_set = set(d.strip() for d in existing_derives.split(',') if d.strip())
            needed = required - existing_set
            
            if needed:
                all_derives = sorted(list(existing_set | required))
                derives_str = ', '.join(all_derives)
                new_struct = re.sub(
                    r'(#\[derive\([^)]*\)\]\s*)?(?=(?:pub\s+)?struct)',
                    f'#[derive({derives_str})]\n',
                    struct_def
                )
                fixes += len(needed)
                return new_struct
            
            return struct_def
        
        new_content = pattern.sub(replace_import_progress, content)
        return new_content, fixes
    
    def fix_emitter_generics(self, content: str) -> Tuple[str, int]:
        """Fix missing generics in Emitter trait usage"""
        fixes = 0
        
        # Pattern: impl Emitter without generic type
        pattern = re.compile(
            r'impl\s+Emitter\s*<(?!\w)([^>]*)>',
            re.MULTILINE
        )
        
        # This requires manual inspection - just mark for review
        if pattern.search(content):
            print("  ⚠️  Manual review needed for Emitter<T> generics")
        
        return content, fixes
    
    def fix_file(self, file_path: Path) -> int:
        """Apply all derive macro fixes to a file"""
        try:
            content = file_path.read_text(encoding='utf-8')
        except Exception as e:
            print(f"  ❌ Cannot read {file_path}: {e}")
            return 0
        
        original_content = content
        total_fixes = 0
        
        # Apply specific struct fixes
        content, tag_fixes = self.fix_tagged_response(content)
        total_fixes += tag_fixes
        
        content, import_fixes = self.fix_import_progress(content)
        total_fixes += import_fixes
        
        # Try generic struct fixes for PartialEq
        # Look for structs used in comparisons
        comparison_pattern = re.compile(
            r'assert_eq!\(\s*(\w+),\s*(\w+)\s*\)',
            re.MULTILINE
        )
        
        for match in comparison_pattern.finditer(content):
            # These need PartialEq and Eq if not already derived
            struct_name = match.group(1)
            pattern = re.compile(
                rf'(?:pub\s+)?struct\s+{struct_name}\s*\{{',
                re.MULTILINE
            )
            if pattern.search(content):
                # Found the struct, ensure it has PartialEq
                # This is approximate - real solution would be more sophisticated
                pass
        
        if content != original_content:
            file_path.write_text(content, encoding='utf-8')
            self.files_modified += 1
            return total_fixes
        
        return 0
    
    def run(self) -> bool:
        """Run the injector on all Rust files"""
        rust_files = list(self.rust_root.rglob('*.rs'))
        
        if not rust_files:
            print(f"❌ No Rust files found in {self.rust_root}")
            return False
        
        print(f"🔍 Found {len(rust_files)} Rust files")
        print(f"🔧 Injecting missing derive macros...\n")
        
        for file_path in rust_files:
            fixes = self.fix_file(file_path)
            if fixes > 0:
                print(f"  ✅ {file_path.relative_to(self.rust_root)}: {fixes} macros added")
                self.fixes_applied += fixes
        
        print(f"\n{'='*60}")
        print(f"📊 RESULTS")
        print(f"{'='*60}")
        print(f"Files modified:     {self.files_modified}")
        print(f"Total fixes:        {self.fixes_applied}")
        
        if self.fixes_applied > 0:
            print(f"\n✅ Derive macros injected!")
            print(f"Next: Run 'cargo build' to verify")
            return True
        else:
            print(f"\n⚠️  No missing derives found")
            return False


if __name__ == '__main__':
    rust_root = sys.argv[1] if len(sys.argv) > 1 else '.'
    
    injector = DeriveMacroInjector(rust_root)
    success = injector.run()
    
    sys.exit(0 if success else 1)
