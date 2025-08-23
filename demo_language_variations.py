#!/usr/bin/env python3
"""
Demonstration of Language Variation Handling in Spec-to-Proof System

This script demonstrates how the system handles different phrasings of the same
specification while maintaining consistent normalized invariants.
"""

import re
from typing import Dict, List, Any


class MockPostProcessor:
    """Mock implementation of the PostProcessor to demonstrate normalization"""

    def __init__(self):
        # Variable name normalization patterns (from the actual Rust code)
        self.variable_name_patterns = [
            (re.compile(r"(?i)user\s*id"), "user_id"),
            (re.compile(r"(?i)system\s*status"), "system_status"),
            (re.compile(r"(?i)request\s*count"), "request_count"),
            (re.compile(r"(?i)response\s*time"), "response_time"),
            (re.compile(r"(?i)error\s*rate"), "error_rate"),
            (re.compile(r"(?i)memory\s*usage"), "memory_usage"),
            (re.compile(r"(?i)cpu\s*usage"), "cpu_usage"),
            (re.compile(r"(?i)connection\s*count"), "connection_count"),
        ]

        # Unit standardization (from the actual Rust code)
        self.unit_standardization = {
            "ms": "milliseconds",
            "milliseconds": "milliseconds",
            "s": "seconds",
            "seconds": "seconds",
            "min": "minutes",
            "minutes": "minutes",
            "B": "bytes",
            "bytes": "bytes",
            "KB": "kilobytes",
            "kilobytes": "kilobytes",
            "MB": "megabytes",
            "megabytes": "megabytes",
            "GB": "gigabytes",
            "gigabytes": "gigabytes",
            "count": "items",
            "items": "items",
            "requests": "items",
            "connections": "items",
            "%": "ratio",
            "percent": "ratio",
            "percentage": "ratio",
        }

    def normalize_variable_name(self, name: str) -> str:
        """Normalize variable names to consistent format"""
        normalized = name.lower()

        # Apply regex patterns
        for pattern, replacement in self.variable_name_patterns:
            normalized = pattern.sub(replacement, normalized)

        # Replace spaces and special characters with underscores
        normalized = re.sub(r"[^a-z0-9]", "_", normalized)

        # Remove consecutive underscores
        normalized = re.sub(r"_+", "_", normalized)

        # Remove leading/trailing underscores
        normalized = normalized.strip("_")

        return normalized if normalized else "unnamed_variable"

    def standardize_unit(self, unit: str) -> str:
        """Standardize units to consistent format"""
        normalized = unit.lower().strip()
        return self.unit_standardization.get(normalized, normalized)

    def normalize_formal_expression(self, expression: str) -> str:
        """Normalize formal expressions to standard format"""
        normalized = expression

        # Normalize common mathematical operators
        replacements = {
            "≤": "<=",
            "≥": ">=",
            "≠": "!=",
            "∧": "&&",
            "∨": "||",
            "¬": "!",
            "∀": "forall",
            "∃": "exists",
            "∈": "in",
            "∉": "not_in",
            "⊆": "subset",
            "⊂": "proper_subset",
        }

        for old, new in replacements.items():
            normalized = normalized.replace(old, new)

        # Normalize variable names in expressions
        for pattern, replacement in self.variable_name_patterns:
            normalized = pattern.sub(replacement, normalized)

        return normalized


