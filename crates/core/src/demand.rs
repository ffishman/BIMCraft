//! The demand of an installation, and the category of service it asks of the utility, as the utility's own standard works them out.
//! A pure function of its input, with no model behind it: the `demand` command is its only caller. The numbers are those of the
//! utility's document, which is not an ABNT standard (NBR 5410 gives no demand factors), so each is named by its utility and its
//! edition. CPFL works out a demand; ENEL and CELESC leave it to the designer and frame a supply by the installed load:
//!
//! - **CPFL** (CPFL Paulista, Piratininga, Santa Cruz and RGE): Norma Técnica GED 13, "Fornecimento em Tensão Secundária de
//!   Distribuição" (DIST-13-2026-RE-NOR), version 46.0, 19/03/2026. `D = a + b + c + d + e + f + g + h + i` (item 6.22.1): a
//!   lighting and outlets (Tabelas 2 and 3, or 15), b showers, taps, instant heaters and irons (Tabela 4), c boilers (Tabela 5), d
//!   dryers, ovens, dishwashers and microwaves (Tabela 6), e electric stoves (Tabela 7), f window air conditioners (Tabelas 8 and
//!   9), g motors (Tabelas 13, 14 and 10), h special equipment (Tabela 11, power factor 0.75), i hydromassage (Tabela 10). The
//!   category comes of the installed load up to 25 kW and of the demand above it (Tabelas 1A, 1B and 1C).
//!
//! What is not applied: the poles, boxes and the earthing conduit of Tabela 1 and its motor limits, Tabela 1D (two clients on one
//! pole), the motor starting rules of Tabela 12, and the power factors of discharge lighting in Tabela 15's buildings beyond the
//! one given.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The `service` section of the utilities whose methods are here: the part only each one has (`installation`, GED 13's tables,
/// in CPFL's code; `annexes`, GRI-0017's, in Enel's; `categories`, Tabela 01 of N-321.0001, in Celesc's). The other utilities'
/// categories are their `entrance` (`entrance.rs`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceRules {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installation: Option<Ged13>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annexes: Option<Gri0017>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub categories: Option<Rows<CelescRow>>,
}

impl crate::codes::Section for ServiceRules {
    const DISCIPLINE: crate::codes::Discipline = crate::codes::Discipline::Electrical;
    const KEY: &'static str = "service";
    fn check(&self) -> Result<(), String> {
        numbers_fit("service", &serde_json::to_value(self).map_err(|e| e.to_string())?)?;
        self.tables_fit()
    }
}

impl ServiceRules {
    /// What a table must be for a row to be picked as printed: a range that runs up from where it starts, a phase count of 1 to
    /// 3, a conductor with a section, and Tabela 15's bands ending open, so no load falls past them.
    fn tables_fit(&self) -> Result<(), String> {
        let range = |what: &str, lo: f64, hi: f64| {
            if lo < hi { Ok(()) } else { Err(format!("service: {what} runs from {lo} up to {hi}: its start is below its end")) }
        };
        let phases = |what: &str, n: u8| {
            if (1..=3).contains(&n) { Ok(()) } else { Err(format!("service: {what} has {n} phases, not 1, 2 or 3")) }
        };
        let section = |what: &str, mm2: &[f64]| {
            if mm2.iter().all(|x| *x > 0.0) { Ok(()) } else { Err(format!("service: a conductor of {what} has no section (0 mm²)")) }
        };
        if let Some(t) = &self.installation {
            for u in &t.commercial.uses {
                if u.bands.last().is_none_or(|b| b.0.is_some()) {
                    return Err(format!("service: Tabela 15's `{}` ends with a band with no end (null), so no load falls past it", u.use_));
                }
            }
            for (system, rows) in &t.categories.systems {
                for r in rows {
                    let what = format!("{system} {}", r.name);
                    phases(&what, r.phases)?;
                    section(&what, &[r.cable, r.aluminium, r.earth])?;
                }
            }
        }
        if let Some(t) = &self.annexes {
            for a in [&t.aerial, &t.aerial_120_240, &t.underground, &t.underground_submersible] {
                for r in &a.rows {
                    let what = format!("Anexo {} {}", a.annex, r.name);
                    range(&what, r.lo, r.hi)?;
                    if let Some(n) = r.connection.0 {
                        phases(&what, n)?;
                    }
                    section(&what, &[r.connection.1, r.connection.2, r.entry.0, r.entry.1, r.earth])?;
                }
            }
            for r in &t.aerial_delta.rows {
                let what = format!("Anexo B {}", r.names.join("/"));
                if let Some(n) = r.connection.0 {
                    phases(&what, n)?;
                }
                section(&what, &[r.connection.1, r.connection.2, r.entry, r.earth])?;
            }
        }
        if let Some(t) = &self.categories {
            for r in &t.rows {
                range(&r.name, r.lo, r.hi)?;
                if let Some((lo, hi)) = r.demand {
                    range(&format!("{}'s demand", r.name), lo, hi)?;
                }
                phases(&r.name, r.phases)?;
                if r.wires <= r.phases {
                    return Err(format!("service: {} has {} wires for {} phases: a neutral too", r.name, r.wires, r.phases));
                }
                section(&r.name, &[r.connection.0, r.connection.1, r.entry, r.earth.0, r.earth.1])?;
            }
        }
        Ok(())
    }
}

