---
id: d.1
title: normative baselines for the JSON boundary
status: planned
branch: feature/phase-d-d1-baselines
worktree: ../docli-worktrees/feature/phase-d-d1-baselines
target: integrate/phase-d
---

# Sprint d.1 — normative baselines for the JSON boundary

## Goal

`docs/requirements.md` and `docs/architecture.md` state that Phase D owns the System.CommandLine adapter JSON and the Rust HTML proof, and that a .NET renderer stays later.

## Closes

- `ADR-005` (decision record text in `docs/architecture.md`)
- `ARCH-RULE-010`

## Hard Dependencies

- d.0 — `integrate/phase-d` synced from `develop` (this plan already on `develop`).

## Deliverables

- [`docs/requirements.md`](../../requirements.md) header: normative scope is **Phase A, Phase B, and Phase D** unless marked **(later)**. The Phase D adapter paragraph in section 5 is normative for Phase D. `REQ-DOCLI-NET-001` stays **(later)**.
- Section 2 phases table gains a Phase D row. Closes cell text, exact: `host FromCommand JSON, the tool's own docli model JSON, and Rust docli generate --input HTML`. Does-not-close cell text, exact: `a .NET HTML or Markdown renderer, a template-engine port, Go, Python, MCP, NuGet publish, and a tool that walks a third-party app`.
- Section 2 Later row names these as still later: `REQ-DOCLI-GO-001`, `REQ-DOCLI-PY-001`, the renderer sentence of `REQ-DOCLI-NET-001`, `REQ-DOCLI-PRODUCT-004`, `REQ-DOCLI-GEN-003`, and `REQ-DOCLI-MD-002`. The Phase D adapter is the Phase D row, not this Later cell.
- Section 5 replaces the single `REQ-DOCLI-NET-001` bullet with these two bullets, in this wording:
  - `REQ-DOCLI-NET-001` (later): a .NET generator renders the neutral JSON to the shared HTML and Markdown. Phase D does not implement this sentence and does not close this id.
  - Phase D adapter (normative, not a close of `REQ-DOCLI-NET-001`): a library under `dotnet/` walks a `System.CommandLine.Command` and writes the snake_case JSON documented by `REQ-DOCLI-INPUT-001` through `REQ-DOCLI-INPUT-005` and `src/schema.rs` (`CliModel`, `OptionSpec`, `ArgumentSpec`, unknown fields preserved). A host app emits its own tree by calling `Docli.CommandLineAdapter.FromCommand`. The Rust command `docli generate --input <that-json>` writes the HTML. `--markdown` remains a flag on that same Rust command. The dotnet tool whose command name is `docli` emits JSON only for the command tree it owns (`docli model`). It does not walk a third-party app and does not render HTML or Markdown.
- Section 11 row for `REQ-DOCLI-NET-001`: summary text is `.NET renderer stays later. Phase D adapter JSON is ADR-005, not a close of this id.` Text column stays `section 5`. ADR column stays `—`.
- [`docs/architecture.md`](../../architecture.md) system overview: one sentence that `Docli.CommandLineAdapter.FromCommand` produces `CliModel` JSON and does not call `html::render` or `markdown::render`.
- ADR index table in `docs/architecture.md` gains an ADR-005 row: link `[ADR-005](#adr-005--net-emits-climodel-json-rust-renders-html)`, title `.NET emits CliModel JSON; Rust renders HTML`, status Accepted. The body is placed after ADR-004 and before the `## Architectural rules` section.
- `ADR-005` in that file, status Accepted:

```text
### ADR-005 — .NET emits CliModel JSON; Rust renders HTML

**Status**: Accepted
**Requirements**: REQ-DOCLI-INPUT-001 through REQ-DOCLI-INPUT-005. Does not close REQ-DOCLI-NET-001, REQ-DOCLI-PRODUCT-004, or REQ-DOCLI-GEN-003.

**Decision**

- Phase D .NET code writes CliModel JSON. It does not render HTML or Markdown and does not port the template engine.
- The Rust binary remains the HTML and Markdown renderer. `docli generate --input <json>` writes HTML. `--markdown` stays on that command.
- The dotnet tool command name is `docli`. It emits JSON only for its own tree (`docli model`). It does not implement `generate` or `show`, and it does not walk a third-party app. `FromCommand` is how a host app emits its own tree. Package and TFM pins in this decision are the only copy of those two values.
- JSON object keys are the snake_case field names in `src/schema.rs`. Unknown keys are preserved.
- C# types and the serializer are sprint d.2. `Docli.CommandLineAdapter.FromCommand` and its mapping are sprint d.3 (the only signature source). Tool argv is sprint d.4. The sample proof is sprint d.5.
- Package System.CommandLine version 2.0.12 (stable 2.0 line, not a 3.0 preview). TFM net10.0.

**Consequences**

- HTML bytes for adapter JSON come from the Rust `docli generate` path (ADR-001, ADR-004).
- A .NET renderer, a template-engine port, Go, Python, and MCP need a later ADR.
```