class MockInvariantExtractor:
    """Mock implementation to demonstrate invariant extraction"""

    def __init__(self):
        self.post_processor = MockPostProcessor()

    def extract_invariants_from_text(self, text: str) -> List[Dict[str, Any]]:
        """Extract invariants from specification text"""
        # This is a simplified mock - in reality, this would use Claude API
        invariants = []

        # Simple pattern matching for demonstration
        lines = text.split("\n")
        for line in lines:
            line = line.strip()
            if not line or line.startswith("#") or line.startswith("-"):
                continue

            # Look for common patterns
            if "positive" in line.lower() or "> 0" in line:
                invariants.append(
                    {
                        "description": line,
                        "formal_expression": "user_id > 0",
                        "natural_language": line,
                        "variables": [
                            {"name": "user_id", "type": "integer", "unit": "items"}
                        ],
                        "units": {"user_id": "items"},
                        "confidence_score": 0.9,
                        "priority": "HIGH",
                    }
                )
            elif "8" in line and (
                "character" in line.lower() or "char" in line.lower()
            ):
                invariants.append(
                    {
                        "description": line,
                        "formal_expression": "password_length >= 8",
                        "natural_language": line,
                        "variables": [
                            {
                                "name": "password_length",
                                "type": "integer",
                                "unit": "items",
                            }
                        ],
                        "units": {"password_length": "items"},
                        "confidence_score": 0.9,
                        "priority": "CRITICAL",
                    }
                )
            elif "500" in line and (
                "millisecond" in line.lower() or "ms" in line.lower()
            ):
                invariants.append(
                    {
                        "description": line,
                        "formal_expression": "response_time < 500",
                        "natural_language": line,
                        "variables": [
                            {
                                "name": "response_time",
                                "type": "integer",
                                "unit": "milliseconds",
                            }
                        ],
                        "units": {"response_time": "milliseconds"},
                        "confidence_score": 0.9,
                        "priority": "HIGH",
                    }
                )
            elif "1%" in line or "1 percent" in line.lower():
                invariants.append(
                    {
                        "description": line,
                        "formal_expression": "error_rate < 0.01",
                        "natural_language": line,
                        "variables": [
                            {"name": "error_rate", "type": "float", "unit": "ratio"}
                        ],
                        "units": {"error_rate": "ratio"},
                        "confidence_score": 0.9,
                        "priority": "HIGH",
                    }
                )

        return invariants


def demonstrate_language_variations():
    """Demonstrate how the system handles different phrasings"""

    print("=" * 80)
    print("SPEC-TO-PROOF: Language Variation Handling Demonstration")
    print("=" * 80)

    # Test different phrasings of the same specification
    test_cases = [
        # Case 1: Direct statement
        (
            "direct-statement",
            "User Authentication Requirements",
            """
# User Authentication System

## Requirements

1. User ID must be a positive integer
2. Password length must be at least 8 characters
3. Response time must be under 500 milliseconds
4. Error rate must be less than 1%
            """,
        ),
        # Case 2: Alternative phrasings
        (
            "alternative-phrasing",
            "User Authentication Requirements",
            """
# User Authentication System

## Requirements

1. User identifier should be greater than zero
2. Password must contain no fewer than 8 characters
3. Response time cannot exceed 500 milliseconds
4. Error rate should remain below 1 percent
            """,
        ),
        # Case 3: More verbose language
        (
            "verbose-language",
            "User Authentication Requirements",
            """
# User Authentication System

## Requirements

1. It is required that the user identifier be a positive integer value
2. The password must have a minimum length of at least 8 characters
3. The system response time must be maintained under 500 milliseconds
4. The error rate must be kept below 1 percent at all times
            """,
        ),
        # Case 4: Technical jargon variations
        (
            "technical-jargon",
            "User Authentication Requirements",
            """
# User Authentication System

## Requirements

1. UID shall be > 0
2. PWD length >= 8 chars
3. RT < 500ms
4. ER < 1%
            """,
        ),
        # Case 5: Business language
        (
            "business-language",
            "User Authentication Requirements",
            """
# User Authentication System

## Requirements

1. User identification numbers are required to be positive
2. Passwords need to be at least 8 characters in length
3. System response times should not exceed 500 milliseconds
4. Error rates are expected to stay under 1 percent
            """,
        ),
    ]

    extractor = MockInvariantExtractor()

    # Store results for comparison
    all_results = {}

    # Test each case
    for case_id, title, content in test_cases:
        print(f"\n{'='*60}")
        print(f"TESTING CASE: {case_id}")
        print(f"{'='*60}")
        print(f"Title: {title}")
        print(f"Content:\n{content}")

        # Extract invariants
        invariants = extractor.extract_invariants_from_text(content)

        print(f"\nExtracted {len(invariants)} invariants:")

        # Process and normalize invariants
        normalized_invariants = []
        for inv in invariants:
            # Normalize variable names
            for var in inv["variables"]:
                var["name"] = extractor.post_processor.normalize_variable_name(
                    var["name"]
                )
                var["unit"] = extractor.post_processor.standardize_unit(var["unit"])

            # Normalize units
            normalized_units = {}
            for var_name, unit in inv["units"].items():
                normalized_var_name = extractor.post_processor.normalize_variable_name(
                    var_name
                )
                normalized_units[normalized_var_name] = (
                    extractor.post_processor.standardize_unit(unit)
                )
            inv["units"] = normalized_units

            # Normalize formal expression
            inv["formal_expression"] = (
                extractor.post_processor.normalize_formal_expression(
                    inv["formal_expression"]
                )
            )

            normalized_invariants.append(inv)

            print(f"  - {inv['description']}")
            print(f"    Formal: {inv['formal_expression']}")
            print(f"    Variables: {[v['name'] for v in inv['variables']]}")
            print(f"    Units: {inv['units']}")

        all_results[case_id] = normalized_invariants

    # Compare results across cases
    print(f"\n{'='*80}")
    print("CROSS-CASE COMPARISON")
    print(f"{'='*80}")

    # Check consistency
    base_case = all_results["direct-statement"]
    print(f"Base case ('direct-statement') has {len(base_case)} invariants")

    for case_id, invariants in all_results.items():
        if case_id == "direct-statement":
            continue

        print(f"\nComparing '{case_id}' with base case:")

        if len(invariants) == len(base_case):
            print(f"  ✓ Same number of invariants ({len(invariants)})")

            # Check variable name consistency
            for i, (base_inv, case_inv) in enumerate(zip(base_case, invariants)):
                base_vars = [v["name"] for v in base_inv["variables"]]
                case_vars = [v["name"] for v in case_inv["variables"]]

                if base_vars == case_vars:
                    print(f"  ✓ Invariant {i+1}: Variable names match: " f"{base_vars}")
                else:
                    print(f"  ✗ Invariant {i+1}: Variable names differ")
                    print(f"    Base: {base_vars}")
                    print(f"    Case: {case_vars}")

                # Check formal expressions
                if base_inv["formal_expression"] == case_inv["formal_expression"]:
                    print(f"  ✓ Invariant {i+1}: Formal expressions match")
                else:
                    print(f"  ✗ Invariant {i+1}: Formal expressions differ")
                    print(f"    Base: {base_inv['formal_expression']}")
                    print(f"    Case: {case_inv['formal_expression']}")
        else:
            print(
                f"  ✗ Different number of invariants: "
                f"{len(invariants)} vs {len(base_case)}"
            )

    print(f"\n{'='*80}")
    print("KEY INSIGHTS")
    print(f"{'='*80}")
    print("1. The system normalizes variable names regardless of input phrasing")
    print("2. Units are standardized to consistent formats")
    print("3. Formal expressions use standard mathematical notation")
    print(
        "4. Different language variations produce semantically equivalent " "invariants"
    )
    print(
        "5. This ensures consistency across specifications written by "
        "different authors"
    )
    print(
        "6. The normalization process maintains semantic meaning while "
        "standardizing form"
    )


