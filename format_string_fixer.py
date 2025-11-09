#!/usr/bin/env python3
"""
Format String Fixer - Automatically fixes format string errors in Rust code
Category 1: 28 errors of type "invalid reference to positional argument N"
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple, Dict
import subprocess

class FormatStringFixer:
    def __init__(self, rust_root: str = '.'):
        self.rust_root = Path(rust_root)
        self.patterns = self._compile_patterns()
        self.fixes_applied = 0
        self.files_modified = 0
    
    def _compile_patterns(self) -> Dict[str, re.Pattern]:
        """Compile regex patterns for format string errors"""
        return {
            # format!("{0}") with no args
            'format_indexed': re.compile(r'format!\("([^"]*\{(\d+)[^"]*)"(?:\s*,?\s*)?(?=\))', re.MULTILINE),
            
            # println!("{0}") with no args
            'println_indexed': re.compile(r'println!\("([^"]*\{(\d+)[^"]*)"(?:\s*,?\s*)?(?=\))', re.MULTILINE),
            
            # eprintln!("{0}") with no args
            'eprintln_indexed': re.compile(r'eprintln!\("([^"]*\{(\d+)[^"]*)"(?:\s*,?\s*)?(?=\))', re.MULTILINE),
            
            # write!/writeln! with indexed args but no values
            'write_indexed': re.compile(r'write[ln]?\(&?[^,]+,\s*"([^"]*\{(\d+)[^"]*)"(?:\s*,?\s*)?(?=\))', re.MULTILINE),
            
            # format!("{1}") but only 1 arg exists - this is trickier
            'format_mismatch': re.compile(
                r'(format|println|eprintln|writeln?)\!\("([^"]*\{(\d+)[^"]*)"(?:,\s*([^\)]+))?\)',
                re.MULTILINE
            ),
        }
    
    def find_rust_files(self) -> List[Path]:
        """Find all Rust files in project"""
        return list(self.rust_root.rglob('*.rs'))
    
    def analyze_format_string(self, format_str: str, args: str = "") -> Tuple[int, List[int]]:
        """
        Analyze format string and extract referenced indices
        Returns: (max_index, list_of_indices)
        """
        # Find all {N} patterns
        indices = [int(m.group(1)) for m in re.finditer(r'\{(\d+)\}', format_str)]
        
        # Count actual arguments
        if not args:
            arg_count = 0
        else:
            # Simple count: split by comma, but handle nested structures
            # This is approximate - real parsing would be more complex
            arg_count = len([a.strip() for a in args.split(',') if a.strip()])
        
        return (max(indices) if indices else -1, indices, arg_count)
    
    def fix_file(self, file_path: Path) -> int:
        """
        Fix format string errors in a single file
        Returns number of fixes applied
        """
        try:
            content = file_path.read_text(encoding='utf-8')
        except Exception as e:
            print(f"  ❌ Cannot read {file_path}: {e}")
            return 0
        
        original_content = content
        fixes = 0
        
        # Strategy 1: Remove indexed arguments that reference nothing
        # Pattern: {0} with no arguments → {} (empty placeholder)
        pattern1 = re.compile(
            r'(format|println|eprintln|writeln?|write)\!\(\s*"([^"]*\{(\d+)[^"]*)"(?:\s*(?:,\s*(&?\w+))?)\s*\)',
            re.MULTILINE
        )
        
        def replace_indexed_with_empty(match):
            nonlocal fixes
            macro_name = match.group(1)
            format_str = match.group(2)
            indexed = match.group(3)
            args = match.group(4) if match.group(4) else ""
            
            # Replace {N} with {} if no corresponding args
            new_format = re.sub(r'\{\d+\}', '{}', format_str)
            
            # Build new macro call
            if args:
                result = f'{macro_name}!("{new_format}", {args})'
            else:
                result = f'{macro_name}!("{new_format}")'
            
            fixes += 1
            return result
        
        content = pattern1.sub(replace_indexed_with_empty, content)
        
        # Strategy 2: Handle format!("text {0}", value) → format!("text {}", value)
        pattern2 = re.compile(
            r'(format|println|eprintln)\!\("([^"]*)\{(\d+)\}([^"]*)"([^)]*)\)',
            re.MULTILINE
        )
        
        def replace_indexed_format(match):
            nonlocal fixes
            macro = match.group(1)
            before = match.group(2)
            index = match.group(3)
            after = match.group(4)
            args = match.group(5)
            
            # Check if we have enough args
            arg_list = [a.strip() for a in args.split(',') if a.strip() and a.strip() != '']
            
            if len(arg_list) > 0:
                # We have args, so just remove the index
                new_format = f'{before}{{}}{after}'
                fixes += 1
                return f'{macro}!("{new_format}"{args})'
            
            return match.group(0)  # Leave unchanged if no args
        
        content = pattern2.sub(replace_indexed_format, content)
        
        # Strategy 3: writeln!(stream, "{0}") with args
        pattern3 = re.compile(
            r'writeln?\(&?(\w+),\s*"([^"]*\{(\d+)[^"]*)"([^)]*)\)',
            re.MULTILINE
        )
        
        def replace_write_indexed(match):
            nonlocal fixes
            stream = match.group(1)
            format_str = match.group(2)
            index = match.group(3)
            args = match.group(4)
            
            # Replace {N} with {}
            new_format = re.sub(r'\{(\d+)\}', '{}', format_str)
            fixes += 1
            return f'writeln!(&{stream}, "{new_format}"{args})'
        
        content = pattern3.sub(replace_write_indexed, content)
        
        if content != original_content:
            file_path.write_text(content, encoding='utf-8')
            self.files_modified += 1
            return fixes
        
        return 0
    
    def run(self) -> bool:
        """Run the fixer on all Rust files"""
        rust_files = self.find_rust_files()
        
        if not rust_files:
            print(f"❌ No Rust files found in {self.rust_root}")
            return False
        
        print(f"🔍 Found {len(rust_files)} Rust files")
        print(f"🔧 Fixing format string errors...\n")
        
        for file_path in rust_files:
            fixes = self.fix_file(file_path)
            if fixes > 0:
                print(f"  ✅ {file_path.relative_to(self.rust_root)}: {fixes} fixes")
                self.fixes_applied += fixes
        
        print(f"\n{'='*60}")
        print(f"📊 RESULTS")
        print(f"{'='*60}")
        print(f"Files modified:     {self.files_modified}")
        print(f"Total fixes:        {self.fixes_applied}")
        
        if self.fixes_applied > 0:
            print(f"\n✅ Format string errors fixed!")
            print(f"Next: Run 'cargo build' to verify compilation")
            return True
        else:
            print(f"\n⚠️  No format string errors found/fixed")
            return False


if __name__ == '__main__':
    rust_root = sys.argv[1] if len(sys.argv) > 1 else '.'
    
    fixer = FormatStringFixer(rust_root)
    success = fixer.run()
    
    sys.exit(0 if success else 1)
