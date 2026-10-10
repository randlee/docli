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

This planning branch (`plan/phase-d`) does not edit `docs/requirements.md` or `docs/architecture.md`. d.1 is the first execution sprint and writes those texts before d.2. A planning review treats the d.1 specification as the deliverable for that baseline. The absence of `REQ-DOCLI-NET-002` in the current baseline is the d.1 execution job, not a missing planning deliverable.

## Closes

- `ADR-005` (decision record text in `docs/architecture.md`)
- `ARCH-RULE-010`
- Requirement text for `REQ-DOCLI-NET-002` (section 5 body and section 11 row) and the renderer-only wording of `REQ-DOCLI-NET-001`. This sprint does not implement `REQ-DOCLI-NET-002`. d.2 through d.5 close the portions named in those sprint docs.

## Hard Dependencies

- d.0 — `integrate/phase-d` synced from `develop` (this plan already on `develop`).

## Deliverables

- [`docs/requirements.md`](../../requirements.md) header: normative scope is **Phase A, Phase B, and Phase D** unless marked **(later)**. The Phase D adapter paragraph in section 5 is normative for Phase D. `REQ-DOCLI-NET-001` stays **(later)**.
- Each sentence checked by a Required Validation `rg` is one physical line in the target file. Do not hard-wrap inside it. That includes the header phrase `Phase A, Phase B, and Phase D`, the section 5 lead-in `Each language adapter walks its native command definition and emits CliModel JSON. Phase D's .NET adapter does not render. HTML and Markdown bytes for that JSON come from the Rust generator.`, the section 5 sentence `Phase D does not implement this sentence and does not close this id`, the `REQ-DOCLI-NET-002` bullet (including `walks a System.CommandLine.Command`, `schema field set and snake_case names`, `docli generate --input`, `docli model`, and `can shadow the Rust`), the system overview sentence, the section 11 `REQ-DOCLI-NET-001` row, the section 11 `REQ-DOCLI-NET-002` row, and the `ARCH-RULE-010` table row.
- Section 2 phases table gains a Phase D row. Closes cell text, exact: `REQ-DOCLI-NET-002: host FromCommand JSON, the tool's own docli model JSON, and Rust docli generate --input HTML`. Does-not-close cell text, exact: `a .NET HTML or Markdown renderer, a template-engine port, Go, Python, MCP, NuGet publish, and a tool that walks a third-party app`.
- Section 2 Later row names these as still later: `REQ-DOCLI-GO-001`, `REQ-DOCLI-PY-001`, the renderer sentence of `REQ-DOCLI-NET-001`, `REQ-DOCLI-PRODUCT-004`, `REQ-DOCLI-GEN-003`, and `REQ-DOCLI-MD-002`. `REQ-DOCLI-NET-002` is not in that Later cell.
- Section 5 replaces the lead-in `Each language walks its native command definition and emits the neutral model, then renders that model with that language's generator.` with this one physical line: `Each language adapter walks its native command definition and emits CliModel JSON. Phase D's .NET adapter does not render. HTML and Markdown bytes for that JSON come from the Rust generator.` Per-language generators stay on the later `REQ-DOCLI-NET-001`, `REQ-DOCLI-GO-001`, and `REQ-DOCLI-PY-001` bullets. Do not edit the `REQ-DOCLI-GO-001` or `REQ-DOCLI-PY-001` bullets. The replacement `REQ-DOCLI-NET-001` bullet keeps `a .NET generator renders the neutral JSON to the shared HTML and Markdown.`
- Section 5 replaces the single `REQ-DOCLI-NET-001` bullet with these two bullets, each one physical line:
  - `REQ-DOCLI-NET-001` (later): a .NET generator renders the neutral JSON to the shared HTML and Markdown. Phase D does not implement this sentence and does not close this id.
  - `REQ-DOCLI-NET-002`: a library under `dotnet/` walks a `System.CommandLine.Command` and writes the snake_case JSON documented by `REQ-DOCLI-INPUT-001` through `REQ-DOCLI-INPUT-005` and `src/schema.rs` (`CliModel`, `OptionSpec`, `ArgumentSpec`, unknown fields preserved). Same JSON means the schema field set and snake_case names, not byte-identical output with `from_clap` for an arbitrary tree. A host app emits its own tree by calling `Docli.CommandLineAdapter.FromCommand`. The Rust command `docli generate --input <that-json>` writes the HTML. `--markdown` remains a flag on that same Rust command. The dotnet tool whose command name is `docli` emits JSON only for the command tree it owns (`docli model`). It does not walk a third-party app and does not render HTML or Markdown. A global `dotnet tool install` shim named `docli` can shadow the Rust `docli` binary. This phase's tests and proofs invoke the built DLL or a local tool path, not a global install. `REQ-DOCLI-PRODUCT-004` stays later and must reconcile that shim when it is closed.
