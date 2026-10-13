# Phase D — System.CommandLine JSON, Rust HTML (`integrate/phase-d`)

Sprint docs here are the **authority** for deliverables, acceptance, and validation.
[`docs/requirements.md`](../../requirements.md) and [`docs/architecture.md`](../../architecture.md)
are product baselines. Git workflow: [`docs/plans/project-plan.md`](../project-plan.md).

Planning branch: `plan/phase-d` (PR target `develop`). Execution trunk after the plan merges: `integrate/phase-d`.

This planning branch does not edit [`docs/requirements.md`](../../requirements.md) or [`docs/architecture.md`](../../architecture.md). d.1 is the first execution sprint and writes those texts before d.2. A planning review treats the d.1 specification as the deliverable for that baseline. The absence of `REQ-DOCLI-NET-002` in the current baseline is the d.1 execution job, not a missing planning deliverable.

## Pipeline

1. A host .NET app calls `Docli.CommandLineAdapter.FromCommand` on its own `System.CommandLine` tree. That library method writes `CliModel` JSON whose field set and snake_case names match `src/schema.rs` (`CliModel`, `OptionSpec`, `ArgumentSpec`, unknown fields preserved). "Same JSON" means that schema field set and those names. It does not mean byte-identical output with `from_clap` for an arbitrary tree. This phase does not add a live clap dump harness.
2. The Rust command `docli generate --input <that-json>` writes the HTML. `--markdown` stays on that same Rust command.
3. The dotnet global tool, command name `docli-dotnet`, emits JSON only for the command tree it owns (`docli-dotnet model`) and, with `docli-dotnet project`, for an export host named by `--project`. It is not a way to document an arbitrary user app. It does not render HTML or Markdown. Rust `docli` is the user-facing doc product. `docli generate --dotnet-project` orchestrates `docli-dotnet project` and then renders that JSON.

## Layout

| Path | Sprint | Role |
|------|--------|------|
| `global.json` | d.2 | SDK pinned in `global.json` |
| `dotnet/Docli.sln` | d.2 (tool and sample projects added later) | Solution |
| `dotnet/src/Docli/` | d.2 model and JSON; d.3 walker | Class library |
| `dotnet/src/Docli.Tool/` | d.4 | Console app, `PackAsTool`, command name `docli-dotnet` |
| `dotnet/tests/Docli.Tests/` | d.2–d.6 | JSON parity, adapter, tool, HTML proof, project orchestration |
| `dotnet/samples/Docli.Sample/` | d.5 | Sample command tree for the proof |
| `dotnet/samples/Docli.Sample.Export/` | d.6 | Export host that writes the sample tree as CliModel JSON |

## What Phase D closes

- `REQ-DOCLI-NET-002`: a `System.CommandLine` tree becomes `CliModel` JSON, and Rust `docli generate` renders it. This is an id split of the adapter half of `REQ-DOCLI-NET-001`, not a new feature. Portions: d.2 JSON types and schema field names; d.3 `FromCommand`; d.4 the tool's own `docli-dotnet model` JSON; d.5 the sample proof; d.6 `docli generate --dotnet-project` orchestrating `docli-dotnet project`. d.1 writes the requirement and `ADR-005` text and does not implement the adapter.
- The tool's own `docli-dotnet model` JSON, proven when Rust `docli generate --input` writes `index.html` from that file (d.4). Global tool command name is `docli-dotnet`.
- A sample-library proof: the sample calls `FromCommand`, then Rust `docli generate` writes HTML and still accepts `--markdown` (d.5). d.5 depends on d.4 because it calls `RustDocli.Generate`, which d.4 owns. This proof does not invoke the tool and does not read `docli-dotnet model` output.
- d.1 adds the decision record (`ADR-005`) and `ARCH-RULE-010` for that boundary.

## What Phase D does not close