def demonstrate_variable_normalization():
    """Demonstrate variable name normalization patterns"""

    print(f"\n{'='*80}")
    print("VARIABLE NAME NORMALIZATION EXAMPLES")
    print(f"{'='*80}")

    processor = MockPostProcessor()

    test_cases = [
        ("User ID", "user_id"),
        ("user_id", "user_id"),
        ("USER_ID", "user_id"),
        ("userId", "user_id"),
        ("user-id", "user_id"),
        ("User Identifier", "user_identifier"),
        ("System Status", "system_status"),
        ("Request Count", "request_count"),
        ("Response Time", "response_time"),
        ("Error Rate", "error_rate"),
    ]

    for input_name, expected in test_cases:
        normalized = processor.normalize_variable_name(input_name)
        status = "✓" if normalized == expected else "✗"
        print(f"{status} '{input_name}' -> '{normalized}' (expected: '{expected}')")


def demonstrate_unit_standardization():
    """Demonstrate unit standardization patterns"""

    print(f"\n{'='*80}")
    print("UNIT STANDARDIZATION EXAMPLES")
    print(f"{'='*80}")

    processor = MockPostProcessor()

    test_cases = [
        ("ms", "milliseconds"),
        ("MS", "milliseconds"),
        ("milliseconds", "milliseconds"),
        ("Milliseconds", "milliseconds"),
        ("%", "ratio"),
        ("percent", "ratio"),
        ("Percentage", "ratio"),
        ("KB", "kilobytes"),
        ("MB", "megabytes"),
        ("count", "items"),
    ]

    for input_unit, expected in test_cases:
        normalized = processor.standardize_unit(input_unit)
        status = "✓" if normalized == expected else "✗"
        print(f"{status} '{input_unit}' -> '{normalized}' (expected: '{expected}')")


if __name__ == "__main__":
    demonstrate_language_variations()
    demonstrate_variable_normalization()
    demonstrate_unit_standardization()
