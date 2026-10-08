# Project plan

Product requirements: [docs/requirements.md](../requirements.md).  
Architecture (ADRs, `ARCH-RULE-*`): [docs/architecture.md](../architecture.md).

| Phase | Integration branch | Plan authority | Status |
|-------|-------------------|----------------|--------|
| A | `integrate/phase-a` | [phase-a/README.md](phase-a/README.md) | Complete (merged to `develop`) |
| B | `integrate/phase-b` | [phase-b/README.md](phase-b/README.md) | In progress on `integrate/phase-b` — normative baselines, error gate, consumer proof, **HTML templates (b.7–b.10)**, publish readiness (b.11) |

Phase B is the **only** active plan after Phase A. HTML templating, preview, and
author/agent docs are **sprints b.7–b.10 inside Phase B**, not a separate phase.
See [phase-c/README.md](phase-c/README.md) if an old link pointed at “Phase C”.

Phase B follows [codex-orchestration](../../.claude/skills/codex-orchestration/SKILL.md):
sprint docs are authoritative for `req-qa` deliverable enumeration; run
`/plan-hardening` on Phase B plans before dispatching work.
