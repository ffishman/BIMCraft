## Import CAD: a drawing with nothing in model space comes in from its paper space

Builds on the tables PR (#TABLES), whose commit it includes.

Many sheets, old ones above all, are drawn straight on the layout: the title sheet, a detail sheet of doors and windows, everything of an R11/R12 drawing with the model left empty. Import CAD refused them as drawing nothing.

Now, when model space draws nothing, the reader places paper space instead: the entities marked as paper space's (code 67) in ENTITIES, as R12 and the active layout keep them, or else the `*Paper_Space` block. It uses paper space's extents (`$PEXTMIN`/`$PEXTMAX`), keeps counting what model space left out, and says so: `Dxf::paper` is true and the reply's `notes` says the sheet came in, in paper units. A drawing with something in model space is read exactly as before, its paper space still left out and counted. A file with nothing in either is still refused, now saying "in model space or in paper space".

### Checked

- Tests: a sheet of R12 (paper space entities in ENTITIES, a viewport, an XLINE left out), one of 2000 and later (the `*Paper_Space` block), one with nothing at all; a drawing with both spaces keeps model space alone; `import_cad` on a paper-space sheet comes in with the note.
- **Real drawings**, import only, on the 104 DWG files that main refuses as drawing nothing:

  | | come in | refused |
  |---|---|---|
  | main | 0 | 104 |
  | with tables | 3 | 101 |
  | with paper space | 9 | 95 |

  The 6 that paper space brings are detail and title sheets drawn on the layout: an R11 drawing (1,406 entities), the door and window detail sheets of a Brazilian public health-centre project (9,321 and 3,481), two Canadian sheet templates and a US Navy title sheet. None got worse.
- The 95 still refused are right to be, or are for another change:
  - R12 files where acadrust gives no model-space entities although they have some (a limit of the R12 reader, reported separately);
  - 3D only (3DSOLID, 3DFACE, HELIX) or MLINE/XLINE, which the reader doesn't draw yet;
  - block definitions only;
  - files that are truly empty.
- `cargo test --workspace`, clippy, fmt and `cargo i18n check` pass.
