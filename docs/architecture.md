# docli Architecture

**Status**: Authoritative design baseline for this repository.  
**Applies to**: `docli` implementation, reviews, and QA.  
**Consumers**: `arch-qa` (structural rules and ADRs), `req-qa` (cross-check with
[`requirements.md`](requirements.md)).

Sprint plans under `docs/plans/` add sequencing and deliverables. When a sprint
doc conflicts with this file or `requirements.md`, treat the conflict as a
**blocking** compliance failure until resolved.

## System overview

`docli` is a deterministic build tool: neutral `CliModel` JSON in, HTML and/or
Markdown bytes out. Framework adapters (Rust: `from_clap`) produce the JSON;
renderers produce bytes; `ops` writes files, hashes artifacts, and returns the
version `"1"` envelope defined in [`requirements.md`](requirements.md) section 9.

Docli.CommandLineAdapter.FromCommand produces CliModel JSON and does not call html::render or markdown::render. Rust `docli generate --dotnet-project` runs `docli-dotnet project` to obtain that JSON from an export host and then renders it; the .NET tool does not render.

```text
  CliModel JSON ──► render (html/markdown) ──► bytes
        ▲                    ▲
        │                    │
   from_clap            search_index (HTML only)
        │
   caller / fixture

  argv ──► cli (hidden) ──► ops (generate | show | templates_*) ──► Envelope<T>
                │                    │
                │                    └──► render + filesystem I/O (+ pack resolve)
                └──► stdout envelope (--json) or human lines
```

| Layer | Module(s) | Responsibility |
|-------|-----------|----------------|
| Operations API | `src/ops.rs`, `src/contract.rs` | Stable integration boundary; I/O, hashing, envelopes |
| CLI transport | `src/cli.rs` (`#[doc(hidden)]`) | clap argv → ops; not for other crates |
| Render bytes | `src/render/` | `render(model) -> String`; no I/O, no envelope |
| Template packs | `src/templates/` | Embedded `default` pack and filesystem pack resolve; MiniJinja render |
| Search index | `src/search.rs` | `SearchEntry`, `search_index`, `matching_anchors` |
| Input model | `src/schema.rs` | `CliModel`, `OptionSpec`, `ArgumentSpec` |
| clap adapter | `src/clap_model.rs` | `from_clap` |

A later MCP wrapper calls `ops` directly — `generate`, `show`, and
`templates_list` / `templates_show` / `templates_validate` (no argv re-parse,
no JSON reshaping).

## Architecture Decision Records (ADRs)

Canonical ADRs live in **this file**. Copies under `docs/plans/phase-a/` are
historical sprint artifacts; **do not** treat them as a second source of truth.

