This is the ROADMAP 2.x MEP row's "Left" list ("fittings and connectors (a pipe network that knows where it ends), sizing, circuits and panel schedules, a family kind for IFC predefined types") and the limits in `docs/disciplines.md` ("a pipe network cannot yet be checked for continuity or sized"; "a network that carries the units along the runs comes with connectors"). It proposes one design for both plumbing and electrical and the order to build it in, so each slice is a PR of its own size. I'd like to agree the design here before the first PR.

## Where it stands (main, 0733fd4)

- Pipes, conduits, trays, wires, conductors and cables are `Run`s with a `system` name (`IfcSystem` in IFC). Runs that meet butt against each other.
- Fixtures, devices, boards, fittings and valves are components of a family whose `family::Category` says what it is. A family has no kind, so every sanitary terminal is `.NOTDEFINED.` and a WC is a basin to any reader. A fitting is placed by hand; nothing says a pipe ends on it (`family.rs:31`).
- Sizing is done and stateless: `circuit` and `conduit_fill` (NBR 5410), `drain_size` and `vent_size` (NBR 8160), `water_size` (NBR 5626), `storm_size` (NBR 10844), `hydrant_size` (IT 22). The caller types the load, the length, the fixtures by kind (`{wc, lav, sink, shower, laundry}`), the fittings by kind for the equivalent length (`{elbow90, tee_side, …}`).

So the drawing and the sizing are two separate things: to size a branch you count its fixtures and measure it yourself.

## Design

### 1. A family kind (prerequisite, already on the ROADMAP)

`Family::kind`, a closed list per category that maps to the IFC predefined type and to the NBR tables' rows: for a sanitary terminal `wc`, `lav`, `sink`, `shower`, `bath`, `laundry`, `floor_drain`…; for a fitting `bend`, `tee`, `cross`, `transition`, `coupling`; for a device `outlet`, `switch`, `luminaire`… The tables give the defaults (a WC's 6 UHC and DN 100, its weight for NBR 5626); a type parameter of the same name (`UHC`, `Weight`, `Power`) overrides them for that type. This alone fixes `.NOTDEFINED.` in IFC.

### 2. Connectors on families

A connector is a part of a family, with an id and a GUID like a form or a symbol (`HasPart`, in `parts_now`, `sync_parts` and `forget_parts`):

```
Connector { name, at: [Expr; 3], direction: [Expr; 3], domain: cold|hot|waste|vent|storm|gas|power|data|earthing, flow: in|out|both, size: Expr }
```

Expressions over the family's parameters, as forms are. Run ends are connectors without being stored (their point, their axis, their width). The starter families in `fixtures.rs` get theirs (a WC's waste outlet and water inlet, a board's outgoing side, an outlet's box). A file without them reads as today; an older version leaves them out (`lenient.rs`), which loses nothing it could use, so no `FORMAT` bump.

### 3. The network is derived, never stored

A pure function `network(model, level?, system?)`: run ends, connectors and a run's interior that meet within `TOL` become nodes; runs are edges (one split at an interior node, without splitting the element). Like wall joins, it is rebuilt from the model, so it cannot go stale and needs no rules for renaming or deleting. Bounded (`MAX_POINTS` nodes per call, no recursion), cached per model revision for drawing.

A query command `network {system?, level?}` replies with nodes, edges, open ends, and what looks wrong without refusing it (a drawing in progress has open ends): two systems meeting ("Esgoto" into "Água fria"), a drain that gets smaller downstream, a run whose slope is against its flow. Nothing of this goes in `Model::check`: an open end is not an invalid model.

### 4. Fittings as elements, made by a command (as `mass_floors` makes floors)

`fittings {system?, level?, prune?}` puts a fitting at each node where runs meet: two runs at an angle are a bend (45° or 90°), two in line of different sizes a transition, of the same size a coupling (only when asked), three a tee, four a cross. Each is a component of a starter fitting family chosen by kind and size, and holds the GUIDs of the runs it joins (`visit_refs!`). Running it again updates them instead of doubling them, `prune` removes the ones whose node is gone, and without it they are listed as `stale`: the same contract as `mass_floors`. A fitting placed by hand with connectors is a node already and is left alone.

Being elements, they get ids, GUIDs, phases, schedules, Visibility/Graphics and IFC (`IfcPipeFitting` / `IfcCableCarrierFitting` with BEND, TEE, CROSS, TRANSITION) for free. The runs keep their length in this slice (not cut back by the fitting), which a schedule's note says.

Why elements and not drawn on the fly: everything in BIMCraft that IFC writes and a schedule counts has an id and a GUID; a derived object would need GUIDs hashed from its runs and a second schedule path.

### 5. Plumbing sizing that reads the network

- Flow direction: a drain flows down its slope (`end_offset`) and towards a connector with `flow: out` (the outlet to the building drain); water flows from a connector with `flow: out` (the meter, the tank) to the points of use. A level run with neither is reported, not guessed.
- `drain_size {system}` and `vent_size {system}`: the UHC of the fixtures upstream of each run, summed along a spanning tree from the outlet, and the `role` from where the run is (branch, stack if it stands, main below the lowest level). A loop is reported.
- `water_size {system, from?}`: the weights downstream of each run, the length and the fittings by kind along the path to the farthest point of use (the fittings from step 4, as the `fittings` key counts them today), and the pressure left.
- The reply is per run. The same command called with numbers answers as today; reading the model is a second way in. A key `apply` that sets each pipe's type to the size found comes in its own PR, because it is the first sizing command that changes the model.

