## Save as DXF and DWG: a level's plan, in model millimetres, each category on its layer

Two commits: the DXF writer, then DWG on top of it through acadrust. They can be reviewed one commit at a time.

## Part 1: DXF

DXF export is on the README's "Next, in order" list and was the "still to come" of Import CAD. This is its first part: `save` to a path ending `.dxf` writes a level's plan (`view`, the lowest by default, as for .svg and .png).

### What the file holds

- **What the plan shows**, exactly: the view's display list (`render::plan`, the one the PNG and SVG draw), its pixels turned back into model millimetres with `View::mm`. So visibility, overrides, cut/projection, the crop and annotation all come out as the plan draws them, with no second drawing path to keep in step.
- **AutoCAD 2004 (AC1018)**, the oldest format with true colours, with the structure every reader since R2000 expects: header (`$INSUNITS` = mm, extents, handle seed), tables (VPORT, LTYPE, LAYER, STYLE, VIEW, UCS, APPID, DIMSTYLE, BLOCK_RECORD), the model and paper space blocks, the entities, and the root dictionary with the Model and Layout1 layouts. The skeleton follows CADCraft's writer (MIT OR Apache-2.0), cut down to what a view needs; no new dependency.
- **Layers by category**: each element's lines go on its category's layer, or the subcategory it's drawn in (`wall`, `door`, `dooropening`), what the view cuts on `<category>-cut`, patterns on `<category>-pattern`, lettering on `<category>-text` (a text note's on `text`), and what no element draws on `annotation`. This is the layering people expect from Revit's DWG export (category, with cut and projection split) without inventing a layer standard; a mapping table (AIA, NBR 13532, ISO 13567) could come later as data.
- **Lines** are LWPOLYLINEs in their true colour, with the standard line weight nearest their width on paper (`dxf_out::lineweight`) and their dashes as a linetype in model millimetres (`BIM_DASH_n`). **Fills** (the walls' poché) are solid HATCHes, the first ring the outline and the rest holes.

### What is approximate, and what is left out

- Lettering is written as our own font's strokes, as in the PDF, not as TEXT entities: it looks the same everywhere but can't be edited as text. TEXT/MTEXT with a style is a follow-up.
- A picture (an image underlay) can't be written; it is counted in the reply's `left_out`.
- Plans only, as for .svg/.png; sections, elevations and sheets (paper space) can follow the same path.

### Checked

- `dxf_out_tests.rs` reads the file back with our own DXF reader (the one Import CAD uses): unit mm, nothing left out, the layers the reply names, the walls' cut lines exactly where the walls are (to 0.01 mm), the poché as a solid hatch, weights present, and every dash naming a linetype that is defined. A second test checks `view` picks the level.
- The sample house's plan (565 entities, 27 layers) passes ezdxf's audit with no errors and no fixes, and ezdxf's own drawing of it matches our PNG.
- Through acadrust (the DWG bridge already in main), the same file becomes a DWG and comes back with all 565 entities, 27 layers and 77 fills, which is what part 2 builds on.

## Part 2: DWG, and File > Export > Plan as DXF / DWG

- `save` to a `.dwg` makes the plan's DXF (`dxf_out`) and hands it to the session's **`DwgWriter`** (`cad::DwgWriter`, `Session::dwg_out`), the twin of the existing `DwgReader`. The core still never depends on `bimcraft-dwg`: the desktop, the command line and the MCP server set it to `bimcraft_dwg::writer()`, behind their `dwg` feature, as they do the reader. A session without one (the core's and the desktop's tests, a build without the feature) says to save as .dxf.
- `bimcraft_dwg::from_dxf` is acadrust reading that DXF and writing it as DWG (what `dwg_from_dxf` already did for the tests, which keeps its name). It runs **in process**, under `catch_unwind`: unlike the reader, what it reads is always the core's own DXF, never a stranger's file, so the child process the reader needs isn't worth its cost here. The crate's CLAUDE.md says so.
- The desktop's **File > Export** gets *Plan as DXF…* and *Plan as DWG…*: the plan on screen's level (the lowest when no plan is on screen), and the project stays unsaved, as with every export. Labels in Portuguese and Spanish.

### Tests (DWG)

- `bimcraft-dwg`: a plan saved as DXF and as DWG through a session; the DWG read back through `to_dxf` has the same entities, the same layers and the same solid fills.
- Core: without a writer, `.dwg` is refused with a reply naming `.dxf` and no file is written; with one, what it returns is what's written.
- Desktop (egui_kittest): both profiles list the two items; with an upper level's plan on screen, the DXF holds that level's wall and not the ground floor's doors; the project stays unsaved; the tests' window, which has no writer, says to save as .dxf.
- The sample house's DXF also went through the ODA File Converter to DWG 2018 with audit on: no errors, and back to DXF.

### Both

- `cargo test --workspace` (3924 tests), clippy, fmt and `cargo i18n check` pass on macOS (M4).
