#
Prerequisites: `brew install llvm && cargo install cargo-watch`
To run: `cargo watch -x run`, or `cargo watch -q -c -w src/ -x "r -F dev"`
To run on web: `trunk serve --open --no-default-features`
To release: Remove feature `bevy/dynamic_linking`

# notes:
https://docs.rs/petgraph/latest/petgraph/algo/simple_paths/fn.all_simple_paths.html


# Editor pls: https://github.com/jakobhellermann/bevy_editor_pls
The default controls are:

E to toggle the editor
Ctrl+Enter to pause/unpause time
F to focus selected entity
T/R/S to show translate/rotate/scale gizmo
Double click on the menu bar to go fullscreen
Cameras:

2d (Pan/Zoom): any mouse button to pan, scroll to zoom
3d (Free): WASD + Ctrl/Shift + Shift for a speed boost for the free 3d camera
3d (Pan/Orbit): Right click to rotate around focus, Middle mouse button to pan