| ID | Title | Status |
|----|-------|--------|
| [ADR-001](#adr-001--operations-are-the-library-boundary) | Operations are the library boundary | Accepted |
| [ADR-002](#adr-002--search-index-is-embedded-json) | Search index is embedded JSON | Accepted |
| [ADR-003](#adr-003--cli-contract-is-ai-first-normative) | CLI contract is AI-first normative | Accepted |
| [ADR-004](#adr-004--bundled-template-packs-are-embedded) | Bundled template packs are embedded | Accepted |
| [ADR-005](#adr-005--net-emits-climodel-json-rust-renders-html) | .NET emits CliModel JSON; Rust renders HTML | Accepted |

### ADR-001 — Operations are the library boundary

**Status**: Accepted  
**Requirements**: `REQ-DOCLI-PRODUCT-003`, `REQ-DOCLI-CLI-001`–`007`, `REQ-DOCLI-CLI-011`, `REQ-DOCLI-CLI-012`

**Decision**

- `ops::generate`, `ops::show`, `ops::templates_list`, `ops::templates_show`,
  and `ops::templates_validate` are the **stable Rust API**. Request and
  response structs live in `src/ops.rs`.
- `docli` and `cargo-docli` binaries convert argv through `docli::cli`
  (`#[doc(hidden)]`): that module prints the envelope and calls `ops`. **`cli`
  is not a supported integration surface** for other crates.
- A later MCP wrapper calls `ops` directly. It does not re-parse CLI flags or
  reshape response JSON. The CLI envelope is `serde_json` of the same
  `Envelope<T>` the library returns. Error `code`, `details` shape, and
  `suggested_action` match `REQ-DOCLI-CLI-003`. MCP callers surface `ok: false`
  without flattening or dropping fields.
- `pub mod render` stays public. `html::render` and `markdown::render` remain
  `pub fn render(model: &CliModel) -> String`. That is a **second stable API**:
  document bytes only — no file writes, hashes, or envelopes. Callers needing
  paths, hashes, and errors use `ops`. `ops` calls render.
- **Amended by ADR-004:** `html::render` still takes only `&CliModel` and
  still returns `String`. The HTML body is the embedded `default` pack, not
  an inline string in `src/render/html.rs`.

**Consequences**

- Tests and integrators prefer `ops` or render functions, not `cli`.
- HTML body may evolve; render signatures stay stable unless a new ADR says otherwise.

### ADR-002 — Search index is embedded JSON

**Status**: Accepted  
**Requirements**: `REQ-DOCLI-HTML-005`, `REQ-DOCLI-GEN-001`

**Decision**

- The page keeps `<script id="docli-data" type="application/json">` as the full
  `CliModel`. Tree building and the detail panel read only that tag.
- Search is a second tag, `<script id="docli-search" type="application/json">`,
  holding `SearchEntry` values from `search_index`. The page **must not** rebuild
  the index from `docli-data` alone. Rust tests lock `matching_anchors` without a
  browser; embedded JSON is part of the HTML byte lock under `fixtures/contract/`.
- `SearchEntry`, `search_index`, and `matching_anchors` live in `src/search.rs`
  and are re-exported from `src/lib.rs`. The default HTML pack (ADR-004) calls
  `search_index` in `src/templates/mod.rs` (`render_pack`) and embeds the
  result in `#docli-search`. `src/render/html.rs` delegates to that pack and
  **must not** define `SearchEntry`.
- Later language implementations that match `fixtures/contract/index.html` emit
  the same `docli-search` JSON for the same model (no client-side substitute
  schema).

**Consequences**

- Search behavior changes require updating `search.rs`, fixture bytes, and tests.

### ADR-003 — CLI contract is AI-first normative

**Status**: Accepted  
**Requirements**: `REQ-DOCLI-NORM-001`, `REQ-DOCLI-CLI-001`–`010`

**Decision**

- The `docli` / `cargo docli` machine contract version is the string `"1"`.
  Every success and failure envelope includes `ok`, `data`, and `error`.
  Success sets `ok` true, `data` to the response object, and `error` null.
  Failure sets `ok` false, `data` null, and `error` to the error object
  (`REQ-DOCLI-CLI-002`). `data` and `error` are always present.
- `error.kind` maps to process exit the way sprint b.2 and
  `REQ-DOCLI-CLI-004` specify: `validation` exits `2`, `not_found` exits `3`,
  `dependency` exits `4`, and `internal` exits `1`. `--help` and `--version`
  stay human-readable and exit `0`.
- That machine contract is normatively defined by the mandatory
  **creating-ai-clis** skill (`.claude/skills/creating-ai-clis/`).
- `docs/requirements.md` section 9 assigns stable `REQ-DOCLI-CLI-*` ids to that
  contract for `req-qa` traceability. Implementations **must** keep `--json`
  envelopes, typed actionable errors, and `generate` / `show` readback aligned
  with the skill’s `core-contract.md` and `error-contracts.md`.

**Consequences**

- CLI changes require skill conformance, not only ad hoc clap behavior.
- Wrapper scripts and tests that invoke `docli --json` must surface envelope
  errors (`code`, `message`, `suggested_action`), not discard stdout.
- Kind-to-exit drift is an ADR-003 failure even when the process still exits
  non-zero. `tests/error_contract.rs` is the gate (see Error contract
  verification).

### ADR-004 — Bundled template packs are embedded

**Status**: Accepted

**Requirements**: `REQ-DOCLI-HTML-007`, `REQ-DOCLI-HTML-008`, `REQ-DOCLI-HTML-009`, `REQ-DOCLI-HTML-010`

**Amends**: ADR-001

**Decision**

- Bundled HTML packs are compiled into the `docli` binary with `include_dir`.
  Phase B embeds `default` from `templates/html/default/` and `cli-doc` from
  `templates/html/cli-doc/`. Each pack is `template.toml`, `page.html.j2`,
  `style.css.j2`, and `script.js`. `cargo install` renders those packs from
  the embedded bytes. It does not read `default` or `cli-doc` from disk.
- `templates/html/_skeleton/` is an author starter. It is not a bundled id.
  `templates validate` accepts that directory. `templates list` omits pack ids
  that start with `_` (`REQ-DOCLI-HTML-009`).
- `share/docli/templates` holds **optional extra** packs only. It is not the
  source of `default` or `cli-doc`.
- `html::render(&CliModel) -> String` renders the embedded `default` pack with
  the theme defaults in that pack's `theme_schema`. It does not take a template
  id or a theme override.
- Only `ops::generate` applies a caller-selected template, theme, or preview
  directory, and only `ops::generate` returns the version `"1"` envelope for
  that choice (`REQ-DOCLI-CLI-012`). `html::render` still takes only `&CliModel`.
  `docli templates` lists, shows, and validates packs (`REQ-DOCLI-CLI-011`).
- Failures while rendering an embedded pack (`default` or `cli-doc`) are
  `DOCLI.INTERNAL`.
  A filesystem read of a pack directory is `DOCLI.IO`. The embedded pack does
  not perform that read. A filesystem pack whose manifest or templates do not
  compile is `DOCLI.TEMPLATE_INVALID`. An unknown bundled id, and a pack id
  that is not installed under `share/docli/templates`, is
  `DOCLI.TEMPLATE_NOT_FOUND`.
- The default page root element carries `id="docli-default-pack"`. That marker
  is written only in `templates/html/default/page.html.j2`.
- The `cli-doc` page includes `data-layout="two-column"` and
  `class="option-card"`. Those markers are not in the `default` pack. The page
  still embeds `#docli-data` and `#docli-search` (`REQ-DOCLI-HTML-008`).
- ADR-002 is unchanged: the pack embeds `#docli-data` and `#docli-search`, and
  the search JSON still comes from `search_index`.

**Consequences**

- `src/render/html.rs` does not contain the Phase A CSS or JavaScript bodies.
- `html::render` of `fixtures/contract/model.json` matches
  `fixtures/contract/index.html`, including the pack marker.
- MiniJinja is the pack renderer. Phase B does not add Askama.

### ADR-005 — .NET emits CliModel JSON; Rust renders HTML

**Status**: Accepted
**Requirements**: REQ-DOCLI-NET-002, which uses REQ-DOCLI-INPUT-001 through REQ-DOCLI-INPUT-005. Does not close REQ-DOCLI-NET-001, REQ-DOCLI-PRODUCT-004, or REQ-DOCLI-GEN-003.

**Decision**

- Phase D .NET code writes CliModel JSON. It does not render HTML or Markdown and does not port the template engine.
- The Rust binary remains the HTML and Markdown renderer. `docli generate --input <json>` writes HTML. `docli generate --dotnet-project <csproj>` obtains JSON by running `docli-dotnet project` and then uses that same renderer. `--markdown` stays on that command. There is no .NET renderer.
- The dotnet global tool command name is `docli-dotnet`. `docli-dotnet model` emits JSON for the tool's own tree. `docli-dotnet project` is an allowed JSON-only export-host command: it runs the .NET project named by `--project` and writes that host's CliModel JSON. It does not implement `generate` or `show`, it does not render HTML or Markdown, and it does not walk a third-party app without an export host. `FromCommand` is how a host app emits its own tree. The Rust `docli` binary is the user-facing doc product for `generate` and `show`. This ADR is the only normative pin. Only Docli.csproj (dotnet/src/Docli/Docli.csproj) repeats the System.CommandLine package version. Every project file under dotnet/ repeats the TFM. A pin change edits the ADR first.
- JSON object keys are the snake_case field names in `src/schema.rs`. Unknown keys are preserved. Same JSON means that schema field set and those snake_case names. It does not mean byte-identical output with `from_clap` for an arbitrary tree. This phase does not add a live clap dump harness.
- C# types and the serializer are sprint d.2. `Docli.CommandLineAdapter.FromCommand` and its mapping are sprint d.3 (the only signature source). Tool argv is sprint d.4. The sample proof is sprint d.5.
- Package System.CommandLine version 2.0.12 (stable 2.0 line, not a 3.0 preview). TFM net10.0.

**Alternatives**

- Phase D initially shipped the global tool as `docli`, which collided on PATH with the Rust binary. The tool is now `docli-dotnet` (`ToolCommandName` and root `Command` name).

**Consequences**

- HTML bytes for adapter JSON come from the Rust `docli generate` path (ADR-001, ADR-004).
- Rust render, `docli generate --input`, and `docli generate --dotnet-project` failures stay on the section 9 `DOCLI.*` contract (ADR-003). Export stdout that is empty, not UTF-8, or not CliModel JSON is `DOCLI.INPUT_INVALID`. A `docli-dotnet project` exit 2 whose stderr reports an empty or invalid JSON export is also `DOCLI.INPUT_INVALID`. A missing `docli-dotnet`, a failure to start `docli-dotnet` or `dotnet`, a child that exceeds the project deadline, or any other non-zero `docli-dotnet project` exit is `DOCLI.IO`. This ADR adds no codes.
- Dotnet adapter and tool human stderr and exit semantics are closed in sprint d.4 (`docs/plans/phase-d/d4-dotnet-tool-json.md`). The dotnet tool is not the Rust version `"1"` envelope.
- An exception from `DefaultValueFactory` propagates unchanged from `FromCommand`, per sprint d.3 (`docs/plans/phase-d/d3-commandline-walker.md`).
- This phase's tests and proofs invoke the built DLL or a local tool path, not a global install. `REQ-DOCLI-PRODUCT-004` stays later and must document NuGet install alongside `cargo install docli`.
- A .NET renderer, a template-engine port, Go, Python, and MCP need a later ADR.

## Architectural rules (`arch-qa`)

These rules implement the ADRs above. **Severity: BLOCKING** unless noted.

| Rule | ADR | Check |
|------|-----|--------|
| **ARCH-RULE-001** | ADR-001 | No crate may document or export `docli::cli` as integration API; external callers use `ops` or `render`. |
| **ARCH-RULE-002** | ADR-001 | `src/render/**` must not write files, compute sha256 for artifacts, or construct `Envelope`. |
| **ARCH-RULE-003** | ADR-001 | `src/cli.rs` must delegate to `ops`; no duplicated generate/show/templates logic in binaries. |
| **ARCH-RULE-004** | ADR-001 | `Envelope`, `ErrorBody`, and exit codes live in `src/contract.rs`; ops returns envelopes CLI serializes verbatim. |
| **ARCH-RULE-005** | ADR-002 | `SearchEntry`, `search_index`, `matching_anchors` are defined only in `src/search.rs`. |
| **ARCH-RULE-006** | ADR-002 | The rendered HTML embeds `docli-data` and `docli-search`; search JSON comes from `search_index(model)` in the default pack pipeline (`src/templates/mod.rs` / `render_pack`). `html.rs` must not redefine search types. |
| **ARCH-RULE-007** | ADR-003 | CLI envelope and error shape changes require `REQ-DOCLI-CLI-*` and creating-ai-clis alignment. |
| **ARCH-RULE-008** | ADR-001 | Widening `cli` visibility or moving argv parsing into `ops` requires a new ADR and requirements update. |
| **ARCH-RULE-009** | ADR-004 | Bundled packs are embedded with `include_dir`. `html::render` takes `&CliModel` only and uses the embedded `default` pack. `src/render/**` does not select a caller template or return an envelope. |
| **ARCH-RULE-010** | ADR-005 | The `dotnet/` tree may serialize `CliModel` JSON and walk `System.CommandLine`. It must not render HTML or Markdown, embed template packs, or define a second renderer. HTML bytes for that JSON come from the Rust `docli generate` path. arch-qa runs the three ARCH-RULE-010 gate commands listed directly under this table. |

### ARCH-RULE-010 gate

```text
rg -n 'html::render|markdown::render|MiniJinja|\.j2|<!DOCTYPE|<html|CLI Reference|Scriban|Fluid|Razor' dotnet/src dotnet/samples
find dotnet -type d -name templates -print
rg -n 'Include=.*\.(html|md)' dotnet/src dotnet/samples -g '*.csproj'
```

**Evaluation (arch-qa)**

1. Read this file (ADRs + table above).
2. Read `authoritative_sprint_doc` when provided.
3. Inspect `review_targets` against ARCH-RULE-001–010.
4. Emit findings with `rule`: `ARCH-RULE-00N`, `adr`: `ADR-00N`, file, line,
   remediation.

Relaxing an ADR or ARCH-RULE requires an updated ADR in this file, matching
`REQ-*` changes in `requirements.md`, and explicit approval — not “tests pass”
alone.

## Error contract verification

CLI error shape is part of the architecture boundary (ADR-001, ADR-003).
`tests/error_contract.rs` is the merge gate for every `DOCLI.*` code listed in
`requirements.md` §9. `req-qa` treats a new or changed error code without a
matching test as **Blocking**. Embedded default-pack failures reuse
`DOCLI.INTERNAL`. Filesystem pack reads reuse `DOCLI.IO`. Invalid filesystem
pack content is `DOCLI.TEMPLATE_INVALID` (`REQ-DOCLI-CLI-011`). Unknown template
ids are `DOCLI.TEMPLATE_NOT_FOUND` (`REQ-DOCLI-CLI-012`).
