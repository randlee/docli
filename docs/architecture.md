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

```text
  CliModel JSON ──► render (html/markdown) ──► bytes
        ▲                    ▲
        │                    │
   from_clap            search_index (HTML only)
        │
   caller / fixture

  argv ──► cli (hidden) ──► ops::generate | ops::show ──► Envelope<T>
                │                    │
                │                    └──► render + filesystem I/O
                └──► stdout envelope (--json) or human lines
```

| Layer | Module(s) | Responsibility |
|-------|-----------|----------------|
| Operations API | `src/ops.rs`, `src/contract.rs` | Stable integration boundary; I/O, hashing, envelopes |
| CLI transport | `src/cli.rs` (`#[doc(hidden)]`) | clap argv → ops; not for other crates |
| Render bytes | `src/render/` | `render(model) -> String`; no I/O, no envelope |
| Search index | `src/search.rs` | `SearchEntry`, `search_index`, `matching_anchors` |
| Input model | `src/schema.rs` | `CliModel`, `OptionSpec`, `ArgumentSpec` |
| clap adapter | `src/clap_model.rs` | `from_clap` |

A later MCP wrapper calls `ops::generate` and `ops::show` directly (no argv
re-parse, no JSON reshaping).

## Architecture Decision Records (ADRs)

Canonical ADRs live in **this file**. Copies under `docs/plans/phase-a/` are
historical sprint artifacts; **do not** treat them as a second source of truth.

| ID | Title | Status |
|----|-------|--------|
| [ADR-001](#adr-001--operations-are-the-library-boundary) | Operations are the library boundary | Accepted |
| [ADR-002](#adr-002--search-index-is-embedded-json) | Search index is embedded JSON | Accepted |
| [ADR-003](#adr-003--cli-contract-is-ai-first-normative) | CLI contract is AI-first normative | Accepted |

### ADR-001 — Operations are the library boundary

**Status**: Accepted  
**Requirements**: `REQ-DOCLI-PRODUCT-003`, `REQ-DOCLI-CLI-001`–`007`

**Decision**

- `ops::generate` and `ops::show` are the **stable Rust API**. Request and
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
  and are re-exported from `src/lib.rs`. `src/render/html.rs` calls
  `search_index` and embeds the result; it **must not** define `SearchEntry`.
- Later language implementations that match `fixtures/contract/index.html` emit
  the same `docli-search` JSON for the same model (no client-side substitute
  schema).

**Consequences**

- Search behavior changes require updating `search.rs`, fixture bytes, and tests.

### ADR-003 — CLI contract is AI-first normative

**Status**: Accepted  
**Requirements**: `REQ-DOCLI-NORM-001`, `REQ-DOCLI-CLI-001`–`007`

**Decision**

- The `docli` / `cargo docli` machine contract is normatively defined by the
  mandatory **creating-ai-clis** skill (`.claude/skills/creating-ai-clis/`).
- `docs/requirements.md` section 9 assigns stable `REQ-DOCLI-CLI-*` ids to that
  contract for `req-qa` traceability. Implementations **must** keep `--json`
  envelopes, typed actionable errors, and `generate` / `show` readback aligned
  with the skill’s `core-contract.md` and `error-contracts.md`.

**Consequences**

- CLI changes require skill conformance, not only ad hoc clap behavior.
- Wrapper scripts and tests that invoke `docli --json` must surface envelope
  errors (`code`, `message`, `suggested_action`), not discard stdout.

## Architectural rules (`arch-qa`)

These rules implement the ADRs above. **Severity: BLOCKING** unless noted.

| Rule | ADR | Check |
|------|-----|--------|
| **ARCH-RULE-001** | ADR-001 | No crate may document or export `docli::cli` as integration API; external callers use `ops` or `render`. |
| **ARCH-RULE-002** | ADR-001 | `src/render/**` must not write files, compute sha256 for artifacts, or construct `Envelope`. |
| **ARCH-RULE-003** | ADR-001 | `src/cli.rs` must delegate to `ops`; no duplicated generate/show write logic in binaries. |
| **ARCH-RULE-004** | ADR-001 | `Envelope`, `ErrorBody`, and exit codes live in `src/contract.rs`; ops returns envelopes CLI serializes verbatim. |
| **ARCH-RULE-005** | ADR-002 | `SearchEntry`, `search_index`, `matching_anchors` are defined only in `src/search.rs`. |
| **ARCH-RULE-006** | ADR-002 | `html.rs` embeds `docli-data` and `docli-search`; search JSON comes from `search_index(model)`. |
| **ARCH-RULE-007** | ADR-003 | CLI envelope and error shape changes require `REQ-DOCLI-CLI-*` and creating-ai-clis alignment. |
| **ARCH-RULE-008** | ADR-001 | Widening `cli` visibility or moving argv parsing into `ops` requires a new ADR and requirements update. |

**Evaluation (arch-qa)**

1. Read this file (ADRs + table above).
2. Read `authoritative_sprint_doc` when provided.
3. Inspect `review_targets` against ARCH-RULE-001–008.
4. Emit findings with `rule`: `ARCH-RULE-00N`, `adr`: `ADR-00N`, file, line,
   remediation.

Relaxing an ADR or ARCH-RULE requires an updated ADR in this file, matching
`REQ-*` changes in `requirements.md`, and explicit approval — not “tests pass”
alone.
