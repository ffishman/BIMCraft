## Import 3D Model: OBJ and glTF come in as Generic Model families

Furniture, equipment and site objects are often available only as OBJ or glTF (from SketchUp, Blender, manufacturers' sites and the many free model libraries). This adds `import_mesh` (Insert > Import 3D Model in the desktop): it reads an OBJ, with the colours from its MTL, or a glTF 2.0, as `.gltf` with its buffers or as `.glb`. Each object becomes a family with a component placed where it stood, so a room set out in another program comes in as separate pieces that can be moved, scheduled and replaced one by one.

### How it comes in

- **Our own reader** (`mesh_in.rs`), written from the public specifications (Wavefront OBJ/MTL and Khronos glTF 2.0), with no new dependency. It caps what it reads: 256 MB per file, 8 million vertices, 2,000 objects, and the depth and visits of a glTF's node tree.
- **One family per object.** Each OBJ object or group, and each glTF mesh at the place its nodes give it, becomes a Generic Model family with one type, `Imported`, and a form for each material. A node's place can be a translation, rotation and scale, or a matrix. A mirrored node's triangles are turned back so they face outward.
- **Placement.** The family stands on its base, with the centre of its box at its origin. Its component goes where the object was, plus `at`, on the level, with its base height as the offset. So a file's objects keep their places relative to one another, and a component's insertion point sits under its middle, as one placed by hand would.
- **Units and up.** A glTF is in metres with y up. An OBJ's unit is guessed from its size or given with `units` (mm, cm, m, in or ft), and `up` says whether it is y-up.
- **Big objects.** An object with more triangles than one form holds (`MAX_TRIANGLES`) comes in as pieces: `Name 2`, `Name 3` and so on. A family name the project already has gets a number, as in `Box (2)`.
- **Colours.** A material's base colour becomes the colour of a new material. That colour is MTL's `Kd` and `d`, or glTF's `baseColorFactor` converted from linear to sRGB, and its alpha becomes the transparency. A material the project already has keeps its own look.
- **Files beside it.** A buffer or an MTL file is read only from the model's own folder, never through an absolute path or `..`. A data URI is decoded.
- **IFC.** The new components export as `IfcBuildingElementProxy`, like other Generic Models.

### What is left out, and said

The reply's `left_out` lists what the reader cannot use, and the desktop's status line repeats it:

- textures;
- lines and points;
- sparse accessors;
- Draco and meshopt compression. A file that needs these extensions is refused with an error that says so.

A file with nothing to bring in is an error that lists what was left out. Possible follow-ups: an average colour taken from a texture, and an options box for units and up in the desktop.

### Checked

- **Tests.** 12 core tests:
  - the reader on hand-made OBJ, glTF and GLB files: units, y-up, MTL colours, node transforms, a mirrored node, a data URI, splitting into pieces, and refusals;
  - the command: placement and offset, new materials and their transparency, a numbered name, undo and redo, and the IFC proxy count.
- **Desktop test.** A UI test imports a two-object OBJ from Insert and checks that both components are selected, the families are named, the status says what came in, and the step can be undone.
- **Real files.** These came in correctly (glTF sample models, and the classic OBJ test meshes):
  - Box, Duck (texture left out), 2CylinderEngine (67 objects), Buggy (190, colours right), ReciprocatingSaw (113);
  - Sponza, whose one 262k-triangle mesh came in as 3 pieces, with 103 textures left out;
  - Stanford bunny and Utah teapot (OBJ);
  - the sample house exported as GLB, which came back as 33 objects.
- **Checks.** `cargo test --workspace`, clippy, fmt and `cargo i18n check` pass. The desktop's new strings have their Portuguese and Spanish.
