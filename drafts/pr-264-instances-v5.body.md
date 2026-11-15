### The problem

In the Tekla model from #264 (218 MB IFC), 38,373 of 39,739 bolts are left out. Each bolt comes in as a new shape in world coordinates, so the `Bolt assembly` family runs out of its 256 forms. The file already shares them: about 3,000 distinct shapes, used through `IfcMappedItem`.

### What it does

1. **Reads each mapped shape once**, in the frame of its `IfcRepresentationMap` (reusing #374's `map_frame`), so identical copies of one shape become one type.
2. **Keeps a component's 3D orientation** in an optional field, read in one place (`Model::component_placement`) and checked in `Model::check`. Upright components don't write it, so existing files don't change. It goes in format 20, which 0.12's unreleased changes share, so there's no new bump.
3. **Continues a full family** in "Name (2)", "(3)" and so on, or names it by size when the file gives one (Tekla's bolt name).
4. **Stores extrusions as exact forms** (profile, plane, from and to), as a family drawn by hand already does, and meshes them only when drawing. Only breps stay meshes.
   - Arcs (`IfcTrimmedCurve`) stay arcs, as a line with a turn, not as segments.
   - One reference plane per direction, with the distance kept in from and to, so a family doesn't gather a plane per copy.
   - Copies are compared at 0.1 mm and 0.001° only to decide that they are the same shape; the type keeps the first copy's exact values.
5. **The plan with every bolt:** `Model::shape_key` holds only the values the family reads, as the 3D cache already does, and a tilted part's heights and outline come from its form's vertices. Drawing the plan alone went from about 8.7 s to 3.4 s on this model, with the same image; with opening the file it now takes 8.7 s, against 14.7 s before.

Saving is main's, from #449: this PR doesn't touch it. Its tests are in `ifc_exact_tests.rs`.

### Numbers on the Tekla model

| | main (a0c57b73) | this PR |
|---|--:|--:|
| Components | 15,919 | 54,497 |
| Bolts that come in | 1,366 | all 39,739 |
| Saved project | 53.1 MB | 117.9 MB |
| Saved, compressed | 5.7 MB | 15.0 MB |
| Reopen | 2.1 s | 5.4 s |
| Plan, with opening | 3.7 s | 8.7 s |

Import and save take 25.3 s and 2.5 GB. The whole suite passes (4,413 tests, with main merged); fmt and clippy are clean.

### What it costs in 3D

With every bolt in, the 3D view has much more to draw. Measured on the same model and machine:

| | main | this PR |
|---|--:|--:|
| Building the scene | 1.2 s | 25.3 s |
| Mesh for the graphics card | 0.16 s, 1.6 M triangles | 4.4 s, 45 M triangles |
| First frame | 0.19 s | 1.4 s |
| Rotating | 9–11 ms a frame | 80 ms a frame (about 12 fps) |
| Picking | 0.6 ms | 24 ms |

The still image is right; while rotating, grey boxes stand in for the beams. The image from the CLI takes 50.6 s and 7.5 GB. I'd rather show this than hide the bolts: the fix belongs in the 3D view, not in the import.

### Left for later

- **A coarse form in 3D**, through the forms' Detail Level (emilianocapasso), following `docs/3d-acceleration-plan.md`, rather than a new level-of-detail scheme.
- **Values that repeat on every copy**, which make most of the larger file. Moving them to #250's type parameters with one value per type would also move Mark ("V1") and FireRating to the type, which 4 tests refuse, so it needs a decision on what a type value means (dansity). It would save little once compressed (15.0 against 15.2 MB in a trial).
- **Standing beams:** 3,152 are still left out (2,712 standing and 440 steeper than 75°; the Eberick rule).
- Voids for plate cuts, other IFC forms, and writing `IfcMappedItem` back in the export.

### Questions for IPedrax

1. **Is an optional 3D orientation on a component acceptable in format 20?**
2. **Could a standing `IfcBeam` come in as a generic component when no column stands at the same place?**
3. **Should type values carry a component's Mark and FireRating, or stay per instance?** (with dansity)
4. **Is the forms' Detail Level the right place for a coarse 3D form?** (with emilianocapasso)
