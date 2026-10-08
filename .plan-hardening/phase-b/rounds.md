| Round | Step | Reviewer | reviewed_commit | status | blocking | important | minor | findings_hash | Note |
|-------|------|----------|-----------------|--------|----------|-----------|-------|---------------|------|
| 1 | 2 | plan-scope-reviewer (composer) | 3e19ed8 | FAIL | 2 | 12 | 1 | PLAN-SCOPE-001…014 | scope cycle 1 |
| 2 | 2 | plan-scope-reviewer (composer) | 546ea61 | FAIL | 0 | 6 | 0 | PLAN-SCOPE-015… | scope cycle 2; fixes in 546ea61 |
| 3 | 3 | arch-ctm (inline) | 546ea61 | PASS | | | | | sprint-scope hardening |
| 4 | 4 | critical-plan-reviewer (grok) | 546ea61 | FAIL | 6 | 2 | 0 | PLAN-CRIT-001…008 | critical cycle 1 |
| 5 | 4 | remediation | 715ee18 | PASS | | | | | critical fixes committed |
| 6 | 5 | arch-ctm (inline) | 715ee18 | PASS | | | | | consistency hardening |
| 7 | 6 | quality-mgr | 715ee18 | IN-FLIGHT | | | | | plan QA started |
