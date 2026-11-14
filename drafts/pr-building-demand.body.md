## A building's demand, by a utility's or a country's method, as data

`demand` covers one installation (CPFL's GED 13, Enel's and Celesc's categories). What it leaves out, and says so, is the demand of a **building of several dwellings**, which is what sizes a building's entrance. This adds `building_demand` for that, with the methods as data in `crates/core/data/building-demand.json`, read by `building_demand.rs`. A new utility or country is a new entry in the file, with no new code.

### One shape, five methods

I read the building-demand sections of the Brazilian utilities' standards and of Spain's ITC-BT-10. They all reduce to one formula:

```
D = factor · F(n) · d + common + special + commercial + premises
```

- **d** is one dwelling's demand. It comes either from the floor area, through a table and past its end a curve `c·A^e`, or from each dwelling's declared demand, with a least.
- **F(n)** is an equivalent number of dwellings for `n` dwellings, read from a table, plus a step for each dwelling past the table where the method gives one.
- **factor** is the method's constant. Some methods apply it to the common areas too.
- **common** is the common areas' load. Where the method gives demand-factor bands for a class (lighting, outlets), the bands apply to that class's installed load. The designer's own figure goes in `common_demand`.
- **special** loads count whole, an EV charging station for example.
- **premises** are loads given per m², such as shops and garages.

| Method | d | F(n) | factor | Entrance table |
|---|---|---|---|---|
| CELESC N-321.0003 (SC) | Tabela 02, the curve above 400 m² | Tabela 01 | k = 1.2, dwellings and common areas | Tabela 11 |
| CEMIG ND-5.2 (MG) | Tabela 13 | Tabela 12, 4 to 300 | 1.4 | Tabela 1 |
| FECOERGS REGD 035.01.06 (RS) | Anexo T | Anexo U | 1.2, dwellings only | none |
| HIDROPAN RIC BT 2014 (RS) | Tabela 3 | Tabela 4, up to 217 | 1.2, dwellings only | none |
| ITC-BT-10 (Spain, RD 842/2002) | declared, at least 5.75 kW; grades basic and elevated | Tabla 1, then 0.5 per dwelling past 21 | 1 | none |

Spain is there to show that the shape is not a Brazilian one: the mean of the dwellings' powers times a simultaneity coefficient, with services, shops and garages counted whole. The same pattern shows up in DIN 18015-1, NF C 14-100, NEC 220.84 and the UK's ADMD. A method of another shape, the installed load times simultaneity coefficients (CPFL's GED 119, NEC 220, AS/NZS 3000), is listed as not covered.

### Data checked against itself

- **Printed values and errata.** Tables keep the values as printed. An erratum corrects a row that the same table prints differently in the other standards:
  - Celesc's Tabela 01 at 62, 85 and 203;
  - Cemig's Tabela 12 at 51, 78, 121, 164 and 172.

  The reply says which errata it took, and `as_printed: true` uses the printed value instead.
- **Tests on the data itself.**
  - Every table names its standard and page.
  - Keys and values are in order once the errata are taken.
  - F(n) is never above n.
  - Entrance rows have no gap or overlap.
  - A curve meets the end of its table.
  - With the errata taken, the four Brazilian diversity tables are one table (they all come from CODI's RTD-027), and Celesc's area table follows its own curve to the cent.
- **The standards' own examples.**
  - Celesc's 20 × 100 m² + 16 × 80 m² is 36 apartments at 91.1 m², which the standard takes as 92 m².
  - Cemig's 1.4 × 19.86 × 1.96 = 54.50 kVA.

### The command

```json
{"utility": "CELESC", "dwellings": [{"count": 20, "area": 100}, {"count": 16, "area": 80}],
 "common": {"lighting": 14, "outlets": 5}, "special": 7}
```

The reply has each term, the demand (88.04 kVA), the entrance row (150 A and its conductors), the errata taken, `sources`, `notes` and `not_covered`. It reads nothing from the model and changes nothing. Hostile input is refused, with a test. The words in the data are mine: only numbers, the standard's name, item and page are taken from the standards.

`demand`'s Celesc reply now points to `building_demand` for a building of several units. The area doc (`docs/areas/structure-and-mep.md`) has the paragraph, and the module map has the line.

A follow-up PR will update `demand`'s CPFL tables (GED 13 is now at v46) and Enel's (ET 1569 v02) and move them to data in the same way.

### Checks

`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo i18n check`.
