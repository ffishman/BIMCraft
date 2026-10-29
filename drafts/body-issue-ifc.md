## What happens now

`import.rs` reads walls, doors, windows, floors, ceilings, rooms, columns, beams, footings, grids, stairs, ramps, railings, roofs and the MEP runs into BIMCraft elements, and the classes a family category reads (`Category::read_classes`: furniture, sanitary and waste terminals, light fixtures, outlets, switches, boards, fittings, valves, communications appliances, proxies) into components of families made from their bodies (`import.rs:2076`). Everything else inside the building is counted and listed as "N IFCxxx: BIMCraft has no element for them" (`import.rs:2831`) and is gone from the model.

What that leaves out of a typical file: curtain walls with their plates and members (`IfcCurtainWall`, `IfcPlate`, `IfcMember`), coverings that are not ceilings (flooring, cladding, roofing), ducts and air terminals, the site and its terrain (`IfcSite` bodies, `IfcGeographicElement`), transport elements, piles, shading devices, chimneys, element assemblies, discrete accessories and fasteners, and classes of IFC4.3 (`IfcCourse`, `IfcEarthworksFill`…). For an as-built (a scan or a measuring app exported to IFC, a model exported from Revit) these are often the part that matters, and open-then-save loses them.

The family path is the way in for all of them: a component whose family is made of its body. What is missing is taking it for every class, and keeping what the object was.

## Proposal

1. **Every physical product with a body comes in.** What has no native element becomes a component through the family path, in the `Generic` category, in a family per IFC class + type object (named after the type, or the class when there is none). It keeps:
   - the IFC class and PredefinedType / ObjectType;
   - the property and quantity sets, its own and its type's, values in SI as read;
   - its GlobalId as the element's GUID (as native elements already do);
   - its storey as the level.
2. **Export writes it back as it came**: the same class, predefined type, type object and sets. `ifc.rs` writes components by family category today; a family that came from IFC holds the class it was read as, and that wins.
3. **The openings that void it are cut** from its mesh (`csg.rs`), as a slab's or a wall's are.
4. **The import report changes**: "kept as generic objects: 12 IfcCovering, 4 IfcPlate…", and "left out" is only for what has no body or breaks a cap (`MAX_TRIANGLES`, `MAX_POINTS`).

## Data that is lost even for elements that come in natively

Read today: the instance property sets' `IfcPropertySingleValue`s (text, boolean, length, number), as project parameters by property name; marks; materials and layer sets; colours; BIMCraft's own sets (phases, design options, masses). Not read:

