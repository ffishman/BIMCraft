## Save as DXF and DWG: a level's plan, in model millimetres, each category on its layer

Four commits and a merge of main, each reviewable on its own: the DXF writer; DWG on top of it through acadrust; imported CAD drawings written from their own entities (their layers, their text as TEXT, no lines of no length); and layer names with accents or another script written as the header's code page asks.

## Part 1: DXF

DXF export is on the README's "Next, in order" list and was the "still to come" of Import CAD. This is its first part: `save` to a path ending `.dxf` writes a level's plan (`view`, the lowest by default, as for .svg and .png).

### What the file holds

- **What the plan shows**, exactly: the view's display list (`render::plan`, the one the PNG and SVG draw), its pixels turned back into model millimetres with `View::mm`. So visibility, overrides, cut/projection, the crop and annotation all come out as the plan draws them, with no second drawing path to keep in step.
- **AutoCAD 2004 (AC1018)**, the oldest format with true colours, with the structure every reader since R2000 expects: header (`$INSUNITS` = mm, extents, handle seed), tables (VPORT, LTYPE, LAYER, STYLE, VIEW, UCS, APPID, DIMSTYLE, BLOCK_RECORD), the model and paper space blocks, the entities, and the root dictionary with the Model and Layout1 layouts. The skeleton follows CADCraft's writer (MIT OR Apache-2.0), cut down to what a view needs; no new dependency.
- **Layers by category**: each element's lines go on its category's layer, or the subcategory it's drawn in (`wall`, `door`, `dooropening`), what the view cuts on `<category>-cut`, patterns on `<category>-pattern`, lettering on `<category>-text` (a text note's on `text`), and what no element draws on `annotation`. This is the layering people expect from Revit's DWG export (category, with cut and projection split) without inventing a layer standard; a mapping table (AIA, NBR 13532, ISO 13567) could come later as data.
- **Lines** are LWPOLYLINEs in their true colour, with the standard line weight nearest their width on paper (`dxf_out::lineweight`) and their dashes as a linetype in model millimetres (`BIM_DASH_n`). **Fills** (the walls' poché) are solid HATCHes, the first ring the outline and the rest holes.

- **An imported CAD drawing** (Import CAD / Link CAD) is written from its own entities rather than from what the plan draws of it: each on its own layer (less those the import or the view hides), placed, turned and scaled as it sits in the project, its lettering as TEXT. So a DWG brought in goes back out with its layers and text, at about its own size. A line or a ring of no length is left out, since readers take it as an empty entity.
- **Names in any script.** The file says it is in `ANSI_1252`, so a layer's name, like a text's, writes what isn't ASCII as `\U+XXXX`: `MOBILIÁRIOS` and `通风系统图` come back whole from the DXF and from the DWG. Our own DXF reader now reads `\U+XXXX` in layer names too, as any 2004 file may write them.

### What is approximate, and what is left out

- The model's own lettering (tags, room names, dimensions' text) is written as our own font's strokes, as in the PDF, not as TEXT entities: it looks the same everywhere but can't be edited as text. TEXT/MTEXT with a style is a follow-up.
- A picture (an image underlay) can't be written; it is counted in the reply's `left_out`.
- An imported drawing's blocks are written exploded, as lines, so a big drawing grows (one of 29 MB came out at 40 MB). Writing them back as BLOCK and INSERT is a follow-up.
- A DWG over about 13 MB that this writes becomes, read back, a DXF past the reader's 128 MB (`MAX_BYTES`, in main), so BIMCraft can't reopen it although other readers do. Raising that limit is a separate change.
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

### Real drawings

1,214 DWG files were imported and saved as DXF and as DWG, before and after the last two commits:

- **Where they come from:** 39 real projects (residential, electrical, structural, plumbing, fire and lightning protection), 688 from public repositories (LibreDWG, ACadSharp, ezdxf, GDAL, LibreCAD and others), and 487 published by public bodies (Brazil's Ministry of Health and FNDE, US and Wisconsin standards, Seattle utilities, Taiwan, Japan).
- **What they are:** versions R11 to 2018, written by AutoCAD and LT, BricsCAD, DraftSight, ZWCAD, GstarCAD, ODA/Teigha, LibreCAD, LibreDWG and Revit (as the files' own AppInfo says, where they have one), with text in Portuguese, Chinese and Japanese.
- **The files themselves are not part of this PR.**

| Check | Result |
|---|---|
| Saves | 1,040 of 1,040 imports saved as DXF and as DWG, 0 failures (the other 174 files are refused by the import, as in main: empty, pre-R13, not DWG, or only in paper space) |
| ezdxf `recover` + `audit` on every DXF | 0 errors (12 before the last commit, all layer names with accents or CJK) |
| ODA File Converter, audit on, 75 picked (every one with any warning, the 10 largest, CJK text, one per version × program, the real projects, 5 at random) | 75 of 75 converted, no error report, entity counts equal to ours |
| LibreDWG `dwgread` | 16 warnings, the same files before and after: a polyline of 72,700 vertices far from the origin, past one of its sanity limits |
| Reopened in BIMCraft | all but 8, each past the reader's 128 MB (see above) |
| The source's layers kept | 15.9 per file on average; on the public bodies' files 2.8 before → 26.8 after (on a real electrical plan, 3 → 107) |
| Empty entities on reading back | 36 → 6, the 6 left being still under review |

### Both

- `cargo test --workspace` (3932 tests), clippy, fmt and `cargo i18n check` pass on macOS (M4).
