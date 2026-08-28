# Biofoundry Phase One

**Milestone:** Playable Warren

**Status:** Planned

**Target experience:** one complete 30–45 minute campaign

**Primary platforms:** desktop browser and Windows

**Input standard:** every required action works with visible tap/click controls

## Purpose

Biofoundry already has the breadth of a game, but it still presents like a
tech demo: many systems arrive at once, objectives compete for attention, and
the build has been validated more by deterministic simulation than by new
players. Phase One turns the existing systems into a deliberate first-session
experience before more content is added.

The milestone is not “add another feature.” It is “make the existing campaign
easy to enter, satisfying to master, and possible to finish without outside
instructions.”

## Document set

| Document | Owns |
| --- | --- |
| [`PRODUCT_BRIEF.md`](PRODUCT_BRIEF.md) | Audience, promise, pillars, scope, and success measures |
| [`PLAYABLE_SLICE.md`](PLAYABLE_SLICE.md) | Player journey, rules, pacing, UX, and completion contract |
| [`PRODUCTION_PLAN.md`](PRODUCTION_PLAN.md) | Ordered work, acceptance criteria, dependencies, and exit gate |
| [`PLAYTEST_PLAN.md`](PLAYTEST_PLAN.md) | Test cohorts, sessions, metrics, issue severity, and reporting |

## Source-of-truth rules

- This folder owns Phase One product intent and acceptance criteria.
- `assets/data/*.json` owns live balance values and content definitions.
- Tests own deterministic behavior guarantees.
- `AGENTS.md` and `CODE_STANDARDS.md` own repository working rules.
- `docs/verification/` contains the current canonical screenshots.

When the running game and these documents disagree, record the discrepancy in
the Phase One backlog instead of silently rewriting the intended experience.

## Phase One exit statement

Phase One is complete when a first-time player can launch a fresh warren,
learn the food economy, survive the first major pressure event, automate ore
and ingot production, awaken the Colossal Worm, and return to the menu or
continue in endless play—all without a keyboard, developer guidance, or a
progress-blocking defect.
