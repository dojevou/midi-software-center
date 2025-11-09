#!/usr/bin/env python3
"""
Error Parser - Extracts detailed error information from Quantum Analyzer output
Generates a structured CSV for systematic fixing
"""

import re
import json
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import List, Dict, Set
from collections import defaultdict

@dataclass
class ErrorInfo:
    error_id: int
    category: str
    severity: str
    error_type: str
    error_message: str
    file_path: str = ""
    line_number: int = 0
    affected_items: List[str] = None
    
    def __post_init__(self):
        if self.affected_items is None:
            self.affected_items = []

class QuantumErrorParser:
    def __init__(self, errors_file: str):
        self.errors_file = errors_file
        self.errors: List[ErrorInfo] = []
        self.categories: Dict[str, List[ErrorInfo]] = defaultdict(list)
        self.error_patterns = self._compile_patterns()
    
    def _compile_patterns(self) -> Dict[str, re.Pattern]:
        """Compile regex patterns for different error types"""
        return {
            'format_string': re.compile(
                r'invalid reference to positional argument (\d+) \(no arguments were given\)'
            ),
            'missing_type': re.compile(
                r"cannot find type `(\w+)` in module `([^`]+)`"
            ),
            'missing_method': re.compile(
                r"no method named `(\w+)` found for struct `(\w+)`"
            ),
            'unresolved_import': re.compile(
                r"unresolved import `([^`]+)`"
            ),
            'trait_bound': re.compile(
                r"the trait bound `([^`]+)` is not satisfied"
            ),
            'doc_comment': re.compile(
                r'expected outer doc comment'
            ),
            'binary_op': re.compile(
                r"binary operation `([^`]+)` cannot be applied to type `([^`]+)`"
            ),
            'iterator': re.compile(
                r"`([^`]+)` is not an iterator"
            ),
        }
    
    def parse(self) -> Dict[str, any]:
        """Parse the error file and categorize"""
        with open(self.errors_file, 'r') as f:
            content = f.read()
        
        # Extract critical errors
        critical_pattern = r'🔴 CRITICAL \[Build\] Build error\s*\n\s*(.+?)(?=\n\n|🔴|🟡|🟢|$)'
        matches = re.finditer(critical_pattern, content)
        
        error_id = 0
        for match in matches:
            error_message = match.group(1).strip()
            category = self._categorize_error(error_message)
            error_info = ErrorInfo(
                error_id=error_id,
                category=category,
                severity='CRITICAL',
                error_type=self._get_error_type(error_message),
                error_message=error_message,
                affected_items=self._extract_affected_items(error_message)
            )
            self.errors.append(error_info)
            self.categories[category].append(error_info)
            error_id += 1
        
        return self._generate_report()
    
    def _categorize_error(self, message: str) -> str:
        """Categorize error by type"""
        if 'positional argument' in message:
            return 'Format String Errors'
        elif 'cannot find type' in message:
            return 'Missing Type Definitions'
        elif 'cannot find struct' in message or 'cannot find value' in message:
            return 'Missing Type/Value Definitions'
        elif 'unresolved import' in message or 'failed to resolve' in message:
            return 'Unresolved Imports'
        elif 'no method named' in message:
            return 'Missing Repository Methods'
        elif 'no clone' in message or 'cannot be cloned' in message:
            return 'AppState & Clone Issues'
        elif 'the trait bound' in message or 'trait' in message.lower():
            return 'Trait Bound & Type Mismatch'
        elif 'expected outer doc comment' in message or 'doc comment' in message:
            return 'Documentation Comment Errors'
        elif 'iterator' in message or 'Iterator' in message:
            return 'Iterator & Type Conversion'
        else:
            return 'Other/Uncategorized'
    
    def _get_error_type(self, message: str) -> str:
        """Extract specific error type"""
        for error_type, pattern in self.error_patterns.items():
            if pattern.search(message):
                return error_type
        return 'unknown'
    
    def _extract_affected_items(self, message: str) -> List[str]:
        """Extract affected types/methods from error message"""
        items = []
        
        # Extract type names
        type_pattern = r'`(\w+)`'
        items.extend(re.findall(type_pattern, message))
        
        return list(set(items))  # Remove duplicates
    
    def _generate_report(self) -> Dict:
        """Generate summary report"""
        category_summary = {
            cat: len(errors) 
            for cat, errors in self.categories.items()
        }
        
        return {
            'total_errors': len(self.errors),
            'category_summary': category_summary,
            'categories': self.categories,
            'errors': self.errors
        }
    
    def export_csv(self, output_file: str):
        """Export errors as CSV for tracking"""
        import csv
        with open(output_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow([
                'ID', 'Category', 'Error Type', 'Message', 
                'Affected Items', 'Status', 'Priority'
            ])
            for i, error in enumerate(self.errors, 1):
                priority = self._calculate_priority(error.category)
                writer.writerow([
                    i,
                    error.category,
                    error.error_type,
                    error.error_message[:100],
                    '|'.join(error.affected_items),
                    'TODO',
                    priority
                ])
    
    def export_json(self, output_file: str):
        """Export structured error data as JSON"""
        report = self._generate_report()
        
        # Convert categories dict to list for JSON serialization
        categories_list = []
        for category, errors in report['categories'].items():
            categories_list.append({
                'category': category,
                'count': len(errors),
                'errors': [asdict(e) for e in errors]
            })
        
        output = {
            'timestamp': Path(self.errors_file).stat().st_mtime,
            'total_errors': report['total_errors'],
            'summary': report['category_summary'],
            'categories': categories_list
        }
        
        with open(output_file, 'w') as f:
            json.dump(output, f, indent=2, default=str)
    
    def print_summary(self):
        """Print summary to console"""
        print("\n" + "="*60)
        print("QUANTUM ERROR ANALYSIS SUMMARY")
        print("="*60)
        print(f"Total Critical Errors: {len(self.errors)}\n")
        
        print("Errors by Category (Priority Order):")
        print("-" * 60)
        
        priority_order = [
            'Format String Errors',
            'Missing Type Definitions',
            'Unresolved Imports',
            'AppState & Clone Issues',
            'Missing Repository Methods',
            'Trait Bound & Type Mismatch',
            'Iterator & Type Conversion',
            'Documentation Comment Errors',
            'Missing Type/Value Definitions',
            'Other/Uncategorized'
        ]
        
        for priority, category in enumerate(priority_order, 1):
            count = len(self.categories.get(category, []))
            if count > 0:
                print(f"{priority}. {category:.<40} {count:>3} errors")
        
        print("\n" + "="*60)
    
    def _calculate_priority(self, category: str) -> int:
        """Calculate priority (1=highest, 10=lowest)"""
        priorities = {
            'Format String Errors': 1,
            'Missing Type Definitions': 2,
            'Unresolved Imports': 3,
            'AppState & Clone Issues': 4,
            'Missing Repository Methods': 5,
            'Trait Bound & Type Mismatch': 6,
            'Iterator & Type Conversion': 7,
            'Documentation Comment Errors': 8,
            'Missing Type/Value Definitions': 9,
            'Other/Uncategorized': 10
        }
        return priorities.get(category, 10)


if __name__ == '__main__':
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python3 error_parser.py <errors_file> [output_dir]")
        print("\nExample: python3 error_parser.py eroors ./error_reports")
        sys.exit(1)
    
    errors_file = sys.argv[1]
    output_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else Path('error_reports')
    output_dir.mkdir(exist_ok=True)
    
    parser = QuantumErrorParser(errors_file)
    report = parser.parse()
    
    # Export reports
    parser.export_csv(output_dir / 'errors.csv')
    parser.export_json(output_dir / 'errors.json')
    
    # Print summary
    parser.print_summary()
    
    print(f"\n✅ Reports generated in: {output_dir}/")
    print(f"   - errors.csv (for spreadsheet review)")
    print(f"   - errors.json (for programmatic use)")