/// Whether every number in a utility's tables can be one: a load, a factor, a current, a section or a count is never below
/// zero, a factor never above 10, and a power factor is over 0 and at most 1.
pub(crate) fn numbers_fit(section: &str, v: &Value) -> Result<(), String> {
    fn walk(section: &str, key: &str, v: &Value, depth: usize) -> Result<(), String> {
        if depth > 64 {
            return Err(format!("{section}: the tables are nested too deep"));
        }
        match v {
            Value::Object(m) => m.iter().try_for_each(|(k, x)| walk(section, k, x, depth + 1)),
            Value::Array(a) => {
                // A utility's demand factors, [load, factor] by steps or bands and what lies beyond: a share, 1 at most.
                if section == "service" && (key == "steps" || key == "bands") && a.iter().all(Value::is_array) {
                    // and the loads they start at, in order.
                    let firsts: Vec<Option<&Value>> = a.iter().map(|x| x.as_array().and_then(|p| p.first())).collect();
                    // A row with no load (`null`) is the rest, so the last.
                    if firsts.iter().rev().skip(1).any(|f| f.is_none_or(Value::is_null)) {
                        return Err(format!("{section}: only the last row of `{key}` may have no load (the rest)"));
                    }
                    let at: Vec<f64> = firsts.iter().filter_map(|f| f.and_then(Value::as_f64)).collect();
                    if at.windows(2).any(|w| w[0] >= w[1]) {
                        return Err(format!("{section}: the loads of `{key}` grow from row to row"));
                    }
                    for x in a {
                        if let Some(f) = x.as_array().filter(|p| p.len() == 2).and_then(|p| p[1].as_f64())
                            && f > 1.0
                        {
                            return Err(format!("{section}: a demand factor in `{key}` is 1 at most, not {f}"));
                        }
                    }
                }
                a.iter().try_for_each(|x| walk(section, key, x, depth + 1))
            }
            Value::Number(n) => {
                let x = n.as_f64().unwrap_or(f64::NAN);
                if !x.is_finite() || !(0.0..=1e9).contains(&x) {
                    return Err(format!("{section}: `{key}` is a number from 0 up, not {n}"));
                }
                if key == "power_factor" && !(x > 0.0 && x <= 1.0) {
                    return Err(format!("{section}: `power_factor` is over 0 and at most 1, not {n}"));
                }
                if section == "service" && key == "beyond" && x > 1.0 {
                    return Err(format!("{section}: a demand factor (`beyond`) is 1 at most, not {n}"));
                }
                if key.starts_with("breaker") && x <= 0.0 {
                    return Err(format!("{section}: a breaker (`{key}`) is over 0 A, not {n}"));
                }
                if key == "factor" && x > 10.0 {
                    return Err(format!("{section}: `factor` is 10 at most, not {n}"));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    walk(section, section, v, 0)
}

/// The `service` section of each of BIMCraft's electrical codes that has one, by the code's id, in the library's order; one that
/// does not read is left out, which a test keeps from happening.
pub fn services() -> impl Iterator<Item = (&'static str, &'static ServiceRules)> {
    static ALL: OnceLock<Vec<(&'static str, ServiceRules)>> = OnceLock::new();
    ALL.get_or_init(|| {
        crate::codes::of(crate::codes::Discipline::Electrical)
            .filter_map(|c| Some((c.code.as_str(), serde_json::from_value(c.sections.get("service")?.clone()).ok()?)))
            .collect()
    })
    .iter()
    .map(|(c, s)| (*c, s))
}

/// The `service` section of the electrical code `code`.
pub fn service(code: &str) -> Option<&'static ServiceRules> {
    services().find(|(c, _)| *c == code).map(|(_, s)| s)
}

/// The utilities whose methods are here (their numbers are in `data/codes/electrical/`).
pub const UTILITIES: &[&str] = &["CPFL", "ENEL", "CELESC"];

/// Where the numbers come from, for each utility: the source its code's data names.
pub fn source(utility: &str) -> Option<&'static str> {
    match utility {
        "CPFL" => ged13().map(|t| t.source.as_str()),
        "ENEL" => gri0017().map(|t| t.source.as_str()),
        "CELESC" => Some(celesc_01().source.as_str()).filter(|s| !s.is_empty()),
        _ => None,
    }
}

/// The supply voltages of Tabelas 1A and 1B.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum System {
    S127_220,
    S220_380,
}

impl System {
    pub fn parse(s: &str) -> Option<System> {
        match s {
            "127/220" => Some(System::S127_220),
            "220/380" => Some(System::S220_380),
            _ => None,
        }
    }
}

/// A load of `count` equal appliances of `w` watts each.
#[derive(Clone, Copy, Debug)]
pub struct Load {
    pub w: f64,
    pub count: u32,
}

/// Window air conditioners: the VA and the W of one, and how many.
#[derive(Clone, Copy, Debug)]
pub struct Air {
    pub va: f64,
    pub w: f64,
    pub count: u32,
}

/// Motors: the kVA and the kW of one, and how many.
#[derive(Clone, Copy, Debug)]
pub struct Motor {
    pub kva: f64,
    pub kw: f64,
    pub count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialKind {
    ArcWeld,
    ResistanceWeld,
    Xray,
}

pub const SPECIAL_KINDS: &[&str] = &["arc_weld", "resistance_weld", "xray"];

impl SpecialKind {
    pub fn name(self) -> &'static str {
        match self {
            SpecialKind::ArcWeld => "arc_weld",
            SpecialKind::ResistanceWeld => "resistance_weld",
            SpecialKind::Xray => "xray",
        }
    }

    pub fn parse(s: &str) -> Option<SpecialKind> {
        Some(match s {
            "arc_weld" => SpecialKind::ArcWeld,
            "resistance_weld" => SpecialKind::ResistanceWeld,
            "xray" => SpecialKind::Xray,
            _ => return None,
        })
    }
}

/// Special equipment: the kW of one, and how many.
#[derive(Clone, Copy, Debug)]
pub struct Special {
    pub kind: SpecialKind,
    pub kw: f64,
    pub count: u32,
}

/// What a building that is not a dwelling declares for its lighting and outlets: its use (Tabela 15), the kW and the power factor.
#[derive(Clone, Debug)]
pub struct OtherUse {
    pub kind: String,
    pub kw: f64,
    pub pf: f64,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub residential: bool,
    pub area: Option<f64>,
    pub outlets: Option<u32>,
    pub lighting_points: u32,
    pub other: Option<OtherUse>,
    pub heaters: Vec<Load>,
    pub boilers: Vec<Load>,
    pub appliances: Vec<Load>,
    pub stoves: Vec<Load>,
    pub air: Vec<Air>,
    pub motors: Vec<Motor>,
    pub special: Vec<Special>,
    pub hydro: Vec<Load>,
    /// The air conditioners are for commercial use (Tabela 9), not residential (a factor of 1).
    pub commercial_air: bool,
}

/// The terms of `D`, in kVA.
#[derive(Clone, Debug, Default)]
pub struct Terms {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
    pub g: f64,
    pub h: f64,
    pub i: f64,
}

#[derive(Clone, Debug)]
pub struct Output {
    pub installed_kw: f64,
    pub lighting_outlets_w: f64,
    pub terms: Terms,
    pub demand_kva: f64,
}

/// A factor by a count or a load: the first step whose bound reaches it, `beyond` past the last.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Steps {
    pub source: String,
    /// [up to, factor], in order.
    pub steps: Vec<(f64, f64)>,
    pub beyond: f64,
}

impl Steps {
    pub fn at(&self, x: f64) -> f64 {
        self.steps.iter().find(|s| x <= s.0).map_or(self.beyond, |s| s.1)
    }
}

/// What item 6.22.1 a) counts a dwelling's points at.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DwellingPoints {
    pub source: String,
    pub outlet_w: f64,
    pub kitchen_outlet_w: f64,
    pub lighting_point_w: f64,
    /// The kitchen's outlets where Tabela 2 gives none (no area, or above it).
    pub kitchen_outlets: u32,
}