- Section 11 row for `REQ-DOCLI-NET-001`, one physical line: summary `.NET renderer stays later. Phase D does not close this id.` Text column `section 5`. ADR column `—`.
- Section 11 gains a `REQ-DOCLI-NET-002` row immediately after that row, one physical line: summary `System.CommandLine tree to CliModel JSON; Rust renders`. Text column `section 5`. ADR column `ADR-005`.
- [`docs/architecture.md`](../../architecture.md) system overview, one physical line: `Docli.CommandLineAdapter.FromCommand produces CliModel JSON and does not call html::render or markdown::render.`
- ADR index table in `docs/architecture.md` gains an ADR-005 row: link `[ADR-005](#adr-005--net-emits-climodel-json-rust-renders-html)`, title `.NET emits CliModel JSON; Rust renders HTML`, status Accepted. The body is placed after ADR-004 and before the `## Architectural rules` section.
- `ADR-005` in that file, status Accepted:

```text
### ADR-005 — .NET emits CliModel JSON; Rust renders HTML

**Status**: Accepted
**Requirements**: REQ-DOCLI-NET-002, which uses REQ-DOCLI-INPUT-001 through REQ-DOCLI-INPUT-005. Does not close REQ-DOCLI-NET-001, REQ-DOCLI-PRODUCT-004, or REQ-DOCLI-GEN-003.

**Decision**

- Phase D .NET code writes CliModel JSON. It does not render HTML or Markdown and does not port the template engine.
- The Rust binary remains the HTML and Markdown renderer. `docli generate --input <json>` writes HTML. `--markdown` stays on that command.
- The dotnet tool command name is `docli`. It emits JSON only for its own tree (`docli model`). It does not implement `generate` or `show`, and it does not walk a third-party app. `FromCommand` is how a host app emits its own tree. This ADR is the only normative pin. Only Docli.csproj (dotnet/src/Docli/Docli.csproj) repeats the System.CommandLine package version. Every project file under dotnet/ repeats the TFM. A pin change edits the ADR first.
- JSON object keys are the snake_case field names in `src/schema.rs`. Unknown keys are preserved. Same JSON means that schema field set and those snake_case names. It does not mean byte-identical output with `from_clap` for an arbitrary tree. This phase does not add a live clap dump harness.
- C# types and the serializer are sprint d.2. `Docli.CommandLineAdapter.FromCommand` and its mapping are sprint d.3 (the only signature source). Tool argv is sprint d.4. The sample proof is sprint d.5.
- Package System.CommandLine version 2.0.12 (stable 2.0 line, not a 3.0 preview). TFM net10.0.

**Alternatives**

- A distinct command name such as `dotnet-docli` or `docli-dotnet` would avoid a PATH collision with the Rust binary. Rejected for this phase. `ToolCommandName` stays `docli`, and the root `Command` name stays `docli`.

**Consequences**

- HTML bytes for adapter JSON come from the Rust `docli generate` path (ADR-001, ADR-004).
- A global `dotnet tool install` shim named `docli` can shadow the Rust `docli` binary. This phase's tests and proofs invoke the built DLL or a local tool path, not a global install. `REQ-DOCLI-PRODUCT-004` stays later and must reconcile that shim when it is closed.
- A .NET renderer, a template-engine port, Go, Python, and MCP need a later ADR.
```

