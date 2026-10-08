# Plan hardening (all repos, all phases)

Same process on **every repository** that uses phase plans under `docs/plans/phase-<N>/`.

| Stage | Branch | Notes |
|-------|--------|--------|
| Draft + edit | **Planning branch** off `develop` (`feature/phase-<N>-plan*`, `plan/phase-<N>`, …) | Sprint docs, skills, template author docs only |
| Harden | **Same planning branch** — `/plan-hardening` | Never on `develop` or `integrate/phase-<N>` |
| Publish plan | Planning branch → **`develop`** | Repeat whenever the plan changes; merge is not a lock |
| Sync | **`develop` → `integrate/phase-<N>`** (sprint x.0) | Repeat after plan fixes during execution |
| Execute | **`/sc-gh-stack`**, trunk **`integrate/phase-<N>`** | Operator **go**; build against **current** sprint docs |
| Close phase | **`integrate/phase-<N>` → `develop`** | Phase-end QA |

Per-phase vars and round tables: `.plan-hardening/phase-<N>/` (see `phase-b/` for Phase B).

Authority: [`docs/plans/project-plan.md`](../docs/plans/project-plan.md).