### 6. Electrical: the circuit is logical, the conduits are physical

A conduit carries several circuits, so a run's single `system` can't be the circuit. Instead:

- The circuit is the devices whose `System` is its name (the project parameter fixtures already carry), and the board that feeds it is named by a `Board` project parameter (Text) that the board and its devices both carry ("QD1"); components have no Mark today. No registry of circuits, as `docs/disciplines.md` wants.
- The conduit network is one network whatever the conduits' `system` says. A circuit's route is the shortest path in it from the board's connector to each of its devices; its length (for the drop) is the longest route plus the rises.
- `circuit {system}`: the load from the devices' `Power` (and power factor, `use` lighting or power from the kinds), the length from the routes, `grouped` from the circuits that share the busiest conduit on the route. `conduit_fill {run}`: the circuits routed through that conduit.
- A device not reached by any conduit is reported, and the circuit is sized without the drop, saying so.

### 7. Panel schedule and single-line diagram

- `panel {board}`: the board's circuits, with load per phase (balance), demand, breaker, section, drop, from step 6, as a table a sheet can hold. Whether it is a schedule kind with computed rows or a table of its own is a question for here: the schedule engine lists elements, and a circuit isn't one.
- The single-line diagram of a board as a drafting view made from the same table, placed on a sheet. Its own PR, last.

### 8. IFC ports

Export: each connector and each run end as an `IfcDistributionPort` (FlowDirection, SystemType, PredefinedType) nested in its element by `IfcRelNests`, and two ports at the same node joined by `IfcRelConnectsPorts`, as other tools write networks. Import: ports read back as a family's connectors, so a network from another tool comes in connected. ISO 16739-1 (IFC4 ADD2 TC1): IfcDistributionPort, IfcRelNests, IfcRelConnectsPorts.

### What the network gives beyond sizing

The same network checks lightning protection and data without any sizing: an SPDA whose down conductors all reach the earthing ring (NBR 5419 continuity), a data outlet reached by cabling from its rack (NBR 14565). Gas (NBR 15526) and hot water fit the same domains when their sizing comes. Ducts are out of scope: BIMCraft has no HVAC tab yet.

## Order (each a PR)

| # | PR | Size | Needs |
|---|---|---|---|
| 1 | Family kind + IFC predefined types + table defaults (UHC, weight) and type overrides | S–M | — |
| 2 | Connectors on families (part with id and GUID), starter families with theirs, Family Editor text | M | 1 |
| 3 | `network` query: nodes, edges, open ends, warnings | M | 2 |
| 4 | `fittings` command, starter fitting families by kind and size, IFC fittings | M | 3 |
| 5 | Plumbing sizing over a system (`drain_size`, `vent_size`, `water_size`) | M | 3 (4 for water's fittings) |
| 6 | Electrical: `Power` and `Board`, circuit routes over conduits, `circuit` and `conduit_fill` over the model | M–L | 3 |
| 7 | `apply`: set pipe and conduit sizes from the sizing | S | 5, 6 |
| 8 | Panel schedule on a sheet | M | 6 |
| 9 | IFC ports, out and back | M | 3 |
| 10 | Single-line diagram | L | 8 |

## How each is checked

- Network: a T of three pipes is one node of three edges; a 90° turn one node of two; two crossing runs of different systems or at different heights don't meet; a run ending inside another splits it in the graph only.
- Fittings: the T gets one tee and the turn one bend; run again, nothing doubles; delete a run and the fitting is `stale`, `prune` removes it; IFC writes `IfcPipeFitting .TEE.` and reads it back.
- Plumbing: a bathroom (basin, WC, shower) on one branch gives the branch the same UHC and DN as `drain_size {fixtures: {wc: 1, lav: 1, shower: 1}}`; the same for `water_size`'s weights.
- Electrical: two outlets 12 m of conduit from a board give the same answer as `circuit {load, length: 12000}`; two circuits sharing a conduit give `grouped: 2` and that conduit's fill counts both.
- IfcOpenShell: the ports and their connections validate and `ifcopenshell.util.system` walks the same network.

Related: the electrical issues #106–#114 and the sizing PRs #76–#85, whose tables this reuses unchanged.

## Questions before the first PR

1. Fittings as elements made by a command (the `mass_floors` contract), rather than drawn from the network on the fly: agreed?
2. A circuit's board: a `Board` project parameter on the board and its devices, or a Mark for components (which they don't have today)?
3. The panel schedule: a schedule kind with computed rows, or a table of its own that a sheet takes? The schedule engine lists elements, and a circuit isn't one.
4. Size: if the whole is too much for one issue, 1–4 (kind, connectors, network, fittings) stand on their own as a first phase, and sizing over the model can be a second issue.
5. I'd like to take PR 1 (the family kind) if that suits you.
