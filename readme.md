## Procedural Dungeon Generator

A procedural dungeon generation system built with Rust, Bevy, and LDtk (bevy_ecs_ldtk).

### Features

- Completely Procedural: Generates unique, non-linear dungeon layouts dynamically at runtime.

-  Hand-Coded Core: Built completely from scratch with custom layout logic.

-  Easy LDtk Content Creation: Designing and adding new room templates is completely frictionless directly inside the LDtk level editor.

### Controls

- WASD — Move the camera around the world.

- Scroll Wheel — Zoom the camera in and out.

- R — Refresh / Regenerate a completely new map layout instantly.

### Installation

Ensure you are using the latest stable Rust toolchain. Clone the repository and run:

```bash
cargo run --release
```

    Note: Nightly binaries are also automatically built and available for download under the GitHub Releases tab for testing the bleeding edge.

### Current Status & Known Bugs
#### Status

- Async generation is not yet implemented; generation currently runs synchronously on the main thread.

- Spatial hashing/chunking is not yet implemented; collision checks currently use a flat linear search.

#### Known Bugs

- Door Connection Errors: Rooms do not always properly align or connect their doors seamlessly.
 
- Open Door States: Doors remain marked as "open" after a room connects to two or more doors at once. (mostly fixed, but some edge cases)

- Debug Grid Clipping: The debug visualization grid does not scale or cover the entire viewport screen.

### Future Improvements

While the core generation layout logic is running, here are the planned roadmap items to optimize and expand the system:

-  Asynchronous Task Offloading: Move heavy geometry and collision calculations onto Bevy's AsyncComputeTaskPool to prevent frame stuttering.

-  Spatial Hashing Grid: Implement a chunk-based grid system to optimize bounds checking from O(N) down to O(1) relative to total map size.

-  Save & Load System: Implement serialization and deserialization to allow players to save generated dungeon layouts and reload them later.

-  Room Culling / Unloading: Despawn or freeze LDtk levels that are too far outside the camera's viewport to optimize GPU memory and collision performance.

-  Deterministic Seeding: Expose a seed input to WorldRng so specific dungeon layouts can be replicated and shared via a simple string or number.

- Better Collision Prevention: When placing a room that overlaps with a door, make sure you can connect to that door before placing.

### Tech Stack

    Engine: Bevy Engine

    Level Editor: LDtk (Level Designer Toolkit)

    Crates Used: bevy_ecs_ldtk, rand