- Version and TFM numbers in that decision (`2.0.12`, `net10.0`) stay in the ADR. d.2 through d.5 name the ADR-005 pin and do not repeat those numbers.

- `ARCH-RULE-010` row, one physical line, exactly three columns, no `|` inside any cell. Copy this line into the rules table:

```text
| **ARCH-RULE-010** | ADR-005 | The `dotnet/` tree may serialize `CliModel` JSON and walk `System.CommandLine`. It must not render HTML or Markdown, embed template packs, or define a second renderer. HTML bytes for that JSON come from the Rust `docli generate` path. arch-qa runs the three ARCH-RULE-010 gate commands listed directly under this table. |
```

- Immediately after that table and before `**Evaluation (arch-qa)**`, write the block below. d.5 runs these commands from `docs/architecture.md` and does not keep a second copy. The first command exits 1. The find command prints nothing. The third command exits 1. They do not scan `dotnet/tests`.

````text
### ARCH-RULE-010 gate

```text
rg -n 'html::render|markdown::render|MiniJinja|\.j2|<!DOCTYPE|<html|CLI Reference|Scriban|Fluid|Razor' dotnet/src dotnet/samples
find dotnet -type d -name templates -print
rg -n 'Include=.*\.(html|md)' dotnet/src dotnet/samples -g '*.csproj'
```
````
- arch-qa evaluation range in that file becomes `ARCH-RULE-001`–`010`.
- When d.1 executes, change `.claude/agents/arch-qa.md` so the mandatory baseline reads ADR-001 through ADR-005 and ARCH-RULE-001 through ARCH-RULE-010. The ADR bullet range becomes `ADR-001`–`ADR-005`. The architectural-rules bullet range becomes `ARCH-RULE-001`–`ARCH-RULE-010`. This planning branch does not edit `.claude/agents/arch-qa.md`.

## Out of Scope

- Phase D non-closures: see [README.md](README.md)
- Editing [`project-plan.md`](../project-plan.md) (the planning branch already records Phase D as planned and the d.0–d.5 index). The phase-end docs commit in the README owns the later status flip.
- C# types, the walker, the tool, the sample, or Rust renderer edits (d.2–d.5)
- Skill-tree edits under `.claude/skills/`

## Acceptance Criteria

