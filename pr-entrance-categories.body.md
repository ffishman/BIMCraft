## demand: other utilities' service categories as data

`demand` knows three utilities, CPFL, Enel and Celesc, each written as Rust tables. Most other Brazilian utilities print the same kind of table: a category, its phases, the installed load or demand it covers, the breaker, the conductors and the earthing. This PR reads those tables from one data file, so adding a utility means adding an entry to the file, with no new code. CPFL, Enel and Celesc stay as they are.

This follows the approach proposed to dansity on #474 (https://github.com/IPedrax/BIMCraft/pull/474#issuecomment-6920414491): each utility's data carries its source, edition and page, ready to move into the Building Code menu of #386 when that lands.

### The mechanism

- **`data/entrance-categories.json`, read by `entrance.rs`:**
  - Each utility, or group of utilities that share one standard, names its standard and the item or table and page of each table.
  - It has one table per supply system (lower voltage first, as `127/220`) and per use (`individual`, `metering_centre`).
  - Rows are kept as printed: the category, its phases, the installed load (kW) and the demand (kVA) it covers, the breaker, the measurement, and the other columns under names the utility explains in `columns`.
- **A range** runs from above `over` (or from 0) up to and including `up_to`, the way the tables print "de 5,1 a 8".
- **What `demand` returns** for a utility of the file:
  - the first row that holds the unit;
  - every row that holds it (`candidates`), for example where a single-phase and a three-phase row both cover a load;
  - the rows that need a value that was not given (the installed load or the demand).
- **The demand** is the designer's, passed in as a number. Each utility's own demand method stays in its own module or in `building_demand` (#458).
- **Groups:** members are found by their own names. A member may change a category by its own document, and the reply cites that document in `changed_by`.
- **New parameters:** `{utility, system?, entrance?, installed_kw?, demand_kva?, phases?}`.

### Utilities in this PR

- **Fecoergs** (RIC BT REGD 035.01.06, revision of 23/08/2024, Anexo J):
  - Tables: 127/220, 220/380, rural 220/440 and the metering centre.
  - Members: the 15 Rio Grande do Sul cooperatives that share this RIC.
  - Two members change C18 by their own documents:
    - Coprel's OTD 006.01 v01 (24/09/2025) lets C18 be measured directly.
    - Cerfox's Informes técnicos 07-22 and 08-22 and OT_NIC_019 v3.0 keep C18 indirect, except in a case they name.
  - Fecoergs' errata do not touch Anexo J.
- **Hidropan** (RIC 2014, Tabela 11).
- **Energisa** (NDU 001 version 7.0, November 2024, Tabelas 16 and 18):
  - The group's companies (EAC, EMR, EMS, EMT, ERO, ESE, ESS, ETO, EPB) are members, found by their own names.
  - M and B rows go by the installed load, T rows by the demand.
- **Equatorial** (NT.00001.EQTL revision 09, approved 22/05/2025, Tabelas 1 and 2):
  - Every row goes by the installed load.
  - The tables print no category names, so each row is named by the phases and breaker printed in its breaker column ("25 (MONO)" becomes `MONO 25 A`).
  - Not covered: the two tables that Nota Técnica 004/2026 adds for a unit fed by its own single-phase transformer (binding from 24/02/2027). They are listed in `not_covered`.
- **Copel** (NTC 901100, revision of 01/07/2020, Tabela 2):
  - Rows go by the maximum demand.
  - The rural categories (19, 22, 25, 46 to 48, single-phase three-wire at 127/254 V) are a table of their own.
  - The breakers that notes 9 and 10 allow are given as a second value.

Every value was checked against the printed page, not only against text extracted from the PDF.

### Tests

They check that:
- every table names its page;
- every column is explained;
- the installed-load ranges leave no gap;
- within a group of the same phases and installed load, the demand ranges have no gap or overlap and the breaker never shrinks;
- every change a member makes names a category that exists;
- each utility picks its categories at printed values and at the edges of the ranges;
- a member is found by its own name and its document is cited;
- `demand` refuses a parameter that a utility of the file does not take.

### Not in this PR

- Santa Catarina's cooperatives (Fecoerusc, FECO-D-04).
- São Paulo's and Rio de Janeiro's cooperatives (Fecoeresp, NTC-D-04).
- Cemirim.

Each has a table of its own, and they will come in a later PR the same way.
