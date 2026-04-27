//! models.rs — Data structures for HerbReady Tauri backend.
//!
//! PatientRecord  : one row from the daily / search queries.
//! DrugItem       : a parsed entry from eligible / never / not-yet strings.
//! DispenseHistoryRecord : one visit row from the dispensing-history query.
//! DrugDispenseItem      : one drug per visit (tree-view row).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// DrugItem
// ---------------------------------------------------------------------------

/// A single herb drug entry after parsing the raw GROUP_CONCAT string.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DrugItem {
    pub drug_name: String,
    /// For not-yet-eligible drugs: days until eligible.  None otherwise.
    pub days_remaining: Option<i32>,
}

// ---------------------------------------------------------------------------
// PatientNameResult
// ---------------------------------------------------------------------------

/// Lightweight patient record returned by name-lookup (no drug data).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatientNameResult {
    pub hn: String,
    pub cid: String,
    pub pt_name: String,
    pub pttype_name: String,
}

// ---------------------------------------------------------------------------
// PatientRecord
// ---------------------------------------------------------------------------

/// One patient row returned by the daily-processing or individual-search query.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatientRecord {
    // ── Identifiers ──────────────────────────────────────────────────────
    pub vn: String,
    pub hn: String,
    pub cid: String,
    pub pt_name: String,

    // ── Department / entitlement ─────────────────────────────────────────
    pub current_dept_name: String,
    pub pttype_today: String,

    // ── Last-visit vitals ────────────────────────────────────────────────
    /// ISO date string (YYYY-MM-DD) or None.
    pub last_visit_date: Option<String>,
    pub last_weight: String,
    pub last_blood_pressure: String,
    pub last_pulse: String,

    // ── Raw GROUP_CONCAT drug strings ─────────────────────────────────────
    pub eligible_drugs_raw: String,
    pub never_dispensed_drugs_raw: String,
    pub not_yet_eligible_drugs_raw: String,

    // ── UI state ──────────────────────────────────────────────────────────
    /// Tracks per-drug toggle: true = will dispense (green), false = skip (white)
    pub drug_selection: HashMap<String, bool>,
    /// Whether this record is included in print / export set.
    pub print_selected: bool,
}

impl PatientRecord {
    /// Build a PatientRecord from the raw SQL row map.
    ///
    /// Parses eligible / never / not-yet raw strings and populates
    /// `drug_selection` with sensible defaults:
    ///   - eligible drugs → false (user selects manually)
    ///   - never-dispensed drugs → false (user must opt-in)
    ///   - not-yet-eligible → not in selection map (read-only / red)
    pub fn from_row(row: &HashMap<String, serde_json::Value>) -> Self {
        let get_str = |key: &str| -> String {
            row.get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
        };

        let eligible_raw = get_str("eligible_drugs");
        let never_raw = get_str("never_dispensed_drugs");
        let not_yet_raw = get_str("not_yet_eligible_drugs");

        let eligible_drugs = parse_simple_drug_list(&eligible_raw);
        let never_drugs = parse_simple_drug_list(&never_raw);

        let mut drug_selection = HashMap::new();
        for drug in &eligible_drugs {
            drug_selection.insert(drug.drug_name.clone(), false);
        }
        for drug in &never_drugs {
            drug_selection.insert(drug.drug_name.clone(), false);
        }

        // last_visit_date: sqlx returns NaiveDate; we store as ISO string
        let last_visit_date = row.get("last_visit_date").and_then(|v| {
            if v.is_null() {
                None
            } else {
                v.as_str().map(|s| s.to_string())
            }
        });

        PatientRecord {
            vn: get_str("vn"),
            hn: get_str("hn"),
            cid: get_str("cid"),
            pt_name: get_str("pt_name"),
            current_dept_name: get_str("current_dept_name"),
            pttype_today: get_str("pttype_today"),
            last_visit_date,
            last_weight: get_str("last_weight"),
            last_blood_pressure: get_str("last_blood_pressure"),
            last_pulse: get_str("last_pulse"),
            eligible_drugs_raw: eligible_raw,
            never_dispensed_drugs_raw: never_raw,
            not_yet_eligible_drugs_raw: not_yet_raw,
            drug_selection,
            print_selected: false,
        }
    }
}

// ---------------------------------------------------------------------------
// DispenseHistoryRecord
// ---------------------------------------------------------------------------

/// One visit row from the herbal-dispensing history query (Tab 3).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DispenseHistoryRecord {
    /// ISO date string or None.
    pub vstdate: Option<String>,
    pub hn: String,
    pub cid: String,
    pub pt_name: String,
    /// Comma-separated "drug_name (qty units)" string from GROUP_CONCAT.
    pub drugs_dispensed: String,
    /// Count of distinct herb drugs dispensed on this visit.
    pub drug_count: i32,
}

// ---------------------------------------------------------------------------
// DrugDispenseItem
// ---------------------------------------------------------------------------

/// One drug per visit row — used by the history tree view.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DrugDispenseItem {
    /// ISO date string or None.
    pub vstdate: Option<String>,
    pub drug_name: String,
    /// Quantity as a string (from CAST AS CHAR in SQL).
    pub qty: String,
    /// Unit string (e.g. "เม็ด", "ซอง") — falls back to "หน่วย".
    pub units: String,
}

// ---------------------------------------------------------------------------
// LabResult
// ---------------------------------------------------------------------------

/// One lab result row — latest per HN per lab item code up to a given date.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LabResult {
    pub hn: String,
    pub lab_items_code: String,
    pub lab_items_name: String,
    /// Raw result string from lab_order_result column.
    pub lab_order_result: String,
    /// ISO date string (YYYY-MM-DD).
    pub order_date: String,
}

// ---------------------------------------------------------------------------
// HerbDrugInteractionAlert
// ---------------------------------------------------------------------------

/// An alert indicating a patient has an active prescription for a modern drug
/// that interacts with the herb drugs configured in the interaction rule.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HerbDrugInteractionAlert {
    pub hn: String,
    pub modern_drug_icode: String,
    pub modern_drug_name: String,
    /// Names of the herb drugs that should NOT be co-administered.
    pub herb_drug_names: Vec<String>,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Private parsing helpers
// ---------------------------------------------------------------------------

/// Parse a comma-separated drug name string (eligible / never-dispensed).
///
/// Example input: `"ฟ้าทะลายโจร  500 mg., มะขามแขก, ขิง"`
pub fn parse_simple_drug_list(raw: &str) -> Vec<DrugItem> {
    if raw.trim().is_empty() {
        return vec![];
    }
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|name| DrugItem {
            drug_name: name.to_string(),
            days_remaining: None,
        })
        .collect()
}

/// Parse the not-yet-eligible drug string which contains day counts.
///
/// Example input: `"ขมิ้นชัน (in 5 days), กระเพรา (in 12 days)"`
pub fn parse_not_yet_drug_list(raw: &str) -> Vec<DrugItem> {
    if raw.trim().is_empty() {
        return vec![];
    }
    // Regex-free approach: look for " (in N days)" suffix
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|segment| {
            // Try to find "(in N days)" pattern
            if let Some(paren_start) = segment.rfind(" (in ") {
                let after = &segment[paren_start + 5..]; // skip " (in "
                                                         // after looks like "N days)" or "N day)"
                let days: Option<i32> =
                    after.split_whitespace().next().and_then(|n| n.parse().ok());
                let drug_name = segment[..paren_start].trim().to_string();
                DrugItem {
                    drug_name,
                    days_remaining: days,
                }
            } else {
                DrugItem {
                    drug_name: segment.to_string(),
                    days_remaining: None,
                }
            }
        })
        .collect()
}