- `ARCH-RULE-010` row: ADR-005. Check text: The `dotnet/` tree may serialize `CliModel` JSON and walk `System.CommandLine`. It must not render HTML or Markdown, embed template packs, or define a second renderer. HTML bytes for that JSON come from the Rust `docli generate` path.
- arch-qa evaluation range in that file becomes `ARCH-RULE-001`–`010`.

## Out of Scope

- Phase D non-closures: see [README.md](README.md)
- Editing [`project-plan.md`](../project-plan.md) (the planning branch already records Phase D and the d.0–d.5 index)
- C# types, the walker, the tool, the sample, or Rust renderer edits (d.2–d.5)
- Skill-tree edits under `.claude/skills/`

## Acceptance Criteria

- The requirements header contains `Phase A, Phase B, and Phase D`
- The Phase D row Closes cell is `host FromCommand JSON, the tool's own docli model JSON, and Rust docli generate --input HTML`
- The Phase D row Does-not-close cell is `a .NET HTML or Markdown renderer, a template-engine port, Go, Python, MCP, NuGet publish, and a tool that walks a third-party app`
- The Later row contains `REQ-DOCLI-GO-001`, `REQ-DOCLI-PY-001`, `REQ-DOCLI-NET-001`, `REQ-DOCLI-PRODUCT-004`, `REQ-DOCLI-GEN-003`, and `REQ-DOCLI-MD-002`
- Section 5 contains the sentence `Phase D does not implement this sentence and does not close this id` on `REQ-DOCLI-NET-001`. The adapter bullet contains `walks a System.CommandLine.Command`, `docli generate --input`, the tool command name `docli`, and `docli model`
- `REQ-DOCLI-NET-001` still shows `(later)`
- The section 11 summary for that id is `.NET renderer stays later. Phase D adapter JSON is ADR-005, not a close of this id.` and the ADR column stays `—`
- The system overview contains `does not call html::render or markdown::render`
- The ADR index contains `[ADR-005](#adr-005` with title `.NET emits CliModel JSON; Rust renders HTML` and status Accepted, and the body sits after ADR-004 and before `## Architectural rules`
- `ADR-005` names sprint d.3 as the only signature source for `FromCommand` and contains `Package System.CommandLine version 2.0.12` and `TFM net10.0`
- The `ARCH-RULE-010` row contains `must not render HTML or Markdown`
- The arch-qa evaluation step says `ARCH-RULE-001–010`, and `ARCH-RULE-001–009` does not appear

## Required Validation

```text
git diff --check
rg -n "Phase A, Phase B, and Phase D" docs/requirements.md
rg -n "host FromCommand JSON, the tool's own docli model JSON, and Rust docli generate --input HTML" docs/requirements.md
rg -n "a .NET HTML or Markdown renderer, a template-engine port, Go, Python, MCP, NuGet publish, and a tool that walks a third-party app" docs/requirements.md
rg -n "REQ-DOCLI-GO-001.*REQ-DOCLI-PY-001.*REQ-DOCLI-NET-001.*REQ-DOCLI-PRODUCT-004.*REQ-DOCLI-GEN-003.*REQ-DOCLI-MD-002" docs/requirements.md
rg -n "Phase D does not implement this sentence and does not close this id" docs/requirements.md
rg -n 'walks a `System.CommandLine.Command`' docs/requirements.md
rg -n "docli model" docs/requirements.md
rg -n "REQ-DOCLI-NET-001.*\(later\)" docs/requirements.md
rg -n "\.NET renderer stays later\. Phase D adapter JSON is ADR-005, not a close of this id\." docs/requirements.md
rg -n "does not call html::render or markdown::render" docs/architecture.md
rg -n "\[ADR-005\]\(#adr-005" docs/architecture.md
rg -n "Package System.CommandLine version 2.0.12" docs/architecture.md
rg -n "TFM net10.0" docs/architecture.md
rg -n "ARCH-RULE-010" docs/architecture.md
rg -n "must not render HTML or Markdown" docs/architecture.md
rg -n "ARCH-RULE-001–010" docs/architecture.md
rg -n "ARCH-RULE-001–009" docs/architecture.md
```

The last command must exit 1 (no matches). Every other `rg` command must exit 0.
