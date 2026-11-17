Built on `feat/building-code` at 68090934. Closes E9's last entry of `MOVING`: the tables that were still Rust in `demand.rs` become data. No number changes.

## What moves

- CPFL's GED 13 (version 46.0: Tabelas 1A–1C, 2 to 11, 13 to 15, the W per point) and Enel's GRI-0017 (version 02: Anexos A–E)
  are two new amendments over `nbr-5410`, `cpfl.json` and `enel.json`, as the other utilities are.
- Celesc's Tabela 01 (N-321.0001, December 2025) goes into your `celesc.json`, beside `building_demand`.
- They sit in one new electrical section, `service`, with the part only each utility has: `installation` (CPFL), `annexes` (Enel),
  `categories` (Celesc). Their shapes stay their own, so `demand`'s output is unchanged.
- `demand.rs` keeps only the methods and reads the section: the project's where its amendment is that utility's, else the
  library's. It leaves `MOVING` and joins `READERS`.
- `service` has a `check`: numbers finite and not below zero, power factors over 0 and at most 1, a demand factor at most 1 with
  its loads in order (a row with no load only last), breakers over 0 A; and a table a row can be picked from as printed: Tabela
  15's last band open, ranges that run up, 1 to 3 phases, conductors with a section, a neutral besides the phases.
- `demand` says `changed_by_project` for CPFL, Enel and Celesc where the project's tables are not the code's as printed.
- One line in your `codes/xml.rs`: `known()` took a section's rules from the first library code that holds it (Celesc's
  `service`), so an import lost CPFL's `installation` and Enel's `annexes`. It now takes them from every code that holds the
  section. Any section whose parts differ by code needs this, so you may prefer it in the engine on its own.

## How it was checked

- Before the Rust tables were deleted, a temporary test compared every lookup old against new over a dense sweep (loads from
  −1 to 400 kW in steps of 0.25, counts 0–199, every network and phase, BTU, hp, the uses of Tabela 15, Tabela 11). It passed
  and was removed with the old tables. `demand.rs` was the same at b87e132b and at 68090934, so it holds here.
- Each table is pinned by a digest: a changed number or a lost row fails, and the test says to check the table against its
  document before updating it.
- The GED's Example 3 and the existing `demand` tests pass unchanged. A project's copy of CPFL's Tabela 4 lowers Example 3's
  demand; a changed Celesc or Enel row shows in the category; a change to Celesc's table never reaches CPFL's or Enel's answer;
  CPFL's and Enel's changes go out as XML and come back; each kind of table no row can be picked from is refused, and the
  library's tables as printed are not.

## Open points

1. **The section's name.** `service` and its parts are mine; happy to take yours.
2. **One shape of categories.** Enel's annexes and Celesc's Tabela 01 could fold into `entrance`'s general shape, but that
   changes `demand`'s output, so it waits for your call.