/// A table of rows of numbers, as printed.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rows<R> {
    pub source: String,
    pub rows: Vec<R>,
}

/// One kind of special equipment's factors by rank.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RankFactors {
    pub steps: Vec<(f64, f64)>,
    pub beyond: f64,
}

/// Tabela 11: special equipment.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecialTable {
    pub source: String,
    pub power_factor: f64,
    pub kinds: BTreeMap<String, RankFactors>,
}

/// Tabelas 13 and 14: [cv, kW, kVA].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotorTables {
    pub source: String,
    pub single_phase: Vec<(f64, f64, f64)>,
    pub three_phase: Vec<(f64, f64, f64)>,
}

/// A use of Tabela 15: the demand factor by bands of the declared load ([up to kW, factor], the last open with `null`), and
/// whether its showers and taps go by Tabela 4 (item 6.22.1 b).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Use {
    #[serde(rename = "use")]
    pub use_: String,
    pub bands: Vec<(Option<f64>, f64)>,
    pub heaters_by_table: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UseTable {
    pub source: String,
    pub uses: Vec<Use>,
}

/// One category of Tabela 1A, 1B or 1C.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub name: String,
    /// The installed load (kW) and, for a three-phase row, the demand (kVA) it goes up to: the first row that holds both is taken.
    pub installed_up_to: f64,
    pub demand_up_to: Option<f64>,
    /// The breaker (A), the copper PVC cable and its aluminium alternative (mm²), the conduit (mm) and the earthing conductor
    /// (mm²).
    #[serde(rename = "breaker_a")]
    pub breaker: f64,
    pub cable: f64,
    pub aluminium: f64,
    pub conduit: f64,
    pub earth: f64,
    /// The phases: 1 (one phase and the neutral), 2 or 3.
    pub phases: u8,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Categories {
    pub source: String,
    /// By supply system (`127/220`, `220/380`), the rows in order.
    pub systems: BTreeMap<String, Vec<Category>>,
}