- The requirements header contains `Phase A, Phase B, and Phase D` on one physical line
- The Phase D row Closes cell is `REQ-DOCLI-NET-002: host FromCommand JSON, the tool's own docli model JSON, and Rust docli generate --input HTML`
- The Phase D row Does-not-close cell is `a .NET HTML or Markdown renderer, a template-engine port, Go, Python, MCP, NuGet publish, and a tool that walks a third-party app`
- The Later row contains `REQ-DOCLI-GO-001`, `REQ-DOCLI-PY-001`, `REQ-DOCLI-NET-001`, `REQ-DOCLI-PRODUCT-004`, `REQ-DOCLI-GEN-003`, and `REQ-DOCLI-MD-002`
- Section 5 lead-in is the one physical line `Each language adapter walks its native command definition and emits CliModel JSON. Phase D's .NET adapter does not render. HTML and Markdown bytes for that JSON come from the Rust generator.` The later `REQ-DOCLI-NET-001`, `REQ-DOCLI-GO-001`, and `REQ-DOCLI-PY-001` bullets still name their generators. The old lead-in `then renders that model with that language's generator` is gone.
- Section 5 contains the sentence `Phase D does not implement this sentence and does not close this id` on `REQ-DOCLI-NET-001`. The `REQ-DOCLI-NET-002` bullet contains `walks a System.CommandLine.Command`, `schema field set and snake_case names`, `docli generate --input`, the tool command name `docli`, `docli model`, and `can shadow the Rust`
- `REQ-DOCLI-NET-001` still shows `(later)`. `REQ-DOCLI-NET-002` does not show `(later)`
- The section 11 summary for `REQ-DOCLI-NET-001` is `.NET renderer stays later. Phase D does not close this id.` and the ADR column stays `—`
- The section 11 row for `REQ-DOCLI-NET-002` has summary `System.CommandLine tree to CliModel JSON; Rust renders` and ADR column `ADR-005`
- The system overview contains `does not call html::render or markdown::render`
- The ADR index contains `[ADR-005](#adr-005` with title `.NET emits CliModel JSON; Rust renders HTML` and status Accepted, and the body sits after ADR-004 and before `## Architectural rules`
- `ADR-005` names sprint d.3 as the only signature source for `FromCommand`, names `REQ-DOCLI-NET-002`, contains `Package System.CommandLine version 2.0.12` and `TFM net10.0`, contains `schema field set and those snake_case names`, and contains `can shadow the Rust \`docli\` binary`. It says this ADR is the only normative pin, that only Docli.csproj (dotnet/src/Docli/Docli.csproj) repeats the System.CommandLine package version, that every project file under dotnet/ repeats the TFM, and that a pin change edits the ADR first.
- The `ARCH-RULE-010` row is one physical line with exactly three columns and no `|` inside a cell. It contains `must not render HTML or Markdown` and `listed directly under this table`. The pipe-containing patterns sit in the `### ARCH-RULE-010 gate` block, not in that row.
- `REQ-DOCLI-NET-002` has no `(later)` on its section 5 bullet or its section 11 row
- The arch-qa evaluation step says `ARCH-RULE-001–010`, and `ARCH-RULE-001–009` does not appear
- `.claude/agents/arch-qa.md` mandatory baseline reads ADR-001 through ADR-005 and ARCH-RULE-001 through ARCH-RULE-010. It does not cap that baseline at ADR-004 or ARCH-RULE-009. The ADR-005 body still sits after ADR-004.

## Required Validation

```text
git diff --check
rg -n "Phase A, Phase B, and Phase D" docs/requirements.md
rg -n "REQ-DOCLI-NET-002: host FromCommand JSON, the tool's own docli model JSON, and Rust docli generate --input HTML" docs/requirements.md
rg -n "a .NET HTML or Markdown renderer, a template-engine port, Go, Python, MCP, NuGet publish, and a tool that walks a third-party app" docs/requirements.md
rg -n "REQ-DOCLI-GO-001.*REQ-DOCLI-PY-001.*REQ-DOCLI-NET-001.*REQ-DOCLI-PRODUCT-004.*REQ-DOCLI-GEN-003.*REQ-DOCLI-MD-002" docs/requirements.md
rg -n "Each language adapter walks its native command definition and emits CliModel JSON. Phase D's .NET adapter does not render. HTML and Markdown bytes for that JSON come from the Rust generator." docs/requirements.md
rg -n "then renders that model with that language's generator" docs/requirements.md
rg -n "the Go generator renders" docs/requirements.md
rg -n "a \.NET generator renders" docs/requirements.md
rg -n "the Python generator renders" docs/requirements.md
rg -n "Phase D does not implement this sentence and does not close this id" docs/requirements.md
rg -n 'walks a `System.CommandLine.Command`' docs/requirements.md
rg -n "docli model" docs/requirements.md
rg -n "REQ-DOCLI-NET-001.*\(later\)" docs/requirements.md
rg -n "\.NET renderer stays later\. Phase D does not close this id\." docs/requirements.md
rg -n "REQ-DOCLI-NET-002" docs/requirements.md
rg -n "schema field set and snake_case names" docs/requirements.md
rg -n "can shadow the Rust" docs/requirements.md
rg -n "does not call html::render or markdown::render" docs/architecture.md
rg -n "\[ADR-005\]\(#adr-005" docs/architecture.md
rg -n "Package System.CommandLine version 2.0.12" docs/architecture.md
rg -n "TFM net10.0" docs/architecture.md
rg -n "This ADR is the only normative pin" docs/architecture.md
rg -n -F "Only Docli.csproj (dotnet/src/Docli/Docli.csproj) repeats the System.CommandLine package version. Every project file under dotnet/ repeats the TFM" docs/architecture.md
rg -n "A pin change edits the ADR first" docs/architecture.md
rg -n "ARCH-RULE-010" docs/architecture.md
rg -n "must not render HTML or Markdown" docs/architecture.md
rg -n "schema field set and those snake_case names" docs/architecture.md
rg -n "can shadow the Rust" docs/architecture.md
rg -n "REQ-DOCLI-NET-002" docs/architecture.md
rg -n "find dotnet -type d -name templates" docs/architecture.md
rg -n '^\| \*\*ARCH-RULE-010\*\* \| ADR-005 \| [^|]+ \|$' docs/architecture.md
rg -n -F 'html::render|markdown::render|MiniJinja' docs/architecture.md
rg -n -F 'Include=.*\.(html|md)' docs/architecture.md
rg -n '^\| \*\*ARCH-RULE-010\*\*.*html::render\|' docs/architecture.md
rg -n '^\| \*\*ARCH-RULE-010\*\*.*Include=' docs/architecture.md
rg -n 'REQ-DOCLI-NET-002.*\(later\)' docs/requirements.md
rg -n "ARCH-RULE-001–010" docs/architecture.md
rg -n "ARCH-RULE-001–009" docs/architecture.md
rg -n 'ADR-001`–`ADR-004' .claude/agents/arch-qa.md
rg -n 'ARCH-RULE-001`–`ARCH-RULE-009' .claude/agents/arch-qa.md
rg -n 'ADR-001`–`ADR-005' .claude/agents/arch-qa.md
rg -n 'ARCH-RULE-001`–`ARCH-RULE-010' .claude/agents/arch-qa.md
rg -n '^\| \[ADR-005\].*Accepted' docs/architecture.md
rg -n '^### ADR-00[45]|^## Architectural rules' docs/architecture.md
rg -n '^\| `REQ-DOCLI-NET-001` .*\| — \|$' docs/requirements.md
rg -n '^\| `REQ-DOCLI-NET-002` .*\| ADR-005 \|$' docs/requirements.md
```

