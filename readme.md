# bevy_ldtk_procgen

[![Nightly Build](https://github.com/ezraclintoc/TheAmazingDungeon/actions/workflows/nightly.yaml/badge.svg)](https://github.com/ezraclintoc/TheAmazingDungeon/actions/workflows/nightly.yaml)

A procedural, room-by-room level generator for Bevy and LDtk (`bevy_ecs_ldtk`). Author a
catalog of rooms as LDtk levels, mark their doors, and this plugin randomly connects
them into a non-linear layout at runtime - no hand-authored dungeon required.

Not published as a crate yet - this repo is currently both the plugin's source and a
runnable demo game (`cargo run`) built on top of it. See "Library vs. demo" below.

## Features

- Completely Procedural: generates unique, non-linear layouts dynamically at runtime from a catalog of LDtk rooms.

- Asynchronous Generation: room placement runs in serialized batches on Bevy's `AsyncComputeTaskPool`, so it never blocks the main/render thread, and chains multiple rooms deep per batch instead of one door at a time.

- Spatial Hashing: room-vs-room and room-vs-door collision checks are grid-backed, so placement cost stays close to flat as the dungeon grows instead of degrading with total room count.

- Frame-Rate-Limited Spawning: newly generated rooms are drained into the world a few at a time per frame, so a big batch of new rooms doesn't spike frame time.

- Easy LDtk Content Creation: designing and adding new room templates happens entirely inside the LDtk editor - no code changes needed to add a room.

## Bevy compatibility

| bevy_ldtk_procgen | Bevy | bevy_ecs_ldtk |
|-------------------|------|---------------|
| (unreleased)      | 0.18 | 0.14          |

## Getting Started

Ensure you're on the latest stable Rust toolchain, clone the repository, and run:

```bash
cargo run --release
```

This project uses Bevy system dependencies (udev, alsa, vulkan, wayland, X11) that
aren't always present on a bare host - if `cargo run` fails to find them, this repo is
Nix-managed (`shell.nix` + `.envrc`); run `nix-shell` first, or let direnv pick it up
automatically.

> Note: Nightly binaries are also automatically built and available for download under
> the GitHub Releases tab for testing the bleeding edge.

## Bring your own LDtk file

Point the plugin at your own project instead of the bundled example rooms:

```rust
app.add_plugins(WorldPlugin {
    ldtk_path: "my_dungeon.ldtk".into(),
});
```

Your `.ldtk` file needs to follow a few conventions for the generator to be able to
parse and connect your rooms - these aren't currently validated at load time, so a
mismatch will silently misplace or fail to connect rooms rather than error clearly:

- **One level per room.** Each LDtk level is treated as a single placeable room.
- **Door entities, named exactly `Door`,** on an `Entities` layer, placed flush against
  whichever edge(s) of the level should be connectable. Direction is inferred
  automatically from the door's shape and position: a door exactly 1 tile (16px) wide
  is treated as a side door - West if it sits flush with the level's left edge, East
  otherwise; a door wider than 1 tile is treated as a top/bottom door - North if flush
  with the top edge, South otherwise.
- **Every door must be the same width.** The current door-centering math assumes a
  uniform door size across the whole project - a mixed-width project will misalign.
- **Two level fields, on every level:** `weight` (Float) - relative probability this
  room is chosen when multiple candidates fit a given door; and `room_type` (Enum, one
  of `Spawn` / `Room` / `Hallway`). Exactly one `Spawn`-type level is required - that's
  where generation starts.
- **16px tiles, currently required.** Door and room-size math throughout the generator
  assumes a 16px LDtk grid; this isn't yet configurable (tracked in Future Improvements).

## Controls

- WASD — Move the camera around the world.

- Scroll Wheel — Zoom the camera in and out.

- R — Refresh / Regenerate a completely new map layout instantly.

## Library vs. demo

This repo currently builds as a single executable (`src/main.rs`), not a library - it
can't yet be added as a Cargo dependency by another project. Turning it into a proper
plugin crate (a `src/lib.rs` exposing `WorldPlugin` and friends, with this repo's game
becoming an example under `examples/`) is planned but not done - see Future Improvements.
Until then, using this means cloning the repo and swapping in your own `.ldtk` file as
described above, not `cargo add`-ing it.

## License

No license file is currently present in this repository, so default copyright applies -
others can't legally reuse this code yet even though it's public. Adding an explicit
license (e.g. dual MIT/Apache-2.0, the Rust ecosystem norm) is a prerequisite for this
being usable as a template by anyone else.

## Asset Credits

- Tileset: [Tiny Dungeon](https://kenney.nl/assets/tiny-dungeon) by [Kenney](https://kenney.nl) (`assets/tilemap_packed.png`), licensed [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/).

## Current Status & Known Bugs

### Status

- Just finished Spatial Hashing and multi-room-per-batch branch chaining.

### Known Bugs

- Repeating Patterns: Generated dungeons show visibly repetitive layouts - similar sequences of hallways/rooms recur in a noticeable pattern instead of feeling varied between playthroughs.

- Regenerate Hitch: Pressing R to regenerate freezes for a moment, longer the more rooms are currently placed. `regenerate_on_key` recursively despawns every placed room's entity tree (tiles, colliders, LDtk sub-entities) in a single frame with no throttling - the same class of cost the spawn side already got rate-limited for, but never applied to teardown.

## Future Improvements

While the core generation layout logic is running, here are the planned roadmap items to optimize and expand the system:

- Convert to a library crate: expose `src/world/` as `src/lib.rs`'s public API and move the current game into `examples/`, so this can be used as a Cargo dependency instead of a clone-and-edit template.

- Add a license file.

- Configurable tile size: LDtk grid/tile size is currently hardcoded to 16px throughout the placement math.

- Save & Load System: Implement serialization and deserialization to allow players to save generated dungeon layouts and reload them later.

- Room Culling / Unloading: Despawn or freeze LDtk levels that are too far outside the camera's viewport to optimize GPU memory and collision performance.

- Derive Door Clearance Sizes: Door bounding boxes (the clearance a room reserves beyond each door) are currently hand-picked constants that happen to match the smallest single-door room for that direction in the LDtk project. Derive them from the room catalog instead, so they can't silently drift out of sync if a smaller or larger single-door room is ever added. And make spatial hash query offset be tied to the smallest room.

- Derive Cell size from average or mean room size.

## Tech Stack

- Engine: Bevy Engine
- Level Editor: LDtk (Level Designer Toolkit)
- Crates Used: bevy_ecs_ldtk, rand
