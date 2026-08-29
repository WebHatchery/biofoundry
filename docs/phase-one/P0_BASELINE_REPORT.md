# P0 Playable Baseline Report

**Date:** 2026-08-28  
**Build revision:** `b35b311`  
**Builds tested:** packaged Windows and WebGL artifacts from `dist/`  
**Input protocol:** pointer only; no gameplay keyboard shortcuts used

> This report preserves the 2026-08-28 P0 snapshot and its original findings.
> The current candidate has since added primary-pointer/touch camera input,
> visible zoom and pause controls, touch-first copy, the five-beat tutorial,
> and progressive disclosure. Current completion and first-time-player claims
> still belong in the phase-one production and playtest records below.

## Outcome

The current game starts on Windows and WebGL, starts a fresh warren, and
supports the visible Save, Load, Continue, and Menu controls. The deterministic
campaign reaches the Colossal Worm in 39.9 simulated minutes with no
desertions.

A strict touch-equivalent campaign could not be completed. The opening
tutorial requires camera movement, but the current camera supports right-mouse
drag, keyboard pan, and wheel zoom rather than direct primary-pointer drag or a
visible zoom control. The browser layout also pushes the lower HUD below the
fold at a 1280x720 viewport. These are milestone blockers for the Phase One
touch-first contract, so human critical-path beat times and the live completion
flow remain unverified.

## Test matrix

| Platform | Viewport / window | Input | Result |
| --- | --- | --- | --- |
| Windows packaged build | 1280x720 logical game area | left-click pointer | Launch, New Warren, Save, Menu, Continue, and Load passed. Full HUD remained visible. Opening tutorial blocked under a touch-equivalent protocol. |
| WebGL deployed-layout build | 1440x900 browser viewport; 1200x675 canvas | left-click pointer and wheel | Launch, New Warren, Save, Menu, Continue, and Load passed. Full canvas remained visible. Opening tutorial required wheel or non-touch controls. |
| WebGL deployed-layout build | 1280x720 browser viewport; 1200x675 canvas | left-click pointer | Canvas extended from y=103 to y=778. Build & Dig required page scrolling, which hid the top HUD and tutorial. |

The WebGL build was served with the production-relative layout: the game at
`/biofoundry/`, shared runtime assets at `/shared-assets/`, and shared shell
CSS/JavaScript at the site root. Serving only `dist/webgl/` is not a valid
deployment-layout test because the page intentionally references shared files
from its parent.

## Critical-path timing evidence

No first-time-player timing is claimed. The pointer-only observed run stopped
at the opening camera tutorial. The fixed-seed automated campaign provides the
current balance baseline, not a usability result:

| Beat | Automated simulated time |
| --- | ---: |
| Opening food response established | 5 minutes |
| Food reserve near the secured threshold | 10 minutes |
| Ore delivery approaching the secured threshold | 15 minutes |
| Secured warren achieved; Blacksmith active | before 20 minutes |
| Factory goal complete | before 25 minutes |
| Colossal Worm awakened | 39.9 minutes |

The probe finished with 34 ingots forged, five raids survived, and zero
desertions. It passed the existing 30-60 minute automated timing guardrail.

A same-call timing check on the published WebGL preview advanced the HUD clock
from 00:00 to 00:01 during a one-second wait. The packaged Windows build
advanced from 00:18 to 00:19 during the equivalent check. No platform timing
discrepancy was reproduced.

## Flow verification

| Flow | Windows | WebGL | Evidence / limitation |
| --- | --- | --- | --- |
| New Warren | Pass | Pass | Fresh session opened from the title screen with one left click. |
| Continue availability | Pass | Pass | Disabled with no save; enabled after visible Save. |
| Save | Pass | Pass | `Warren saved.` appeared. |
| Load / Continue | Pass | Pass | Saved campaign timestamp and state returned. |
| Return to menu | Pass | Pass | Visible Menu button returned to the title screen. |
| Campaign completion | Automated only | Automated only | Fixed-seed simulation reaches `worm_awake`; no live pointer campaign reached it. |
| Endless continuation | UI contract only | UI contract only | Completion overlay code exposes `Continue in Endless`; not exercised in a live run. |
| Completion return to menu | UI contract only | UI contract only | Completion overlay code exposes `Return to Menu`; not exercised in a live run. |

## Findings

### BF-P0-001 — Required camera tutorial has no touch-equivalent action

**Severity:** 1 — milestone blocker  
**Platforms:** Windows and WebGL  
**Observed behavior:** Tutorial 1/7 says “Right-drag or WASD to look around;
scroll to zoom.” The standing HUD repeats keyboard and mouse-specific controls.
The camera configuration binds drag to the right mouse button. Primary-pointer
drag does not pan, and there are no visible pan or zoom controls.  
**Expected player experience:** A player can complete the camera tutorial using
a direct touch gesture or a visible control, without a keyboard, right mouse
button, or wheel.  
**Reproduction steps:** Start New Warren; attempt to advance Tutorial 1/7 using
left-button drag and visible controls only.  
**Recovery available:** Mouse wheel, right-drag, WASD, or Skip; none proves the
required touch path.  
**Acceptance test:** On Windows and WebGL, primary-pointer drag pans the world;
touch drag produces the same intent; visible controls or pinch provide zoom;
Tutorial 1/7 names those exact actions and advances without keyboard, secondary
click, or wheel input.