- **The property set's name.** Values land in project parameters by property name only, so `Pset_WallCommon.FireRating` and `Pset_DoorCommon.FireRating` become one parameter, and export can't write them back in the set they came from.
- **Other property kinds**: `IfcPropertyEnumeratedValue`, `IfcPropertyListValue`, `IfcPropertyBoundedValue`, `IfcPropertyTableValue`, `IfcComplexProperty`.
- **Type property sets** (`HasPropertySets` of the type object).
- **Quantity sets** (`IfcElementQuantity`): recomputed for native elements, which is right; for a kept object they are its only quantities.
- **Classification** (`IfcRelAssociatesClassification`, `IfcClassificationReference`): Uniclass, OmniClass, NBR 15965 codes are dropped.
- **Georeferencing** (`IfcMapConversion`, `IfcProjectedCRS`; `IfcSite`'s latitude, longitude, elevation and address): the model is moved near the origin and the offset reported, but the CRS and the site's data are not kept, so the export is no longer georeferenced.
- **Groups and zones** (`IfcZone`, `IfcGroup`), other than BIMCraft's systems.
- **Space boundaries** (`IfcRelSpaceBoundary`) and the space's own outline: a room comes back as a point, bounded by the walls.
- **Assemblies** (`IfcElementAssembly`) and parts (`IfcRelAggregates` beyond stairs and ramps).
- **Annotations** (`IfcAnnotation`: texts, dimensions, lines on a storey).
- **Documents and libraries** (`IfcRelAssociatesDocument`, `IfcLibraryReference`), presentation layers, textures.

Each is in the order below, with an export that writes it back and a round trip test.

## The geometry reader

What a kept object needs from `Reader::faces` that it does not read yet (IFC4 ADD2 TC1, ISO 16739-1):

- `IfcBooleanResult` / `IfcBooleanClippingResult` with DIFFERENCE and INTERSECTION: today the second operand is dropped (`import.rs:684`, "what a clipping cuts away doesn't change what the element is"). That holds for a wall's top clip, not for a kept object, where the uncut solid is wrong. `csg::subtract` can do it, bounded the same way.
- `IfcHalfSpaceSolid` / `IfcPolygonalBoundedHalfSpace` as the second operand of a clip.
- `IfcRevolvedAreaSolid` (and `Tapered`).
- `IfcSurfaceCurveSweptAreaSolid`, `IfcFixedReferenceSweptAreaSolid`, `IfcSweptAreaSolid` along a directrix (rails, handrails, profiles along a path).
- `IfcAdvancedBrep` with B-spline surfaces and curves, tessellated to a tolerance and to `MAX_TRIANGLES`.

Each one is a pure function over STEP entities with a test of its own: a small STEP text in, triangles or volume out.

## Rules this keeps

- A file is read kindly: what can't be kept is still listed, with why.
- Caps on every new path (triangles, nesting depth of booleans, B-spline degree and control points).
- No new dependency, no IFC library: the reader stays our own.
- The file: the kept class, type and sets are new fields with defaults. An older version reads such a family as a plain generic one and leaves the rest out (`lenient.rs`), which it would not misread, so no `FORMAT` bump.
- Size: objects of one type share one family and one mesh (as `IfcMappedItem` already gives), so a file with 500 identical chairs holds one chair; `MAX_FILE` and `MAX_TRIANGLES` still bound the rest.
- The MEP issue (connectors and the network) reads `IfcDistributionPort`s into connectors; a kept object keeps its ports too, so it joins the network when that comes.
- Later, IDEAS MOD-29 (elements by face from an IFC proxy) can turn a kept object into a wall or a floor.

## Schemas

The import takes any `FILE_SCHEMA` starting with IFC. Brazil adopted IFC4.3 as ABNT NBR ISO 16739-1:2026 (ISO 16739-1:2024), so files in `IFC4X3` will come: a test with one, checking that its building classes come in as IFC4's do and its new classes are kept, belongs here. Export stays IFC4.

## How to check it

- A round trip test: a file with one object of each kept class goes in, out and in again, and the class, predefined type, set names, values and GlobalIds are the same.
- IfcOpenShell in CI: the exported file validates and has the same count per class as the input.
- A real file: a Revit IFC4 export and a measuring app's IFC, with the report before and after.

## Order (each a PR)

| # | PR | Size | Needs |
|---|---|---|---|
| 1 | Every class with a body kept as a generic component, with its class, predefined type, type and GlobalId; export in the same class; the new report | M | — |
| 2 | Property sets kept by set name, every property kind, the type's sets, and the quantity sets of kept objects; export writes them back in their sets | M | — |
| 3 | Boolean DIFFERENCE and INTERSECTION and half spaces (`csg.rs`), and the openings that void a kept object | M | 1 |
| 4 | Classification (`IfcClassificationReference`), in and out | S | 2 |
| 5 | Georeferencing (`IfcMapConversion`, `IfcProjectedCRS`, the site's latitude, longitude and address), in and out | S–M | — |
| 6 | Revolved and swept solids | M | 1 |
| 7 | Advanced B-rep with B-spline surfaces | L | 1 |
| 8 | Zones and groups, assemblies, space boundaries, annotations | M each | — |

1 and 2 are what a file from another tool loses most and can go in parallel. 4 waits on 2 because a classification lands where the sets do.

## Questions before the first PR

1. Kept objects as components of `Generic` families holding their IFC class: is that the shape you want, or a category of their own (`imported`) that Visibility/Graphics can hide in one go?
2. Property sets: keep them as project parameters named with their set (`Pset_WallCommon.FireRating`), which schedules and filters already read, or as a `psets` map on the element? The first is smaller; the second is exact.
3. Georeferencing: on `Model`, or on a survey point of its own (the start of shared coordinates)?
4. Booleans: only for kept objects, leaving walls' and slabs' clipping as it is now?