`rg -n "ARCH-RULE-001–009" docs/architecture.md` exits 1 (no matches). `rg -n 'ADR-001`–`ADR-004' .claude/agents/arch-qa.md` exits 1. `rg -n 'ARCH-RULE-001`–`ARCH-RULE-009' .claude/agents/arch-qa.md` exits 1. `rg -n 'ADR-001`–`ADR-005' .claude/agents/arch-qa.md` exits 0. `rg -n 'ARCH-RULE-001`–`ARCH-RULE-010' .claude/agents/arch-qa.md` exits 0. `rg -n '^\| \[ADR-005\].*Accepted' docs/architecture.md` exits 0. `rg -n '^### ADR-00[45]|^## Architectural rules' docs/architecture.md` exits 0, and the matches are `### ADR-004`, then `### ADR-005`, then `## Architectural rules`. `rg -n '^\| `REQ-DOCLI-NET-001` .*\| — \|$' docs/requirements.md` exits 0. `rg -n '^\| `REQ-DOCLI-NET-002` .*\| ADR-005 \|$' docs/requirements.md` exits 0. `rg -n '^\| \*\*ARCH-RULE-010\*\* \| ADR-005 \| [^|]+ \|$' docs/architecture.md` exits 0. `rg -n -F 'html::render|markdown::render|MiniJinja' docs/architecture.md` exits 0. `rg -n -F 'Include=.*\.(html|md)' docs/architecture.md` exits 0. `rg -n '^\| \*\*ARCH-RULE-010\*\*.*html::render\|' docs/architecture.md` exits 1. `rg -n '^\| \*\*ARCH-RULE-010\*\*.*Include=' docs/architecture.md` exits 1. `rg -n 'REQ-DOCLI-NET-002.*\(later\)' docs/requirements.md` exits 1. `rg -n "then renders that model with that language's generator" docs/requirements.md` exits 1. Every other `rg` command exits 0.