### BF-P0-002 — Browser HUD is split across the fold at 1280x720

**Severity:** 2 — important  
**Platform / viewport:** WebGL, 1280x720  
**Observed behavior:** The 1200x675 canvas begins at y=103 and ends at y=778.
The Build & Dig controls are below the viewport. Scrolling to them hides the
Food Grid, Jobs, top bar, and tutorial card.  
**Expected player experience:** Required controls and the current objective are
simultaneously readable at the supported minimum browser viewport.  
**Reproduction steps:** Open the deployed WebGL page at 1280x720; start New
Warren; inspect the bottom of Build & Dig without entering full screen.  
**Recovery available:** Scroll the outer page or enter full screen.  
**Acceptance test:** At the declared minimum viewport, the entire game canvas
fits without page scrolling, or the in-game layout reflows so every required
control and the current objective remain visible and non-overlapping.

### BF-P0-003 — Player-facing control copy conflicts with the touch-first contract

**Severity:** 2 — important  
**Platforms:** Web page, Windows, and WebGL HUD  
**Observed behavior:** The game page advertises Right Mouse, WASD/Arrows, Mouse
Wheel, and Esc. The in-game top bar includes F5/F9 suffixes and mouse/keyboard
camera instructions as the primary help text.  
**Expected player experience:** Player-facing copy names visible controls or
direct gestures first; shortcuts are optional secondary information.  
**Reproduction steps:** Open the game page and then start New Warren; read the
Controls section, top bar, Save/Load labels, and Tutorial 1/7.  
**Recovery available:** Visible Save, Load, and Menu buttons work, but camera
copy has no touch alternative.  
**Acceptance test:** All player-facing control copy passes a repository search
for keyboard/right-click-only instructions and names the visible or direct-touch
path first.

### BF-P0-004 — Final tutorial wording and completion condition disagree

**Severity:** 2 — important  
**Platforms:** Windows and WebGL  
**Observed behavior:** Tutorial 7/7 is titled “Awaken the Worm” and instructs the
player to forge ingots, raise the shrine, and feed the worm, but its completion
condition is the earlier secured-warren `won` flag.  
**Expected player experience:** A tutorial step completes when the action named
by its title and body is complete, or it clearly says that later objectives
continue outside the tutorial.  
**Reproduction steps:** Reach the secured-warren goal before forging 20 ingots
or awakening the worm; observe Tutorial 7/7 complete.  
**Recovery available:** The HUD continues to show ingot and worm progress.  
**Acceptance test:** The final tutorial either remains active through
`worm_awake` or is renamed and rewritten to match the secured-warren condition.

## Implemented-system classification

| Classification | Implemented systems |
| --- | --- |
| Critical path | Title menu and New Warren; tutorial and campaign goals; Food Grid and hunger/brownout; Goblin Miner, Carrier, Cook, Smith, Guard, and Idle jobs; Mushroom Farm, Cook Pot, Ore Mine, Blacksmith, Stockpile, and Worm Shrine; raw/cooked food, ore, ingots, and Iron Pickaxe; construction delivery, building placement, digging, inspection, job reassignment, crafting queue, raid pressure, manual save/load, secured-warren goal, factory goal, worm offerings/awakening, completion overlay, Continue in Endless, and Return to Menu. |
| Optional depth | Beetle Hauler, Salamander Smelter, Slime Janitor, Bat Courier, Hobgoblin, Overseer, and Engineer; Charcoal Kiln, Smelter Den, Snare Trap, Study Pen, Breeding Pit, and Feeding Trough; charcoal and equipment beyond the Iron Pickaxe; capture, specimens, study, breeding, specialist recruitment, spoilage, waste, morale, crowding, reserve policies, equipment affinity, and advanced workstation/status diagnostics. |
| Post-campaign | Worm Outpost construction, route activation, worm cargo/crew transit, remote storage/capacity, and transit recovery after route failure. |

At the time of this baseline, optional systems were visible in the opening
Build & Dig and recruitment panels even when locked or unaffordable. That is
historical classification evidence; the current candidate progressively
reveals those systems after the secured-warren milestone.

## Automated quality evidence

- `cargo fmt -- --check`: passed.
- `cargo test --all-targets`: passed, 66 tests total across the unit and
  integration binaries.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Rust physical line limit: passed through `tests/code_standards.rs`.
- Fixed-seed campaign probe: passed; worm awakened at 39.9 simulated minutes.

## Remaining P0 work

- After BF-P0-001 and BF-P0-002 are resolved, record a complete fresh
  pointer/touch-only campaign with actual player beat times.
- Exercise worm completion, Keep Playing, and Menu in live Windows and WebGL
  runs rather than relying on deterministic state and UI-contract inspection.
- Run the five-player cohorts in `PLAYTEST_PLAN.md`; this engineering baseline
  does not satisfy first-time-player comprehension targets.
