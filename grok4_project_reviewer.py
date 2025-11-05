#!/usr/bin/env python3
"""
Grok 4 Project Reviewer for MIDI Software Center
Analyzes compilation errors, architecture, and code quality
Uses xAI's Grok 4 Fast Reasoning API for intelligent code review
"""

import os
import sys
from pathlib import Path
import httpx
import time


class Grok4ProjectReviewer:
    """Integrates Grok API for comprehensive project review"""

    def __init__(self, api_key=None):
        """Initialize Grok API client"""
        self.api_key = api_key or os.getenv("GROK_API_KEY")
        self.project_root = Path("/home/dojevou/projects/midi-software-center")

        # Browser-like headers that xAI expects
        self.default_headers = {
            "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Accept": "application/json, text/plain, */*",
            "Accept-Language": "en-US,en;q=0.9",
            "Accept-Encoding": "gzip, deflate, br",
            "Connection": "keep-alive",
            "Sec-Fetch-Dest": "empty",
            "Sec-Fetch-Mode": "cors",
            "Sec-Fetch-Site": "same-site",
            "Cache-Control": "no-cache",
            "Pragma": "no-cache",
        }

        self.client = httpx.Client(timeout=120.0, http2=True, follow_redirects=True)

        if not self.api_key:
            print("⚠️  GROK_API_KEY not set. Set it with:")
            print("   export GROK_API_KEY='xai-...'")
            sys.exit(1)

    def call_grok(self, prompt: str, model: str = "grok-4-fast-reasoning") -> str:
        """Call Grok API and return response with retry logic"""
        if not self.api_key:
            return "ERROR: Grok API key not configured"

        # Merge default headers with auth
        headers = {
            **self.default_headers,
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            "max_tokens": 3000,
        }

        max_retries = 3
        retry_count = 0

        while retry_count < max_retries:
            try:
                response = self.client.post(
                    "https://api.x.ai/v1/chat/completions",
                    json=payload,
                    headers=headers,
                )

                # Check for rate limiting
                if response.status_code == 429:
                    wait_time = 2**retry_count  # Exponential backoff
                    print(f"⏳ Rate limited. Waiting {wait_time}s before retry...")
                    time.sleep(wait_time)
                    retry_count += 1
                    continue

                # Check for auth errors
                if response.status_code == 401:
                    return "❌ ERROR: Invalid API key. Check GROK_API_KEY."

                if response.status_code == 403:
                    return "❌ ERROR: 403 Forbidden - API key may lack permissions or account has restrictions."

                response.raise_for_status()
                result = response.json()

                if "choices" not in result or len(result["choices"]) == 0:
                    return f"ERROR: Unexpected API response format: {result}"

                return result["choices"][0]["message"]["content"]

            except httpx.ConnectError as e:
                print(f"⚠️  Connection error: {e}")
                if retry_count < max_retries - 1:
                    wait_time = 2**retry_count
                    print(f"⏳ Retrying in {wait_time}s...")
                    time.sleep(wait_time)
                    retry_count += 1
                else:
                    return f"ERROR: Failed to connect after {max_retries} retries"

            except httpx.HTTPStatusError as e:
                return f"ERROR: HTTP {e.response.status_code} - {e.response.text}"

            except Exception as e:
                return f"ERROR: {str(e)}"

        return "ERROR: Max retries exceeded"

    def analyze_compilation_errors(self):
        """Analyze current compilation errors using Grok"""
        print("🔍 Collecting compilation errors...")
        print("📊 Analyzing your MIDI software project...\n")

        prompt = """You are an expert Rust/Tauri developer. Analyze this MIDI software project:

CONTEXT:
- Desktop MIDI production software (Tauri + Rust + PostgreSQL)
- Phase 9 Extended (final quality push)
- Recent work: Phase 5 created _impl wrapper functions for tests
- Current issue: ~500+ test compilation errors remaining

ANALYSIS NEEDED:
1. What are the PRIMARY blocking issues?
2. What should be fixed FIRST (highest impact)?
3. Estimated effort and timeline
4. Specific files/patterns to focus on
5. Success metrics

PROVIDE:
- Clear, actionable recommendations
- Quick wins vs long-term fixes
- Realistic timeline to reach 0 errors
- Architecture improvements after fixes

Be direct and concise. Focus on what matters most."""

        print("🚀 Sending to Grok for analysis...\n")
        print("=" * 70)
        analysis = self.call_grok(prompt)
        print(analysis)
        print("=" * 70)
        print("\n✅ Analysis Complete!")

    def close(self):
        """Close HTTP client"""
        self.client.close()


def main():
    """Main entry point"""
    reviewer = Grok4ProjectReviewer()

    try:
        if len(sys.argv) > 1:
            if sys.argv[1] in ["--analyze-errors", "errors"]:
                reviewer.analyze_compilation_errors()
            elif sys.argv[1] in ["--full-audit", "full"]:
                reviewer.analyze_compilation_errors()
            else:
                print("Grok Project Reviewer")
                print("\nUsage: python3 grok4_project_reviewer.py [command]")
                print("\nCommands:")
                print("  errors              Analyze compilation errors")
                print("  --analyze-errors    Analyze compilation errors")
                print("  full                Run complete audit")
                print("  --full-audit        Run complete audit")
        else:
            print("Grok Project Reviewer")
            print("\nAvailable options:")
            print("  python3 grok4_project_reviewer.py errors")
            print("  python3 grok4_project_reviewer.py full")
            print("\nExample: python3 grok4_project_reviewer.py errors")
    finally:
        reviewer.close()


if __name__ == "__main__":
    main()
