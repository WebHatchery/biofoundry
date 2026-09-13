# Test coverage review

The legacy unit suites now live under `tests/unit/` and are compiled through
the public `biofoundry` library seam. The suites remain grouped by contract so
the test names describe the behavior they protect rather than the source file
that happens to implement it.

The five-case target was reviewed for the major gameplay surfaces:

| Surface | Distinct cases retained |
| --- | --- |
| Embedded data and copy validation | load, cross-reference, balance, equipment, copy IDs |
| Simulation and economy | deterministic tick, food, mining, crafting, wildlife, routes |
| Persistence and recovery | valid load, malformed content, migration, backup recovery, non-viable state |
| HUD and legibility | compact layout, status vocabulary, objective recovery, touch sizing, modal ownership |
| Outpost network | route lifecycle, cargo priority, upgrades, contracts, transit failure |

Each surface has at least five independent cases. Similar assertions were kept
when they protect different state transitions, save migrations, or player
instructions; collapsing those into one parameterized test would hide the
regression signal. The full migrated suite is run with:

```powershell
$env:CARGO_TARGET_DIR = "D:\WebHatchery\RustGames\biofoundry\target"
cargo test --all-targets
```
