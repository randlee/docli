# Project plan

Product requirements: [docs/requirements.md](../requirements.md).  
Architecture (ADRs, `ARCH-RULE-*`): [docs/architecture.md](../architecture.md).

| Phase | Integration branch | Plan authority | Status |
|-------|-------------------|----------------|--------|
| A | `integrate/phase-a` | [phase-a/README.md](phase-a/README.md) | Complete (merged to `develop`) |
| B | `integrate/phase-b` | [phase-b/README.md](phase-b/README.md) | Planned — normative baselines, error gate, consumer proof, HTML templates (`templates` subcommand, `--preview` / `--theme`), publish readiness |

Phase B follows [codex-orchestration](../../.claude/skills/codex-orchestration/SKILL.md):
sprint docs are authoritative for `req-qa` deliverable enumeration; run
`/plan-hardening` on Phase B plans before dispatching `cwy`.

Template scope lives in Phase B sprints **b.7–b.10** ([`docs/templates/AGENT-PREVIEW.md`](../templates/AGENT-PREVIEW.md)). The former [phase-c/](phase-c/) index is deprecated; use phase-b only.
