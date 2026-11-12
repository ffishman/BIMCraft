## What happened

Two public models wouldn't open at all:

- Schependomlaan (65 MB, IFC2x3 from Synchro): `floor type 'Imported - -100mm' thickness must be above zero, got -100`
- the same building exported from Revit (49 MB): `floor type '…220mm 220' thickness must be above zero, got 0`

One slab measured zero or a negative thickness, the type made of it failed `Model::check`, and the whole import was refused.

## What changes (`crates/core/src/import.rs`)

- `made_of` (and `top_down`) return `None` when the thickness they come to is not above zero: a body measured flat, or layers that add up to nothing. Every place that builds a wall, floor, ceiling or roof type from it (7 places) then leaves the element unseen, so it comes in as a generic component of its own IFC class with its body, as any class without an element of BIMCraft's does, and goes back out as that class.
- A slab or roof whose top is shaped but measures no depth is read as a level or sloped one instead (`shaped_top(..).filter(..)`).
- A slab whose largest face turned down lies above its largest face turned up (a screed stepped or recessed underneath) is read across the height it spans, `z1 - z0`, instead of a negative rise.

## Results

| Model | Before | After |
|---|---|---|
| Schependomlaan (65 MB) | refused | 3,015 elements: 272 floors + 6 rafter frames (`IfcSlab` 'sporenkap', three loose timber bodies) as generic slabs; saved again as IFC, all 278 `IfcSlab` come back |
| Same building from Revit (49 MB) | refused | 217 floors, 10 roofs, 100 rooms |
| BuildingBIMModel, Ifc4_Revit_ARC/MEP, Duplex | open | unchanged |

## Tests

- `a_slab_that_measures_no_thickness_comes_in_as_a_generic_slab_and_the_file_opens`: failed before with the same error as the real files.
- The stepped-slab reading is checked on Schependomlaan only; I couldn't make a small synthetic slab that reproduces it. Happy to add one if you have a shape in mind.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
