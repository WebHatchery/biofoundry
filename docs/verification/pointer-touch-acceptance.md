# Pointer/touch acceptance — 2026-09-12

The capture harness and settled touch-target audit were run without keyboard
input at the common desktop viewport (1280×720) and a compact viewport
(800×450). Every requested scene completed with no `overlap by` diagnostics.

| Flow | Scenes | Desktop | Compact |
| --- | --- | --- | --- |
| Start | `menu` | [menu](ui_menu.png) | [menu](ui_compact_menu.png) |
| Tutorial | `tutorial_food`, `tutorial_factory`, `tutorial_worm` | [food](ui_tutorial_food.png), [factory](ui_tutorial_factory.png), [worm](ui_tutorial_worm.png) | [food](ui_compact_tutorial_food.png), [factory](ui_compact_tutorial_factory.png), [worm](ui_compact_tutorial_worm.png) |
| Core building | `hud_build` | [build](ui_hud_build.png) | [build](ui_compact_hud_build.png) |
| Route interactions | `endless_route_build`, `endless_routes` | [build](ui_endless_route_build.png), [routes](ui_endless_routes.png) | [build](ui_compact_endless_route_build.png), [routes](ui_compact_endless_routes.png) |
| Save/load recovery | `load_confirm` | [load](ui_load_confirm.png) | [load](ui_compact_load_confirm.png) |

The audit’s settled-target reports show the compact layout fits in the
800-pixel viewport. The desktop reports also identify several legacy HUD
controls drawn at 22–40 pixels, including the build palette and tutorial
chrome. They do not overlap, but they remain below the preferred 44-pixel
pointer target and should be enlarged in a future touch-polish pass.

No keyboard-only instruction was required by the tested flows. The remaining
verification limitation is live browser smoke: the published WebGL package
builds successfully, but this environment’s browser declined permission to
open the local package, so direct in-browser pointer taps could not be
completed here.
