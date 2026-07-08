# TheAmazingDungeon - Spec & Benchmarks

Human-readable spec of the generation algorithm, plus a running performance log.
Updated by `/prep-commit` right before a commit, so each commit's version of this file
is a snapshot - the point is to be able to compare a future version (e.g. once spatial
hashing replaces the current flat linear collision scan) against today's numbers using
plain git history, not a separate benchmark tool.

## Architecture

Rooms are authored in LDtk (`assets/rooms.ldtk`), one level per room, each carrying
`Door` entities on its edges plus `weight` (float) and `room_type`
(`Spawn`/`Room`/`Hallway`) level fields.

Startup gates through `GenerationState`: `AssetLoading -> Indexing -> Ready`. Once the
LDtk asset is loaded, every level is parsed into a `RoomIndex` (a flat `Vec<RoomDef>`
catalog plus a `HashMap<Dir, Vec<usize>>` index keyed by which direction each room has a
door facing). Generation then runs in `Ready`.

Generation itself is async, not per-frame synchronous placement: `spawn_if_idle` runs
only when idle, clones the current `WorldState` plus a draw from the session RNG, and
spawns a batch on `AsyncComputeTaskPool`. `poll_task` polls it each frame; on completion
it spawns each placed room as its own `LdtkWorldBundle` and folds the result back into
the real `WorldState` (open-door bookkeeping: matched doors removed, new doors added).

## Algorithms

### Room indexing - `build_room_index` (`pipeline.rs`)

Pure function: parses raw LDtk levels into `RoomDef`s (size, door positions/directions/
widths, weight, room type), then builds the `by_door_dir` index. Doesn't touch Bevy
`App`/`AssetServer`, so it's directly callable from tests against the real
`assets/rooms.ldtk`.

### Batch generation - `generate_batch` (`pipeline.rs`)

Given the current `WorldState`, a `RoomIndex`, a camera position, a search radius, and
an RNG:

1. If no rooms exist yet, pick a random `Spawn`-type room and place it at the origin.
2. Otherwise, find open doors within `search_dist` of the camera, sort by distance, and
   for each one: build a weighted candidate list of rooms with a door facing the
   opposite direction, shuffle it, and try each candidate in turn via `try_place_room`
   until one succeeds.

Placements within a batch mutate a local `WorldState` copy immediately, so later doors in
the same batch see earlier placements.

### Spatial indexing - `SpatialHash` (`spatial_hash.rs`)

A uniform grid: `cells: HashMap<(i32,i32), Vec<usize>>` keyed by cell coordinate,
storing indices into `WorldState.rooms`. `insert`/`query` both floor-divide a rect's
four bounds by `cell_size` and enumerate every cell the rect overlaps (multi-cell for
rects larger than one cell); `query` dedupes via a `HashSet` before returning. Scoped to
`rooms` only (append-only, so indices stay valid) - `open_doors` (which uses
`swap_remove`, invalidating indices) stays a linear scan. `WorldState::add_room` is the
one place that pushes to `rooms`, inserts into the grid, and does open-door bookkeeping
together, so the grid can't drift out of sync with the Vec.

### Placement validation - `try_place_room` (`pipeline.rs`)

Returns `Option<Vec<Room>>` (a placement plus any bridging rooms it required). Checks, in
order:

1. New room's rect vs. `room_grid.query(...)` candidates near the new room's footprint -
   the one unconditional, real visual-safety guarantee (no two rooms' tiles are ever
   drawn overlapping).
