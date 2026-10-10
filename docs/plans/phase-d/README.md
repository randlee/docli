# Phase D — System.CommandLine JSON, Rust HTML (`integrate/phase-d`)

Sprint docs here are the **authority** for deliverables, acceptance, and validation.
[`docs/requirements.md`](../../requirements.md) and [`docs/architecture.md`](../../architecture.md)
are product baselines. Git workflow: [`docs/plans/project-plan.md`](../project-plan.md).

Planning branch: `plan/phase-d` (PR target `develop`). Execution trunk after the plan merges: `integrate/phase-d`.

## Pipeline

1. A host .NET app calls `Docli.CommandLineAdapter.FromCommand` on its own `System.CommandLine` tree. That library method writes the same neutral JSON that `docli::clap_model::from_clap` writes (`CliModel`, `OptionSpec`, `ArgumentSpec` in `src/schema.rs`, snake_case, unknown fields preserved).
2. The Rust command `docli generate --input <that-json>` writes the HTML. `--markdown` stays on that same Rust command.
3. The dotnet tool, command name `docli`, emits JSON only for the command tree it owns (`docli model`). It is not a way to document an arbitrary user app. It does not render HTML or Markdown.

## Layout

| Path | Sprint | Role |
|------|--------|------|
| `global.json` | d.2 | SDK pinned in `global.json` |
| `dotnet/Docli.sln` | d.2 (tool and sample projects added later) | Solution |
| `dotnet/src/Docli/` | d.2 model and JSON; d.3 walker | Class library |
| `dotnet/src/Docli.Tool/` | d.4 | Console app, `PackAsTool`, command name `docli` |
| `dotnet/tests/Docli.Tests/` | d.2–d.5 | JSON parity, adapter, tool, HTML proof |
| `dotnet/samples/Docli.Sample/` | d.5 | Sample command tree for the proof |

## What Phase D closes

- `FromCommand` JSON for a host app's own `System.CommandLine` tree (d.2 types, d.3 walker)
- The tool's own `docli model` JSON, proven when Rust `docli generate --input` writes `index.html` from that file (d.4)
- A sample-library proof: the sample calls `FromCommand`, then Rust `docli generate` writes HTML and still accepts `--markdown` (d.5). d.5 depends on d.4 because it calls `RustDocli.Generate`, which d.4 owns. This proof does not invoke the tool and does not read `docli model` output.
- The decision record for that boundary (d.1, `ADR-005`, `ARCH-RULE-010`)

## What Phase D does not close

- A .NET HTML or Markdown renderer
- Porting the template engine
- Go or Python adapters
- An MCP wrapper
- `REQ-DOCLI-PRODUCT-004`
- The renderer sentence of `REQ-DOCLI-NET-001` (that id stays later)
- `REQ-DOCLI-GEN-003`
- NuGet publish (`dotnet nuget push` or a packed feed)
- A tool that walks a third-party app (no assembly loader; `docli model` documents only the tool's own tree)
- A .NET CI job. `.github/workflows/ci.yml` stays Rust-only. The host gate below is the only .NET gate

No sprint Closes section marks those items done. [`phase-c/README.md`](../phase-c/README.md) stays a retired bookmark: there is no Phase C release.

## Sprint index

| Sprint | Doc | Closes |
|--------|-----|--------|
| d.0 | [d0-integrate-branch.md](d0-integrate-branch.md) | Sync `develop` → `integrate/phase-d` |
| d.1 | [d1-normative-baselines.md](d1-normative-baselines.md) | `ADR-005`, `ARCH-RULE-010` |
| d.2 | [d2-cli-model-json.md](d2-cli-model-json.md) | JSON types for `ADR-005` |
| d.3 | [d3-commandline-walker.md](d3-commandline-walker.md) | `FromCommand` walker |
| d.4 | [d4-dotnet-tool-json.md](d4-dotnet-tool-json.md) | Tool's own `docli model` JSON |
| d.5 | [d5-sample-rust-html-proof.md](d5-sample-rust-html-proof.md) | Sample tree + Rust HTML proof |

Execution stacks use **`/sc-gh-stack`** on trunk **`integrate/phase-d`** after operator **go**.

## Host gate (implementation sprints only)

Not **d.0**. **d.1** is docs only: its Required Validation is the gate (the solution does not exist yet).

Install the .NET SDK that satisfies `global.json` before the host gate. `dotnet --version` satisfies the SDK pinned in `global.json`. Phase D does not add a .NET CI job. The host gate is the only .NET gate.

**d.2** and **d.3**, from repo root. After these commands, `git status --porcelain` lists no `dotnet/**/bin` or `dotnet/**/obj` path:

```text
dotnet test dotnet/Docli.sln -c Release
dotnet build dotnet/Docli.sln -c Release
git diff --check
```

**d.4** and **d.5** invoke the Rust `docli` binary through `Docli.Tests.RustDocli` (owned by d.4). From repo root, `cargo build --bin docli` runs before `dotnet test` so `RustDocli` does not use a stale `target/debug/docli`. After these commands, `git status --porcelain` lists no `dotnet/**/bin` or `dotnet/**/obj` path:

```text
cargo build --bin docli
dotnet test dotnet/Docli.sln -c Release
dotnet build dotnet/Docli.sln -c Release
git diff --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

## Phase-end QA

After **d.5** merges to `integrate/phase-d`, run **quality-mgr** once, then merge **`integrate/phase-d` → `develop`**. d.5 does not depend on QA completing first.