/// CPFL's GED 13, as `data/codes/electrical/cpfl.json` holds it (the `installation` of its `service` section).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ged13 {
    pub source: String,
    pub dwelling: DwellingPoints,
    pub least_outlets: Rows<(f64, u32, u32)>,
    pub lighting: Steps,
    pub heaters: Steps,
    pub boilers: Steps,
    pub appliances: Steps,
    pub stoves: Steps,
    pub window_air: Rows<(f64, f64, f64)>,
    pub commercial_air: Steps,
    pub motor_rank: Steps,
    pub special: SpecialTable,
    pub motors: MotorTables,
    pub commercial: UseTable,
    pub categories: Categories,
}

/// GED 13's tables; `None` if they do not read, which a test keeps from happening.
pub fn ged13() -> Option<&'static Ged13> {
    static T: OnceLock<Option<Ged13>> = OnceLock::new();
    T.get_or_init(|| service("cpfl").and_then(|s| s.installation.clone())).as_ref()
}

impl Ged13 {
    /// Tabela 2: the outlets of 100 W and the outlets of 600 W of the kitchen a dwelling has at least, by built area (m²); `None`
    /// above 250 m², where they are declared.
    pub fn least_outlets(&self, area: f64) -> Option<(u32, u32)> {
        self.least_outlets.rows.iter().find(|r| area <= r.0).map(|r| (r.1, r.2))
    }

    /// Tabela 8: a window air conditioner by its BTU/h, the VA and the W of one (the figures are the same at 110 and 220 V).
    pub fn window_air(&self, btu: f64) -> Option<(f64, f64)> {
        self.window_air.rows.iter().find(|r| r.0 == btu).map(|r| (r.1, r.2))
    }

    /// Tabelas 13 (single-phase) and 14 (three-phase): a motor by its cv, its kVA and its kW.
    pub fn motor(&self, cv: f64, phases: u32) -> Option<(f64, f64)> {
        let rows = if phases == 1 { &self.motors.single_phase } else { &self.motors.three_phase };
        rows.iter().find(|r| (r.0 - cv).abs() < 1e-6).map(|r| (r.2, r.1))
    }

    fn use_of(&self, kind: &str) -> Option<&Use> {
        self.commercial.uses.iter().find(|u| u.use_ == kind)
    }

    /// Tabela 15: the demand (kW) of a building's declared lighting and outlets load of `kw`, by its use; `None` for a use the
    /// table does not have.
    pub fn commercial_demand(&self, kind: &str, kw: f64) -> Option<f64> {
        let (mut left, mut from, mut total) = (kw.max(0.0), 0.0, 0.0);
        for (to, f) in &self.use_of(kind)?.bands {
            let to = to.unwrap_or(f64::INFINITY);
            let part = left.min(to - from);
            total += part * f;
            left -= part;
            from = to;
            if left <= 0.0 {
                break;
            }
        }
        Some(total)
    }

    /// The uses of Tabela 15, as a command gives them.
    pub fn uses(&self) -> Vec<&str> {
        self.commercial.uses.iter().map(|u| u.use_.as_str()).collect()
    }

    /// Whether Tabela 4 (not a factor of 1) applies to the showers and taps of a building of this use (item 6.22.1 b).
    fn heaters_by_table(&self, kind: &str) -> bool {
        self.use_of(kind).is_some_and(|u| u.heaters_by_table)
    }

