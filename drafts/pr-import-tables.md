## Import CAD: AutoCAD tables (ACAD_TABLE) come in

A table in a drawing (a schedule of areas, a legend, a list of sheets, a door and window schedule) was left out and counted in `left_out`; a drawing that held only a table was refused as drawing nothing.

AutoCAD keeps how a table looks in an anonymous block (`*T…`) that the ACAD_TABLE entity names (code 2), drawn from the table's insertion point (code 10), as ezdxf's `AcadTable.virtual_entities` does too. So the reader now places that block there, as it does an INSERT's or a DIMENSION's, without reading the table's cells: the grid comes in as lines and the cells as text, on the table's layer where the block's entities are on layer 0. A table whose block is missing is counted as an INSERT of a block the file doesn't define, and never fails the file.

### Checked

- A test in `dxf.rs`: a table's block placed at its insertion point, its text where the cell has it, on the table's layer, nothing left out; a table with no block says so.
- **Real drawings**, import only, before and after, on 70 DWG files that have tables (from public repositories and public bodies):
  - the 66 that already came in now bring their tables, and none lists ACAD_TABLE as left out;
  - 3 of the 4 refused as drawing nothing now come in (the fourth is past the reader's 128 MB);
  - none got worse.
- **Against ezdxf**, on 4 of them through acadrust: the tables' bounds match ezdxf's exactly (one in inches 859 × 191, one 95.02 × 37.48 at the same insertion point, one in metres at EPSG coordinates). The entity counts match too: 24 = 24, and 2,463 = 2,463 with 1,512 cells' texts. In one file our count is higher, 157 against 125, because the reader splits a multi-line MTEXT into one entity per line, as it always has.
- `cargo test --workspace`, clippy, fmt and `cargo i18n check` pass.