- A .NET HTML or Markdown renderer
- Porting the template engine
- Go or Python adapters
- An MCP wrapper
- `REQ-DOCLI-PRODUCT-004`
- The renderer sentence of `REQ-DOCLI-NET-001` (that id stays later)
- `REQ-DOCLI-GEN-003`
- NuGet publish (`dotnet nuget push` or a packed feed)
- A tool that walks a third-party app (no assembly loader; `docli-dotnet model` documents only the tool's own tree)
- A .NET CI job. `.github/workflows/ci.yml` stays Rust-only. A .NET CI job is a deliberate later item. The host gate below is the only .NET gate

No sprint Closes section marks those items done. [`phase-c/README.md`](../phase-c/README.md) stays a retired bookmark: there is no Phase C release.

## Sprint index

| Sprint | Doc | Closes |
|--------|-----|--------|
| d.0 | [d0-integrate-branch.md](d0-integrate-branch.md) | Sync `develop` → `integrate/phase-d` |
| d.1 | [d1-normative-baselines.md](d1-normative-baselines.md) | `ADR-005`, `ARCH-RULE-010` |
| d.2 | [d2-cli-model-json.md](d2-cli-model-json.md) | `REQ-DOCLI-NET-002` JSON types and schema field names |
| d.3 | [d3-commandline-walker.md](d3-commandline-walker.md) | `REQ-DOCLI-NET-002` `FromCommand` walker |
| d.4 | [d4-dotnet-tool-json.md](d4-dotnet-tool-json.md) | `REQ-DOCLI-NET-002` tool `docli-dotnet model` JSON |
| d.5 | [d5-sample-rust-html-proof.md](d5-sample-rust-html-proof.md) | `REQ-DOCLI-NET-002` sample tree + Rust HTML proof |
| d.6 | [d6-generate-docli-dotnet.md](d6-generate-docli-dotnet.md) | `REQ-DOCLI-NET-002` Rust `generate --dotnet-project` + `docli-dotnet project` |

Execution stacks use **`/sc-gh-stack`** on trunk **`integrate/phase-d`** after operator **go**.

## Host gate (implementation sprints only)

Not **d.0**. **d.1** is docs only: its Required Validation is the gate (the solution does not exist yet).

Install the .NET SDK that satisfies `global.json` before the host gate. `dotnet --version` satisfies the SDK pinned in `global.json`. Phase D does not add a .NET CI job. A .NET CI job is a deliberate later item. The host gate is the only .NET gate. d.4 tool tests require `-c Release`. A `dotnet test` without `-c Release` is not a valid run of those tests. Each implementation sprint PR (d.2, d.3, d.4, d.5, and d.6) pastes that sprint's host-gate output in the PR description.

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

**d.6** adds Rust orchestration and `docli-dotnet project`. From repo root. After these commands, `git status --porcelain` lists no `dotnet/**/bin` or `dotnet/**/obj` path:

```text
cargo build --bin docli
dotnet build dotnet/Docli.sln -c Release
cargo test --test cli_contract -- --nocapture
dotnet test dotnet/Docli.sln -c Release --filter "ProjectCommandTests|DotnetGenerateOrchestrationTests"
git diff --check
```

## Phase-end QA

After **d.6** merges to `integrate/phase-d`, **quality-mgr** re-runs the d.4, d.5, and d.6 host gates. The d.4 and d.5 gate includes `cargo build --bin docli` before `dotnet test`. The d.6 gate runs `dotnet build dotnet/Docli.sln -c Release` before `cargo test --test cli_contract`, so that contract test does not see a missing or stale Release `Docli.Tool.dll`. A phase-end docs commit then sets the project-plan Phase D status off `Planned` and sets each sprint doc frontmatter off `status: planned`, including d.0. d.1 does not edit `project-plan.md`. d.6 does not change that status. Then merge **`integrate/phase-d` → `develop`**. d.6 does not depend on QA completing first.

A GitHub Actions job that runs `dotnet test` stays deferred. The Rust test job installs the SDK pinned by `global.json` so `generate_dotnet_project_writes_sample_html_when_tool_is_available` can build the Release tool and fail, rather than skip, when that build does not produce a working `project` subcommand. That one contract test is not a substitute for the host gate. Develop merge of the .NET suite is honest only when the d.4, d.5, and d.6 host-gate output is on the phase-end PR.

`docli generate --dotnet-project <csproj>` executes that project: `docli-dotnet project` runs `dotnet build` and `dotnet run` on the csproj, then Rust renders the export host's stdout. The temporary JSON file under the system temp directory is removed after `generate` finishes.

NET-CI minors. M1 and M3 are closed below. M2 is unchanged: still Release-only, with no Debug switch in this change. A separate GitHub Actions job that runs `dotnet test` stays deferred.

- **M1 (closed).** The Release DLL path still uses the `TargetFramework` text from `dotnet/src/Docli.Tool/Docli.Tool.csproj` as one path segment (ADR-005). A value that contains a path separator (`/` or `\`) or `..` is rejected as `DOCLI.IO` before that Release DLL is probed.
- **M2 (unchanged).** `docli-dotnet project` always builds and runs `-c Release`. There is no Debug or `--framework` override in this change. The d.4, d.5, and d.6 host gates stay Release-only.
- **M3 (closed).** PATH lookup is `docli-dotnet.exe` on Windows and `docli-dotnet` elsewhere. After `actions/setup-dotnet`, the Rust test job runs `dotnet --version` and `dotnet --list-sdks`, so the CI log records the SDK patch selected under `global.json` `latestPatch`. That step is not a separate `dotnet test` job.