    /// The category of service for an installed load `c` (kW) and a demand `d` (kVA), `None` above the table: the first row
    /// whose installed load, and demand where it prints one, hold them.
    pub fn category(&self, system: System, c: f64, d: f64) -> Option<&Category> {
        let key = match system {
            System::S127_220 => "127/220",
            System::S220_380 => "220/380",
        };
        self.categories.systems.get(key)?.iter().find(|r| c <= r.installed_up_to && r.demand_up_to.is_none_or(|u| d <= u))
    }
}

// What the library's GED 13 gives, for the tests that read its tables as printed.

pub fn least_outlets(area: f64) -> Option<(u32, u32)> {
    ged13()?.least_outlets(area)
}

fn steps(f: impl Fn(&Ged13) -> &Steps, x: f64) -> Option<f64> {
    ged13().map(|t| f(t).at(x))
}

/// Tabela 3: the demand factor of a residence's lighting and outlets by its installed load (kW).
pub fn lighting_factor(kw: f64) -> Option<f64> {
    steps(|t| &t.lighting, kw)
}

/// Tabela 4: showers, taps, instant heaters and irons, by how many there are in all.
pub fn heater_factor(n: u32) -> Option<f64> {
    steps(|t| &t.heaters, f64::from(n))
}

/// Tabela 5: central or storage heaters (boilers).
pub fn boiler_factor(n: u32) -> Option<f64> {
    steps(|t| &t.boilers, f64::from(n))
}

/// Tabela 6: dryers, ovens, dishwashers and microwave ovens.
pub fn appliance_factor(n: u32) -> Option<f64> {
    steps(|t| &t.appliances, f64::from(n))
}

/// Tabela 7: electric stoves.
pub fn stove_factor(n: u32) -> Option<f64> {
    steps(|t| &t.stoves, f64::from(n))
}

/// Tabela 9: commercial window air conditioners, by how many.
pub fn air_factor(n: u32) -> Option<f64> {
    steps(|t| &t.commercial_air, f64::from(n))
}

/// Tabela 10: motors (and hydromassage), by the rank of the motor from the biggest.
pub fn motor_factor(rank: u32) -> Option<f64> {
    steps(|t| &t.motor_rank, f64::from(rank))
}

pub fn window_air(btu: f64) -> Option<(f64, f64)> {
    ged13()?.window_air(btu)
}

pub fn motor(cv: f64, phases: u32) -> Option<(f64, f64)> {
    ged13()?.motor(cv, phases)
}

pub fn commercial_demand(kind: &str, kw: f64) -> Option<f64> {
    ged13()?.commercial_demand(kind, kw)
}

pub fn uses() -> Vec<&'static str> {
    ged13().map(Ged13::uses).unwrap_or_default()
}

pub fn category(system: System, c: f64, d: f64) -> Option<&'static Category> {
    ged13()?.category(system, c, d)
}

/// The kind of network a supply is made from, for the utility's annexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    /// Anexo A: aerial, 127/220 V.
    Aerial,
    /// Anexo C: aerial, 120/240 V.
    Aerial120240,
    /// Anexo D: underground, surface standard, 127/220 V.
    Underground,
    /// Anexo E: underground, submersible.
    Submersible,
}

/// The networks of the annexes that frame a supply by its installed load; Anexo B (`aerial_delta`) goes by the current instead.
pub const NETWORKS: &[&str] = &["aerial", "aerial_delta", "aerial_120_240", "underground", "underground_submersible"];

impl Network {
    pub fn parse(s: &str) -> Option<Network> {
        match s {
            "aerial" => Some(Network::Aerial),
            "aerial_120_240" => Some(Network::Aerial120240),
            "underground" => Some(Network::Underground),
            "underground_submersible" => Some(Network::Submersible),
            _ => None,
        }
    }

    /// The annex of GRI-0017 that holds its table.
    pub fn annex(self) -> &'static str {
        match self {
            Network::Aerial => "A",
            Network::Aerial120240 => "C",
            Network::Underground => "D",
            Network::Submersible => "E",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Network::Aerial => "aerial",
            Network::Aerial120240 => "aerial_120_240",
            Network::Underground => "underground",
            Network::Submersible => "underground_submersible",
        }
    }
}

/// One row of Enel's Anexo A, C, D or E (GRI-0017): a category of supply for a range of installed load.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnelRow {
    pub name: String,
    /// The installed load (kW) it covers: over `lo`, up to `hi`.
    #[serde(rename = "over")]
    pub lo: f64,
    #[serde(rename = "up_to")]
    pub hi: f64,
    /// The general breaker, A.
    #[serde(rename = "breaker_a")]
    pub breaker: f64,
    /// The connection branch: the number of conductors a phase has (`None` where the annex prints none), the phase's section and the
    /// neutral's, mm².
    pub connection: (Option<u8>, f64, f64),
    /// The entry branch: the phase's section and the neutral's, mm².
    pub entry: (f64, f64),
    /// The earthing conductor, copper, mm².
    pub earth: f64,
    pub measurement: String,
    /// The junction box (caixa de passagem) of Anexo E.
    #[serde(default)]
    pub junction_box: Option<String>,
}

