## Procedural Dungeon Generator

A procedural dungeon generation system built with Rust, Bevy, and LDtk (bevy_ecs_ldtk).

### Features

- Completely Procedural: Generates unique, non-linear dungeon layouts dynamically at runtime.

- Asynchronous Generation: Room generation runs in serialized batches on Bevy's `AsyncComputeTaskPool`, so it never blocks the main/render thread.

-  Hand-Coded Core: Built completely from scratch with custom layout logic.

-  Easy LDtk Content Creation: Designing and adding new room templates is completely frictionless directly inside the LDtk level editor.

### Controls

- WASD — Move the camera around the world.

- Scroll Wheel — Zoom the camera in and out.

- R — Refresh / Regenerate a completely new map layout instantly.

### Installation

Ensure you are using the latest stable Rust toolchain. Clone the repository and run:
c-unhjp\b
```bash
cargo run --release
```

    Note: Nightly binaries are also automatically built and available for download under the GitHub Releases tab for testing the bleeding edge.

### Current Status & Known Bugs
#### Status

- Spatial hashing/chunking is not yet implemented; collision checks currently use a flat linear search.

#### Known Bugs

- Repeating Patterns: Generated dungeons show visibly repetitive layouts - similar sequences of hallways/rooms recur in a noticeable pattern instead of feeling varied between playthroughs.

### Future Improvements

While the core generation layout logic is running, here are the planned roadmap items to optimize and expand the system:

-  Spatial Hashing Grid: Implement a chunk-based grid system to optimize bounds checking from O(N) down to O(1) relative to total map size.

-  Save & Load System: Implement serialization and deserialization to allow players to save generated dungeon layouts and reload them later.

-  Room Culling / Unloading: Despawn or freeze LDtk levels that are too far outside the camera's viewport to optimize GPU memory and collision performance.

- Derive Door Clearance Sizes: Door bounding boxes (the clearance a room reserves beyond each door) are currently hand-picked constants that happen to match the smallest single-door room for that direction in the LDtk project. Derive them from the room catalog instead, so they can't silently drift out of sync if a smaller or larger single-door room is ever added.

### Tech Stack

    Engine: Bevy Engine

    Level Editor: LDtk (Level Designer Toolkit)

    Crates Used: bevy_ecs_ldtk, rand
