# ADR-0004: SARIF as Standard Output Format

**Status:** Accepted
**Date:** 2026-01-23
**Deciders:** Project maintainers

## Context

The AI Code Audit Agent runs multiple code analysis tools (semgrep, bandit, ruff, trivy, clippy) and needs to:

- Collect results from different tools with different native output formats
- Store results for historical comparison
- Enable filtering, merging, and enrichment of findings
- Integrate with existing developer workflows (GitHub Code Scanning, IDEs)

A standardized output format would simplify processing and enable tool interoperability.

## Decision

Use **SARIF 2.1.0** (Static Analysis Results Interchange Format) as the standard output format for all code analysis results.

Implementation:
- `sarif-tools-server/src/sarif.rs` - Full SARIF 2.1.0 data model
- Tool runners convert native output to SARIF
- `normalize_sarif` enriches results with categories and severity
- `merge_sarif` combines results from multiple tools
- Artifacts stored as `.sarif` files in `.audit/artifacts/<commit>/`

## Consequences

### Positive

- **Industry standard** - OASIS open standard, widely adopted
- **GitHub Code Scanning integration** - Native upload to GitHub security tab
- **IDE support** - VS Code, IntelliJ can consume SARIF directly
- **Rich metadata** - Rule descriptions, help URIs, fingerprints, fixes
- **Tool interoperability** - Mix results from any SARIF-producing tool
- **Historical comparison** - Standardized format enables diff between runs

### Negative

- **Verbose format** - JSON structure has significant overhead
- **Conversion complexity** - Each tool needs a native-to-SARIF converter
- **Schema complexity** - Full SARIF spec is extensive (we use a subset)
- **Large file sizes** - Thousands of findings produce multi-MB files

### Neutral

- **JSON-based** - Human-readable but not optimized for size
- **Version locked** - Using 2.1.0, future versions may add features

## SARIF Structure Used

```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",
  "runs": [{
    "tool": {
      "driver": {
        "name": "tool-name",
        "version": "1.0.0",
        "rules": [/* rule definitions */]
      }
    },
    "results": [/* findings */],
    "invocations": [/* execution metadata */]
  }]
}
```

## Alternatives Considered

| Alternative | Pros | Cons | Why Not |
|-------------|------|------|---------|
| Custom JSON | Full control, minimal | No ecosystem, maintenance burden | Reinventing the wheel |
| CodeClimate | Simple format | Less metadata, smaller ecosystem | SARIF more widely adopted |
| CheckStyle XML | Java ecosystem standard | XML parsing, language-specific | Not polyglot-friendly |
| Native formats | No conversion needed | Can't merge, no standardization | Defeats the purpose |

## Related

- [SARIF Specification](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html)
- [GitHub Code Scanning](https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning)
- `sarif-tools-server/src/sarif.rs` - Implementation
- `.github/workflows/audit-artifacts.yml` - CI integration