/// An annex: its letter, where it is, and its rows in the printed order (a range starts where the one before ends).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annex<R> {
    pub annex: String,
    pub source: String,
    pub rows: Vec<R>,
}

/// One row of Enel's Anexo B (aerial, 120/240 V, delta with a neutral): a supply by the breaker, which the designer's demand current
/// must not exceed. Types A, B and C are one, two and three phases; a row can serve two types.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeltaRow {
    pub names: Vec<String>,
    #[serde(rename = "breaker_a")]
    pub breaker: f64,
    pub connection: (Option<u8>, f64, f64),
    /// The entry branch's phase conductor, mm².
    pub entry: f64,
    pub earth: f64,
    pub measurement: String,
}

/// Enel's GRI-0017, as `data/codes/electrical/enel.json` holds it (the `annexes` of its `service` section).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gri0017 {
    pub source: String,
    pub aerial: Annex<EnelRow>,
    pub aerial_120_240: Annex<EnelRow>,
    pub underground: Annex<EnelRow>,
    pub underground_submersible: Annex<EnelRow>,
    pub aerial_delta: Annex<DeltaRow>,
}

/// GRI-0017's annexes; `None` if they do not read, which a test keeps from happening.
pub fn gri0017() -> Option<&'static Gri0017> {
    static T: OnceLock<Option<Gri0017>> = OnceLock::new();
    T.get_or_init(|| service("enel").and_then(|s| s.annexes.clone())).as_ref()
}

/// The rows that fit an installed load.
pub struct Pick<'a> {
    /// The first is the smaller supply.
    pub rows: Vec<&'a EnelRow>,
}

fn phases_of(r: &EnelRow) -> u8 {
    match r.name.as_bytes().first() {
        Some(b'A') => 1,
        Some(b'B') => 2,
        _ => 3,
    }
}

impl Gri0017 {
    fn rows(&self, n: Network) -> &[EnelRow] {
        match n {
            Network::Aerial => &self.aerial.rows,
            Network::Aerial120240 => &self.aerial_120_240.rows,
            Network::Underground => &self.underground.rows,
            Network::Submersible => &self.underground_submersible.rows,
        }
    }

    /// A row of an annex by its name.
    pub fn row(&self, n: Network, name: &str) -> Option<&EnelRow> {
        self.rows(n).iter().find(|r| r.name == name)
    }

    /// The rows of an annex whose range holds `kw` (over `lo`, up to `hi`), those of `phases` phases if given; where none holds
    /// it (the annex prints whole kilowatts, so there is a gap between two ranges) the first that is higher. `None` above the
    /// table.
    pub fn pick(&self, n: Network, kw: f64, phases: Option<u8>) -> Option<Pick<'_>> {
        let rows: Vec<&EnelRow> = self.rows(n).iter().filter(|r| phases.is_none_or(|p| phases_of(r) == p)).collect();
        let holding: Vec<&EnelRow> = rows.iter().copied().filter(|r| kw > r.lo && kw <= r.hi || (r.lo == 0.0 && kw <= r.hi)).collect();
        if !holding.is_empty() {
            return Some(Pick { rows: holding });
        }
        rows.iter().copied().find(|r| r.hi >= kw).map(|r| Pick { rows: vec![r] })
    }

    /// The rows of Anexo B with the smallest breaker that holds `amps` (those of `phases` phases if given), the smaller supply
    /// first; `None` above the table.
    pub fn delta_pick(&self, amps: f64, phases: Option<u8>) -> Option<Vec<&DeltaRow>> {
        let all = &self.aerial_delta.rows;
        let serves = |r: &DeltaRow| {
            phases.is_none_or(|p| {
                r.names.iter().any(|n| match n.as_bytes().first() {
                    Some(b'A') => p == 1,
                    Some(b'B') => p == 2,
                    _ => p == 3,
                })
            })
        };
        let breaker = all.iter().filter(|r| serves(r) && r.breaker >= amps).map(|r| r.breaker).fold(f64::INFINITY, f64::min);
        let rows: Vec<&DeltaRow> = all.iter().filter(|r| serves(r) && r.breaker == breaker).collect();
        (!rows.is_empty()).then_some(rows)
    }
}

// What the library's GRI-0017 gives, for the tests that read its annexes as printed.

pub fn enel_row(n: Network, name: &str) -> Option<&'static EnelRow> {
    gri0017()?.row(n, name)
}

pub fn enel_pick(n: Network, kw: f64, phases: Option<u8>) -> Option<Pick<'static>> {
    gri0017()?.pick(n, kw, phases)
}

