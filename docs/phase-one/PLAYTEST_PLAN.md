# Playtest and Quality Plan

## Why test now

The prototype has deterministic coverage for simulation outcomes, but those
tests cannot tell whether a new player understands the HUD, notices a warning,
or enjoys waiting for an unlock. Phase One uses observed play to decide what to
simplify, reveal, retune, or remove from the first campaign.

## Test cohorts

Use at least five first-time players for the milestone gate:

- two players familiar with colony or factory games;
- two players who play strategy games but not this subgenre;
- one player attracted primarily by the theme or visuals.

At least two sessions must use a browser build and at least one must use a
touch-capable device or a strict pointer-only protocol. Developers and existing
Biofoundry testers do not count toward first-time comprehension targets.

## Session protocol

### Setup

Record build revision, platform, viewport, input device, prior genre experience,
and whether a save already exists. Start from the title screen with a fresh
warren unless the session is specifically testing continuation.

Read only this prompt:

> Play until you believe the campaign is complete or you no longer know how to
> proceed. Please think aloud. I will not explain the controls or strategy, but
> I may ask what you think is happening.

Do not teach, hint, or correct during the measured run unless a software defect
prevents further observation. Record any intervention.

### Checkpoints

At these moments, ask “What are you trying to do next, and why?” without giving
feedback:

1. after entering the warren;
2. when the first food warning appears;
3. after the Mine or Blacksmith is inspected;
4. after the secured-warren goal;
5. when the Worm Shrine becomes available.

Also ask the player to explain one stalled food node and one stalled ore node.

### Wrap-up

Ask:

- What did you believe the main goal was?
- When did the economy first make sense?
- Which warning or panel was hardest to understand?
- Did any wait feel pointless?
- Which decision felt most satisfying?
- What did you expect to happen when the worm awakened?
- Would you start another warren? Why or why not?

## Metrics to record

| Metric | How to record |
| --- | --- |
| Time to start | title screen to active warren |
| Time to first farm | active warren to valid placement |
| Food comprehension | player correctly describes production and upkeep |
| Crisis result | recovered, deserted workers, restarted, or abandoned |
| First correct stall diagnosis | time and node |
| First secured goal | elapsed campaign time |
| Worm Shrine unlock | elapsed campaign time |
| Campaign completion | elapsed time or reason not completed |
| Input violations | any required keyboard/right-click action |
| Facilitator interventions | count and reason |
| Long idle periods | waits over 30 seconds without a meaningful decision |
| Defects | severity, reproduction steps, and recovery |

## Milestone targets

- 5/5 start a new warren without help.
- 4/5 identify food as the central constraint by minute 5.
- 4/5 recover from the taught opening food crisis.
- 4/5 correctly diagnose a food-chain and an ore-chain stall.
- 4/5 reach the secured-warren goal.
- 3/5 complete the campaign in 25–55 minutes.
- 5/5 can perform every required action without a keyboard.
- 0 progress-blocking defects occur in a release-gate session.

## Issue severity

### Severity 0 — release blocker

Crash, corrupted or overwritten save, data loss, campaign cannot complete, or
the packaged build cannot start.

### Severity 1 — milestone blocker

A required action is inaccessible by touch/pointer, a player becomes stuck
without knowing why, the critical path can become non-viable without recovery,
or most players misunderstand a required system.

### Severity 2 — important

Confusing feedback, poor pacing, misleading copy, unreadable layout, or a
repeated frustration that does not fully block progress.

### Severity 3 — polish

Minor visual, audio, wording, or consistency issue with an obvious workaround.

## Finding template

```text
Build:
Platform / viewport / input:
Campaign time:
Severity:
Observed behavior:
Expected player experience:
Reproduction steps:
Player quote or action:
Recovery available:
Suggested acceptance test:
```

Record observation separately from proposed solution. One confusing moment may
have several possible fixes; the evidence should survive if the first fix does
not.

## Automated quality checks

Human sessions complement, rather than replace, the repository checks:

1. `cargo fmt -- --check`
2. `cargo test --all-targets`
3. `cargo clippy --all-targets --all-features -- -D warnings`
4. `publish.ps1` with no parameters
5. canonical capture review for every changed screen
6. physical line count check for every `.rs` file

The no-parameter publisher is the final validation path because it exercises
the actual Windows and WebGL packaging/deployment route.

## Reporting cadence

After each session, add findings before changing the build. After every two
sessions, group repeated symptoms and reprioritize the production plan. Run the
full five-player release gate on one candidate build; do not combine results
from materially different builds to claim the milestone target.
