## demand: CPFL's GED 13 to version 46.0 and Enel's GRI-0017 (the former ET 1569) to version 02

`demand` was written against CPFL's GED 13 version 2.24 (2020) and Enel's ET 1569 version 01 (2024). Both utilities have published new editions since then. This PR brings the tables up to date. The command's shape and parameters stay the same.

### CPFL, GED 13 version 46.0 (19/03/2026)

I checked every table `demand` reads against the new edition.

- **Same values:** the demand factors (Tabelas 2 to 11) and Tabela 15, the old Tabela 18, are unchanged.
- **Renumbered:** the commercial table is now Tabela 15 (was 18), the motor tables are now 13 and 14 (were 14 and 15), motor starting is Tabela 12, and the formula is item 6.22.1 (was 6.25.1). The source text, the module doc and the params doc now use the new numbers.
- **Tabela 1 (1A single-phase, 1B two-phase, 1C three-phase):**
  - C9 (220/380 V) now goes up to 52 kVA instead of 46, so C10 starts above 52.
  - B1 and B3 now take a 32 mm conduit instead of 40 mm.
  - C7's earthing conductor is now 6 mm² instead of 10.
  - The table now prints an aluminium entry cable beside the copper one. The reply gives it as `aluminium_mm2`.
- **Three-phase motors:** Tabela 14 now goes up to 300 cv. Its note 3 says that above 50 cv the supply is at medium voltage.

### Enel, GRI-0017 version 02 (06/03/2025)

- **Anexo A (aerial, 127/220 V):**
  - Version 02 no longer prints a power made available. Instead each range now reaches up to that power (A1 to 5 kW, B1 to 11, C1 10 to 15 and so on), so ranges overlap. The command already returns every category that holds the load in `candidates`, the smaller supply first.
  - There is a new C8 at 175 A (57.1 to 67 kW), and the 200 A row is now C9 (67 to 75 kW).
  - The earthing conductor of B1, C1 and C2 is now 10 mm², and C4's entry is now 25(25).
- **Anexo B (delta):** B4 and C4 now connect by 16(16), and B9/C9 now earth by 50 mm².
- **Anexo C (120/240 V):** A2 is now 5 to 6 kW and B1 is now 6 to 12 kW.
- **Anexo D (underground):** the ranges were rewritten, there is a new C5 at 175 A (57.1 to 67 kW), and the 200 A row is now C6.
- **Anexo E:** unchanged.
- **Removed field:** `available_kw` is gone from the reply, because no annex prints it now. The demand note now cites Anexo A's note 11 (the designer works out a three-phase demand, up to 75 kW of installed load), and the 75 kW limit cites item 7.4.
- **Not covered:** Anexos F to H (Enel Ceará), listed in `not_covered`.

### Tests

The tests were changed first to the new printed values, then the code. They cover:
- every changed row;
- the overlapping ranges of Anexo A;
- the new 175 A rows;
- C9 at 52 kVA;
- the aluminium column;
- the 100 and 300 cv motors.

GED 13's Example 3 still comes out at 46.1 kW, D ≈ 33 kVA, C3. The area doc (`docs/areas/structure-and-mep.md`) is updated to match.

### Checks

`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo i18n check`.