pub fn delta_pick(amps: f64, phases: Option<u8>) -> Option<Vec<&'static DeltaRow>> {
    gri0017()?.delta_pick(amps, phases)
}

fn total(loads: &[Load]) -> (u32, f64) {
    (loads.iter().map(|l| l.count).sum(), loads.iter().map(|l| l.w * f64::from(l.count)).sum::<f64>() / 1000.0)
}

/// The demand of an installation (item 6.22.1 of the GED) and the load it installs, by the library's GED 13.
pub fn compute(i: &Input) -> Result<Output, String> {
    ged13().ok_or("GED 13's tables did not read")?.compute(i)
}

impl Ged13 {
    /// The demand of an installation (item 6.22.1 of the GED) and the load it installs, by these tables.
    pub fn compute(&self, i: &Input) -> Result<Output, String> {
        let t13 = self;
        let pts = &t13.dwelling;
        // a) lighting and outlets.
        let (lighting_outlets_w, a, kind) = if i.residential {
            let kitchen = i.area.and_then(|a| t13.least_outlets(a)).map_or(pts.kitchen_outlets, |x| x.1);
            let outlets_w = match (i.outlets, i.area) {
                (Some(n), _) => f64::from(n) * pts.outlet_w + f64::from(kitchen) * pts.kitchen_outlet_w,
                (None, Some(area)) => match t13.least_outlets(area) {
                    Some((n, k)) => f64::from(n) * pts.outlet_w + f64::from(k) * pts.kitchen_outlet_w,
                    None => {
                        return Err("above 250 m² the outlets are declared: give `outlets` (100 W each; the kitchen's are added)".into());
                    }
                },
                (None, None) => return Err("give the built `area` (m²) or the number of `outlets`".into()),
            };
            let w = outlets_w + f64::from(i.lighting_points) * pts.lighting_point_w;
            (w, w / 1000.0 * t13.lighting.at(w / 1000.0), "")
        } else {
            let o =
                i.other.as_ref().ok_or("a building that is not a dwelling declares its lighting and outlets: give `lighting_outlets`")?;
            let d = t13.commercial_demand(&o.kind, o.kw).ok_or_else(|| format!("no use `{}` in Tabela 15", o.kind))?;
            (o.kw * 1000.0, d / o.pf, o.kind.as_str())
        };
        let mut t = Terms { a, ..Terms::default() };
        let mut installed = lighting_outlets_w / 1000.0;
        // b) showers, taps, instant heaters, irons.
        let (n, kw) = total(&i.heaters);
        installed += kw;
        t.b = kw * if i.residential || t13.heaters_by_table(kind) { t13.heaters.at(f64::from(n)) } else { 1.0 };
        // c) boilers.
        let (n, kw) = total(&i.boilers);
        installed += kw;
        t.c = kw * t13.boilers.at(f64::from(n));
        // d) dryers, ovens, dishwashers, microwaves.
        let (n, kw) = total(&i.appliances);
        installed += kw;
        t.d = kw * t13.appliances.at(f64::from(n));
        // e) stoves.
        let (n, kw) = total(&i.stoves);
        installed += kw;
        t.e = kw * t13.stoves.at(f64::from(n));
        // f) window air conditioners: the installed load in W, the demand in VA.
        let n: u32 = i.air.iter().map(|a| a.count).sum();
        installed += i.air.iter().map(|a| a.w * f64::from(a.count)).sum::<f64>() / 1000.0;
        let va: f64 = i.air.iter().map(|a| a.va * f64::from(a.count)).sum::<f64>() / 1000.0;
        t.f = va * if i.commercial_air { t13.commercial_air.at(f64::from(n)) } else { 1.0 };
        // g) motors, the biggest first.
        let mut motors: Vec<f64> = i.motors.iter().flat_map(|m| std::iter::repeat_n(m.kva, m.count as usize)).collect();
        motors.sort_by(|a, b| b.total_cmp(a));
        installed += i.motors.iter().map(|m| m.kw * f64::from(m.count)).sum::<f64>();
        t.g = motors.iter().enumerate().map(|(r, kva)| kva * t13.motor_rank.at(f64::from(r as u32 + 1))).sum();
        // h) special equipment at Tabela 11's power factor, each kind by its factors by rank.
        let sp = &t13.special;
        for kind in [SpecialKind::ArcWeld, SpecialKind::ResistanceWeld, SpecialKind::Xray] {
            let Some(rank) = sp.kinds.get(kind.name()) else {
                return Err(format!("Tabela 11 has no `{}`", kind.name()));
            };
            let mut v: Vec<f64> =
                i.special.iter().filter(|s| s.kind == kind).flat_map(|s| std::iter::repeat_n(s.kw, s.count as usize)).collect();
            v.sort_by(|a, b| b.total_cmp(a));
            installed += v.iter().sum::<f64>();
            t.h += v
                .iter()
                .enumerate()
                .map(|(r, kw)| {
                    let f = rank.steps.iter().find(|s| (r + 1) as f64 <= s.0).map_or(rank.beyond, |s| s.1);
                    kw / sp.power_factor * f
                })
                .sum::<f64>();
        }
        // i) hydromassage, by Tabela 10, at a power factor of 1.
        let mut v: Vec<f64> = i.hydro.iter().flat_map(|l| std::iter::repeat_n(l.w / 1000.0, l.count as usize)).collect();
        v.sort_by(|a, b| b.total_cmp(a));
        installed += v.iter().sum::<f64>();
        t.i = v.iter().enumerate().map(|(r, kw)| kw * t13.motor_rank.at(f64::from(r as u32 + 1))).sum();
        let demand_kva = t.a + t.b + t.c + t.d + t.e + t.f + t.g + t.h + t.i;
        Ok(Output { installed_kw: installed, lighting_outlets_w, terms: t, demand_kva })
    }
}