2. New room's door clearance boxes vs. `room_grid.query(...)` candidates per door
   (queried separately from step 1, since a door's clearance box can extend past the
   room's own footprint) - allowed only if it lands exactly on an existing door spot, or
   if the door is actively completing a connection to an already-open door.
3. Existing open doors' clearance boxes vs. the new room's whole footprint (linear scan
   over `open_doors`) - closes the "doors overlap unrelated rooms" class of bug; allowed
   only if it's the new room's own matching door.
4. New room's doors vs. other open doors' clearance boxes (linear scan) - a collision
   here triggers `find_bridging_room`, tried recursively (bounded by `MAX_BRIDGE_DEPTH`)
   across every candidate that could simultaneously fill both doors, not just the first
   one found.

### Bridging - `find_bridging_room` (`types.rs`)

Given two colliding open doors, returns every room in the catalog whose placement would
satisfy both simultaneously (not just the first match), so `try_place_room` can fall back
to the next candidate if the first fails its own validation.

### RNG

One `GenRng` resource (a `SmallRng`) is seeded once per session - from `DUNGEON_SEED` if
set in the environment (reproducible debugging), otherwise from OS entropy. Each batch
draws one `u64` from it to seed a fresh task-local `SmallRng` moved into the async
closure (needed because the task must be `Send + 'static`). This replaced reseeding a
constant every batch, which was the cause of a "repeating patterns" bug (every batch's
"random" shuffle resolved the same way for structurally similar situations).

### Known complexity characteristic

Steps 1-2 of `try_place_room` are now grid-backed (see "Spatial indexing" above), so
per-room cost should scale close to flat with total room count rather than the O(n)
per-room / O(n^2) total cost of the old linear scan. Steps 3-4 (`open_doors`) are still
a linear scan, but that set is small (the dungeon's "frontier"), not the whole placed
history. The benchmark history below now shows the actual before/after: per-room cost
climb from 100->1000 rooms dropped from ~6.2x (pre-spatial-hash) to ~1.67x.

Also observed post-spatial-hash: seed-reachability at higher targets dropped a lot
(e.g. 1000 rooms: 22/50 seeds -> 1/50). This looks unrelated to speed - the leading
suspect is a real behavior change in `try_place_room`'s bridging path: the `pretend`
state used to validate a bridge candidate now goes through `WorldState::add_room`
(which also does open-door bookkeeping) instead of a plain `rooms.push`, so
`pretend.open_doors` is no longer identical to the real `open_doors` during recursive
bridge validation like it used to be. Not yet root-caused with certainty - worth
digging into before trusting the "reaches 1000 rooms" claim at face value.

## Current Performance

As of merging `feature/spatial-hashing` into `main` (2026-07-08, merge commit not yet
created - this reflects the staged merge on top of `main`'s `b233949`):

- **100 rooms**: avg 33.13 us/room (30/50 seeds reached target)
- **250 rooms**: avg 38.73 us/room (18/50 seeds reached target)
- **500 rooms**: avg 44.87 us/room (7/50 seeds reached target)
- **1000 rooms**: avg 55.31 us/room (1/50 seeds reached target)

Per-room cost climbs only ~1.67x as the target grows 10x (100 -> 1000 rooms), down from
~6.2x before spatial hashing - the flattening the grid was built to deliver. But
seed-reachability at higher targets dropped sharply (see "Known complexity
characteristic" above) - likely a real bug in bridging validation, not a speed issue,
and worth fixing before leaning on the 1000-room number.

## Benchmark History

Each row is one `/prep-commit` run of `generation_speed_by_target`
(`src/world/tests.rs`): for each target room count, up to 50 different seeds are tried
and only the runs that actually *reach* that target are averaged (a run that stalls
early - the known bridging-fallback gap, a correctness issue, not a speed one - is
excluded rather than averaged in, so that unrelated bug can't masquerade as a speed
change). Cells are `avg us/room (seeds reached/50)`; `N/A` means zero seeds reached that
target this run, which is itself meaningful (worth a Notes callout, not silently dropped).

| Date       | Commit                      | 100 rooms     | 250 rooms     | 500 rooms      | 1000 rooms     | Notes                                                                                                                                                                                                                                                                                                                                                                                                          |
|------------|-----------------------------|---------------|---------------|----------------|----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 2026-07-08 | `12443ae`                   | 38.37 (41/50) | 68.46 (39/50) | 118.93 (35/50) | 236.18 (22/50) | Baseline before spatial hashing. Numbers refreshed on a second run of the same pending (uncommitted) working tree - small movement vs. the first measurement is normal seed noise, not a code change.                                                                                                                                                                                                          |
| 2026-07-08 | `b233949`+`b25bad2` (merge) | 33.13 (30/50) | 38.73 (18/50) | 44.87 (7/50)   | 55.31 (1/50)   | First post-spatial-hashing measurement (merge of feature/spatial-hashing into main, not yet committed). Per-room cost flattened a lot (see Current Performance), but seed-reachability at higher targets dropped sharply - suspected bridging-validation bug (pretend.add_room now runs open-door bookkeeping it didn't before), not a speed regression. Needs follow-up before trusting the 1000-room number. |

### Performance Chart

One combined chart, x-axis = commit history, one line per target - regenerated (one more
x-axis entry, one more point per line) every `/prep-commit` run. Mermaid's `xychart-beta`
has no per-line legend, so this relies on the four targets' natural ordering instead: the
1000-room line sits on top, 100-room on the bottom, in that order, in every run so far.
Watch this once spatial hashing lands: today each line should be roughly flat run-to-run
(same version, same cost), and the *gap* between the top and bottom lines is the number
that should shrink - the four lines converging toward each other is exactly what "cost no
longer depends much on room count" looks like.

```mermaid
xychart-beta
    title "Avg cost by commit (us/room) - top to bottom: 1000, 500, 250, 100 rooms"
    x-axis ["12443ae"]
    y-axis "us/room" 0 --> 300
    line [236.18]
    line [118.93]
    line [68.46]
    line [38.37]
```