/// The networks of Celesc's Tabela 01 (N-321.0001): 220/380 V, whose units are single-phase at 220 V (A), two-phase (B) or
/// three-phase (C), and 220/440 V, whose single-phase units take 220 V (A) or 220/440 V on three wires (M).
pub const CELESC_NETWORKS: &[&str] = &["220/380", "220/440"];

/// One row of Celesc's Tabela 01: a category of supply for a range of installed load, as printed.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CelescRow {
    pub name: String,
    /// The installed load (kW) it covers: over `lo`, up to `hi`.
    #[serde(rename = "over")]
    pub lo: f64,
    #[serde(rename = "up_to")]
    pub hi: f64,
    /// The demand (kVA) a three-phase row is printed with beside its load: over the first, up to the second.
    pub demand: Option<(f64, f64)>,
    /// The unit's general breaker, A.
    #[serde(rename = "breaker_a")]
    pub breaker: f64,
    pub phases: u8,
    pub wires: u8,
    /// The aerial connection branch, multiplexed XLPE 90 °C: copper and aluminium, mm².
    pub connection: (f64, f64),
    /// The entry, exit and load branch, single-core HEPR/EPR/XLPE 90 °C in conduit (method B1) or buried (D), copper, mm².
    pub entry: f64,
    /// The earthing conductor, single-core HEPR/EPR/XLPE 90 °C: copper and copper-clad steel, mm².
    pub earth: (f64, f64),
}

/// Tabela 01 of N-321.0001 (rev. December 2025), in its printed order (the `categories` of the `service` section of
/// `data/codes/electrical/celesc.json`); empty if it does not read, which a test keeps from happening.
pub fn celesc_01() -> &'static Rows<CelescRow> {
    static T: OnceLock<Rows<CelescRow>> = OnceLock::new();
    T.get_or_init(|| service("celesc").and_then(|s| s.categories.clone()).unwrap_or_default())
}

impl Rows<CelescRow> {
    /// A row of Celesc's Tabela 01 by its name.
    pub fn row(&self, name: &str) -> Option<&CelescRow> {
        self.rows.iter().find(|r| r.name == name)
    }

    /// The rows of Tabela 01 a unit on `network` (`220/380` or `220/440`) may take: A, B and C on the first, A and M on the
    /// second.
    fn on(&self, network: &str) -> impl Iterator<Item = &CelescRow> {
        let m = network == "220/440";
        self.rows.iter().filter(move |r| r.name.starts_with('A') || r.name.starts_with('M') == m)
    }

    /// The row of Tabela 01 for an installed load of `kw` on `network`: the one whose range holds it, of `phases` phases if
    /// given (a unit with a two- or three-phase appliance takes that supply whatever its load, items 5.3.2.2 and 5.3.2.3), else
    /// the first of those phases that goes higher. `None` above the table (75 kW, or 50 kW on 220/440 V).
    pub fn pick(&self, network: &str, kw: f64, phases: Option<u8>) -> Option<&CelescRow> {
        let fits = |r: &&CelescRow| phases.is_none_or(|p| r.phases == p);
        self.on(network)
            .filter(fits)
            .find(|r| kw > r.lo && kw <= r.hi || (r.lo == 0.0 && kw <= r.hi))
            .or_else(|| phases.and_then(|_| self.on(network).filter(fits).find(|r| r.hi >= kw)))
    }
}

pub fn celesc_row(name: &str) -> Option<&'static CelescRow> {
    celesc_01().row(name)
}

pub fn celesc_pick(network: &str, kw: f64, phases: Option<u8>) -> Option<&'static CelescRow> {
    celesc_01().pick(network, kw, phases)
}

#[cfg(test)]
pub(crate) mod tests;
