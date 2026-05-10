//! commands.rs — All Tauri `#[tauri::command]` handlers for HerbReady.
//!
//! Each command is `pub async fn cmd_*` returning `Result<T, String>`.
//! Errors are converted to strings so the frontend can display them directly.

use std::collections::HashMap;

use sqlx::{Column, Row};

use crate::config::{AppConfig, DatabaseConfig};
use crate::db;
use crate::models::{DispenseHistoryRecord, DrugDispenseItem, LabResult, PatientRecord};
use crate::queries;

// ---------------------------------------------------------------------------
// Utility: convert anyhow / sqlx errors to String
// ---------------------------------------------------------------------------

fn err_str<E: std::fmt::Display>(e: E) -> String {
  e.to_string()
}

// ---------------------------------------------------------------------------
// Helper: run a query and return rows as Vec<HashMap<String, serde_json::Value>>
// ---------------------------------------------------------------------------

async fn fetch_rows(
  sql: &str,
  params: &[String],
) -> Result<Vec<HashMap<String, serde_json::Value>>, String> {
  let pool = db::get_pool().await.map_err(err_str)?;

  // Acquire a single connection so that the SESSION SET and the main
  // SELECT share the same MySQL session (SET only affects the connection
  // it is issued on).
  let mut conn = pool.acquire().await.map_err(err_str)?;

  // Raise GROUP_CONCAT limit to avoid truncating drug-name lists when a
  // patient has many configured drugs.  The default is 1024 bytes which
  // can easily be exceeded.  Non-fatal: we proceed even if the SET fails.
  let _ = sqlx::query("SET SESSION group_concat_max_len = 65536")
    .execute(&mut *conn)
    .await;

  let _ = sqlx::query(
    "SET SESSION optimizer_switch='mrr=on,mrr_cost_based=off,index_condition_pushdown=on'",
  )
  .execute(&mut *conn)
  .await;

  // Build the query dynamically with positional `?` bindings.
  // sqlx requires compile-time queries for type-checked variants, but since
  // we generate SQL at run-time we use the raw query API.
  let mut q = sqlx::query(sql);
  for p in params {
    q = q.bind(p.as_str());
  }

  let rows = q.fetch_all(&mut *conn).await.map_err(err_str)?;

  let mut result = Vec::with_capacity(rows.len());
  for row in &rows {
    let mut map: HashMap<String, serde_json::Value> = HashMap::new();
    let cols = row.columns();
    for col in cols {
      let name = col.name().to_string();
      // Try extracting common types; fall through to null on failure.
      let val: serde_json::Value = if let Ok(v) = row.try_get::<Option<String>, _>(col.name()) {
        match v {
          Some(s) => serde_json::Value::String(s),
          None => serde_json::Value::Null,
        }
      } else if let Ok(v) = row.try_get::<Option<i64>, _>(col.name()) {
        match v {
          Some(n) => serde_json::Value::Number(n.into()),
          None => serde_json::Value::Null,
        }
      } else if let Ok(v) = row.try_get::<Option<i32>, _>(col.name()) {
        match v {
          Some(n) => serde_json::Value::Number(n.into()),
          None => serde_json::Value::Null,
        }
      } else if let Ok(v) = row.try_get::<Option<f64>, _>(col.name()) {
        match v {
          Some(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(f).unwrap_or_else(|| serde_json::Number::from(0)),
          ),
          None => serde_json::Value::Null,
        }
      } else if let Ok(v) = row.try_get::<Option<chrono::NaiveDate>, _>(col.name()) {
        match v {
          Some(d) => serde_json::Value::String(d.format("%Y-%m-%d").to_string()),
          None => serde_json::Value::Null,
        }
      } else {
        serde_json::Value::Null
      };
      map.insert(name, val);
    }
    result.push(map);
  }
  Ok(result)
}

// ---------------------------------------------------------------------------
// DB connection commands
// ---------------------------------------------------------------------------

/// Test connectivity without storing the pool.
#[tauri::command]
pub async fn cmd_test_connection(
  host: String,
  port: u16,
  dbname: String,
  user: String,
  password: String,
) -> Result<String, String> {
  db::test_connection(&host, port, &dbname, &user, &password)
    .await
    .map_err(err_str)
}

/// Connect (or reconnect) to the database and store the pool globally.
#[tauri::command]
pub async fn cmd_connect_db(
  host: String,
  port: u16,
  dbname: String,
  user: String,
  password: String,
) -> Result<String, String> {
  db::reset_pool(&host, port, &dbname, &user, &password)
    .await
    .map_err(err_str)?;
  Ok(format!("เชื่อมต่อสำเร็จกับ {}:{}/{}", host, port, dbname))
}

// ---------------------------------------------------------------------------
// Config commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn cmd_get_db_config() -> Result<DatabaseConfig, String> {
  crate::config::read_db_config().map_err(err_str)
}

#[tauri::command]
pub async fn cmd_save_db_config(
  host: String,
  port: u16,
  name: String,
  user: String,
  password: String,
) -> Result<(), String> {
  let cfg = DatabaseConfig {
    host,
    port,
    name,
    user,
    password,
  };
  crate::config::write_db_config(&cfg).map_err(err_str)
}

#[tauri::command]
pub async fn cmd_get_app_config() -> Result<AppConfig, String> {
  crate::config::read_app_config().map_err(err_str)
}

#[tauri::command]
pub async fn cmd_save_app_config(config: AppConfig) -> Result<(), String> {
  crate::config::write_app_config(&config).map_err(err_str)
}

// ---------------------------------------------------------------------------
// Daily tab commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn cmd_get_daily_records(
  process_date: String,
  vitals_on_date: bool,
) -> Result<Vec<PatientRecord>, String> {
  let app_cfg = crate::config::read_app_config().map_err(err_str)?;
  let dept_codes: Vec<String> = app_cfg.departments.iter().map(|d| d.code.clone()).collect();
  let (sql, params) =
    queries::build_daily_query(&process_date, &dept_codes, &app_cfg.drugs, vitals_on_date);

  let rows = fetch_rows(&sql, &params).await?;
  let records = rows.iter().map(PatientRecord::from_row).collect();
  Ok(records)
}

// ---------------------------------------------------------------------------
// Search tab commands
// ---------------------------------------------------------------------------

/// Auto-detect search field (HN / CID / name) and run individual search.
#[tauri::command]
pub async fn cmd_search_patient(
  process_date: String,
  search_text: String,
) -> Result<Vec<PatientRecord>, String> {
  let text = search_text.trim().to_string();
  if text.is_empty() {
    return Err("กรุณาพิมพ์ข้อมูลที่ต้องการค้นหา".into());
  }

  let app_cfg = crate::config::read_app_config().map_err(err_str)?;
  let is_all_digits = text.chars().all(|c| c.is_ascii_digit());

  let (sql, params) = if is_all_digits && text.len() >= 5 && text.len() <= 9 {
    // HN
    queries::build_individual_search_query(&process_date, Some(&text), None, None, &app_cfg.drugs)
  } else if is_all_digits && text.len() == 13 {
    // CID
    queries::build_individual_search_query(&process_date, None, Some(&text), None, &app_cfg.drugs)
  } else {
    // name
    if text.chars().count() < 2 {
      return Err("กรุณาระบุชื่ออย่างน้อย 2 ตัวอักษร".into());
    }
    queries::build_individual_search_query(&process_date, None, None, Some(&text), &app_cfg.drugs)
  };

  let rows = fetch_rows(&sql, &params).await?;
  let records = rows.iter().map(PatientRecord::from_row).collect();
  Ok(records)
}

// ---------------------------------------------------------------------------
// History tab commands
// ---------------------------------------------------------------------------

/// Fetch per-drug dispensing history for a single patient by HN.
#[tauri::command]
pub async fn cmd_get_patient_history(
  hn: String,
  years_back: Option<i32>,
) -> Result<Vec<DrugDispenseItem>, String> {
  let app_cfg = crate::config::read_app_config().map_err(err_str)?;
  let (sql, params) = queries::build_patient_herb_history_query(&hn, years_back, &app_cfg.drugs);

  let rows = fetch_rows(&sql, &params).await?;
  let items = rows
    .iter()
    .map(|row| {
      let get = |k: &str| -> String {
        row
          .get(k)
          .and_then(|v| v.as_str())
          .map(|s| s.to_string())
          .unwrap_or_default()
      };
      let vstdate = row.get("vstdate").and_then(|v| {
        if v.is_null() {
          None
        } else {
          v.as_str().map(|s| s.to_string())
        }
      });
      DrugDispenseItem {
        vstdate,
        drug_name: get("drug_name"),
        qty: get("qty"),
        units: get("units"),
      }
    })
    .collect();
  Ok(items)
}

/// Resolve a patient name to HN, then return their drug history.
/// Returns (records, hn, pt_name).
#[tauri::command]
pub async fn cmd_search_patient_name_for_history(
  name: String,
  years_back: Option<i32>,
) -> Result<(Vec<DrugDispenseItem>, String, String), String> {
  // Step 1: resolve name → HN
  let (lookup_sql, lookup_params) = queries::build_patient_lookup_by_name(&name);
  let lookup_rows = fetch_rows(&lookup_sql, &lookup_params).await?;

  if lookup_rows.is_empty() {
    return Err(format!("ไม่พบผู้ป่วยชื่อ \"{}\"", name));
  }

  let first = &lookup_rows[0];
  let hn = first
    .get("hn")
    .and_then(|v| v.as_str())
    .map(|s| s.to_string())
    .unwrap_or_default();
  let pt_name = first
    .get("pt_name")
    .and_then(|v| v.as_str())
    .map(|s| s.to_string())
    .unwrap_or_default();

  // Step 2: fetch history for that HN
  let records = cmd_get_patient_history(hn.clone(), years_back).await?;
  Ok((records, hn, pt_name))
}

// ---------------------------------------------------------------------------
// Drug / dept name lookup (for Settings dialog)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn cmd_lookup_drug_name(icode: String) -> Result<String, String> {
  let pool = db::get_pool().await.map_err(err_str)?;
  let row: Option<(String,)> = sqlx::query_as("SELECT name FROM drugitems WHERE icode = ? LIMIT 1")
    .bind(&icode)
    .fetch_optional(&pool)
    .await
    .map_err(err_str)?;
  match row {
    Some((name,)) => Ok(name),
    None => Err(format!("ไม่พบรหัสยา '{}'", icode)),
  }
}

#[tauri::command]
pub async fn cmd_lookup_dept_name(code: String) -> Result<String, String> {
  let pool = db::get_pool().await.map_err(err_str)?;
  let row: Option<(String,)> =
    sqlx::query_as("SELECT department FROM kskdepartment WHERE depcode = ? LIMIT 1")
      .bind(&code)
      .fetch_optional(&pool)
      .await
      .map_err(err_str)?;
  match row {
    Some((name,)) => Ok(name),
    None => Err(format!("ไม่พบรหัสหน่วยงาน '{}'", code)),
  }
}

// ---------------------------------------------------------------------------
// Patient name lookup (returns all matching patients without drug data)
// ---------------------------------------------------------------------------

/// Find all patients whose name contains the given text.
/// Returns up to 50 matches as lightweight PatientNameResult records.
#[tauri::command]
pub async fn cmd_find_patients_by_name(
  name: String,
) -> Result<Vec<crate::models::PatientNameResult>, String> {
  if name.trim().is_empty() {
    return Err("กรุณาระบุชื่อที่ต้องการค้นหา".into());
  }
  let (sql, params) = queries::build_patient_lookup_by_name(&name);
  let rows = fetch_rows(&sql, &params).await?;
  let results = rows
    .iter()
    .map(|row| {
      let get = |k: &str| -> String {
        row
          .get(k)
          .and_then(|v| v.as_str())
          .map(|s| s.to_string())
          .unwrap_or_default()
      };
      crate::models::PatientNameResult {
        hn: get("hn"),
        cid: get("cid"),
        pt_name: get("pt_name"),
        pttype_name: get("pttype_name"),
      }
    })
    .collect();
  Ok(results)
}

/// Look up a single patient by HN (5-9 digits) or CID (13 digits).
/// Returns None if not found.
#[tauri::command]
pub async fn cmd_find_patient_by_id(
  id: String,
) -> Result<Option<crate::models::PatientNameResult>, String> {
  let id = id.trim().to_string();
  if id.is_empty() {
    return Ok(None);
  }
  let is_cid = id.len() == 13 && id.chars().all(|c| c.is_ascii_digit());
  let (sql, params) = queries::build_patient_lookup_by_hn_or_cid(&id, is_cid);
  let rows = fetch_rows(&sql, &params).await?;
  let result = rows.first().map(|row| {
    let get = |k: &str| -> String {
      row
        .get(k)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default()
    };
    crate::models::PatientNameResult {
      hn: get("hn"),
      cid: get("cid"),
      pt_name: get("pt_name"),
      pttype_name: get("pttype_name"),
    }
  });
  Ok(result)
}

// ---------------------------------------------------------------------------
// Lab commands
// ---------------------------------------------------------------------------

/// Look up a lab item name by its lab_items_code.
#[tauri::command]
pub async fn cmd_lookup_lab_item_name(code: String) -> Result<String, String> {
  let (sql, params) = queries::build_lab_item_lookup_query(&code);
  let rows = fetch_rows(&sql, &params).await?;
  if let Some(row) = rows.first() {
    if let Some(serde_json::Value::String(name)) = row.get("lab_items_name") {
      return Ok(name.clone());
    }
  }
  Err(format!("ไม่พบรหัสแลป: {}", code))
}

/// Get the latest lab results for a list of HNs up to (and including) process_date.
/// Only returns results for lab_items_code values configured in lab_rules.
#[tauri::command]
pub async fn cmd_get_lab_results(
  process_date: String,
  hn_list: Vec<String>,
) -> Result<Vec<LabResult>, String> {
  let app_cfg = crate::config::read_app_config().map_err(err_str)?;
  if app_cfg.lab_rules.is_empty() {
    return Ok(vec![]);
  }

  let code_list: Vec<String> = app_cfg
    .lab_rules
    .iter()
    .map(|r| r.lab_items_code.clone())
    .collect();

  let query = queries::build_latest_lab_results_query(&process_date, &hn_list, &code_list);
  let (sql, params) = match query {
    Some(q) => q,
    None => return Ok(vec![]),
  };

  let rows = fetch_rows(&sql, &params).await?;
  let results = rows
    .iter()
    .map(|row| {
      let get_str = |key: &str| -> String {
        row
          .get(key)
          .and_then(|v| v.as_str())
          .map(|s| s.to_string())
          .unwrap_or_default()
      };
      let lab_items_code = row
        .get("lab_items_code")
        .map(|v| match v {
          serde_json::Value::String(s) => s.clone(),
          serde_json::Value::Number(n) => n.to_string(),
          _ => String::new(),
        })
        .unwrap_or_default();

      LabResult {
        hn: get_str("hn"),
        lab_items_code,
        lab_items_name: get_str("lab_items_name"),
        lab_order_result: get_str("lab_order_result"),
        order_date: get_str("order_date"),
      }
    })
    .collect();
  Ok(results)
}

// ---------------------------------------------------------------------------
// Export helpers
// ---------------------------------------------------------------------------

/// Try to load a Thai-capable TrueType font from common system paths.
/// Falls back to built-in Helvetica when no suitable font is available.
fn load_thai_font(doc: &printpdf::PdfDocumentReference) -> printpdf::IndirectFontRef {
  let font_paths = [
    // macOS
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
    "/Library/Fonts/Arial Unicode.ttf",
    // Windows
    "C:/Windows/Fonts/arialuni.ttf",
    "C:/Windows/Fonts/arial.ttf",
    // Linux
    "/usr/share/fonts/truetype/msttcorefonts/arial.ttf",
  ];
  for path in &font_paths {
    if let Ok(file) = std::fs::File::open(path) {
      if let Ok(font) = doc.add_external_font(file) {
        return font;
      }
    }
  }
  doc
    .add_builtin_font(printpdf::BuiltinFont::Helvetica)
    .unwrap()
}

/// Add a blank page to a PDF document and return (layer, y_start_mm).
fn pdf_add_page(doc: &printpdf::PdfDocumentReference) -> (printpdf::PdfLayerReference, f64) {
  let (page, layer) = doc.add_page(printpdf::Mm(210.0), printpdf::Mm(297.0), "Layer 1");
  (doc.get_page(page).get_layer(layer), 275.0)
}

// ---------------------------------------------------------------------------
// Export commands
// ---------------------------------------------------------------------------

/// Export patient records as a proper .xlsx workbook using rust_xlsxwriter.
#[tauri::command]
pub async fn cmd_export_excel(
  records: Vec<PatientRecord>,
  process_date: String,
  output_path: String,
) -> Result<String, String> {
  use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};

  if output_path.is_empty() {
    return Err("กรุณาระบุที่อยู่ไฟล์ส่งออก".into());
  }

  // Load app config to get drug abbreviations and course_days
  let app_cfg = crate::config::read_app_config().map_err(err_str)?;
  // Build a normalized lookup: lowercase trimmed name -> (abbr, capsules)
  // capsules = จำนวนยาต่อครั้ง (the "quantity" field shown in settings as จำนวน)
  let drug_info_norm: std::collections::HashMap<String, (String, i32)> = app_cfg
    .drugs
    .iter()
    .map(|d| {
      (
        d.drug_name.trim().to_lowercase(),
        (d.abbr.clone(), d.capsules),
      )
    })
    .collect();

  // Helper closure for fuzzy lookup
  let lookup = |name: &str| -> Option<(String, i32)> {
    let key = name.trim().to_lowercase();
    // Exact match first
    if let Some(v) = drug_info_norm.get(&key) {
      return Some(v.clone());
    }
    // Fuzzy: find first config drug where one is a prefix of the other
    drug_info_norm
      .iter()
      .find(|(k, _)| k.starts_with(&key) || key.starts_with(k.as_str()))
      .map(|(_, v)| v.clone())
  };

  let mut workbook = Workbook::new();
  let worksheet = workbook.add_worksheet();

  // ── Formats ────────────────────────────────────────────────────────────
  let title_fmt = Format::new()
    .set_bold()
    .set_font_size(13.0)
    .set_background_color(Color::RGB(0x9fe870))
    .set_font_color(Color::RGB(0x163300))
    .set_align(FormatAlign::Left)
    .set_border(FormatBorder::None);

  let header_fmt = Format::new()
    .set_bold()
    .set_font_size(11.0)
    .set_background_color(Color::RGB(0x1a1a1a))
    .set_font_color(Color::White)
    .set_align(FormatAlign::Center)
    .set_border(FormatBorder::Thin)
    .set_border_color(Color::RGB(0x444444));

  let data_fmt = Format::new()
    .set_font_size(11.0)
    .set_border(FormatBorder::Thin)
    .set_border_color(Color::RGB(0xdce8dc));

  let alt_fmt = Format::new()
    .set_font_size(11.0)
    .set_background_color(Color::RGB(0xf9fbf7))
    .set_border(FormatBorder::Thin)
    .set_border_color(Color::RGB(0xdce8dc));

  let drug_wrap_fmt = Format::new()
    .set_font_size(11.0)
    .set_font_color(Color::RGB(0x163300))
    .set_text_wrap()
    .set_border(FormatBorder::Thin)
    .set_border_color(Color::RGB(0xdce8dc));

  let drug_wrap_alt_fmt = Format::new()
    .set_font_size(11.0)
    .set_font_color(Color::RGB(0x163300))
    .set_background_color(Color::RGB(0xf9fbf7))
    .set_text_wrap()
    .set_border(FormatBorder::Thin)
    .set_border_color(Color::RGB(0xdce8dc));

  // ── Row 0: title ───────────────────────────────────────────────────────
  let title = format!("HerbReady — รายงานการจ่ายยาสมุนไพร  วันที่ {}", process_date);
  worksheet
    .merge_range(0, 0, 0, 7, title.as_str(), &title_fmt)
    .map_err(|e| e.to_string())?;

  // ── Row 1: column headers ──────────────────────────────────────────────
  let headers = [
    "HN",
    "เลขบัตรประชาชน",
    "ชื่อ-สกุล",
    "สิทธิ์",
    "น้ำหนัก (kg)",
    "BP",
    "ชีพจร",
    "ยาที่เลือกจ่าย",
  ];
  for (col, h) in headers.iter().enumerate() {
    worksheet
      .write_with_format(1, col as u16, *h, &header_fmt)
      .map_err(|e| e.to_string())?;
  }

  // ── Column widths & row heights ────────────────────────────────────────
  let widths: &[(u16, f64)] = &[
    (0, 10.0), // HN
    (1, 16.0), // CID
    (2, 26.0), // ชื่อ-สกุล
    (3, 12.0), // สิทธิ์
    (4, 11.0), // น้ำหนัก
    (5, 11.0), // BP
    (6, 9.0),  // ชีพจร
    (7, 52.0), // ยาที่เลือกจ่าย
  ];
  for &(col, width) in widths {
    worksheet
      .set_column_width(col, width)
      .map_err(|e| e.to_string())?;
  }
  // Title row height
  worksheet
    .set_row_height(0, 22.0)
    .map_err(|e| e.to_string())?;
  // Header row height
  worksheet
    .set_row_height(1, 18.0)
    .map_err(|e| e.to_string())?;

  // Freeze top 2 rows (title + header)
  worksheet
    .set_freeze_panes(2, 0)
    .map_err(|e| e.to_string())?;

  // ── Data rows (start at row 2) ─────────────────────────────────────────
  for (i, rec) in records.iter().enumerate() {
    let row = (i + 2) as u32;

    // Collect ALL drugs the user selected (eligible + never-dispensed).
    // drug_selection covers both categories — just check the bool flag.
    let mut selected_names: Vec<&str> = rec
      .drug_selection
      .iter()
      .filter(|(_, &v)| v)
      .map(|(name, _)| name.as_str())
      .collect();
    selected_names.sort_unstable();

    // Format each selected drug as "abbr(course_days)"
    let drug_parts: Vec<String> = selected_names
      .iter()
      .map(|&name| {
        if let Some((abbr, days)) = lookup(name) {
          let label = if abbr.is_empty() { name } else { abbr.as_str() };
          format!("{}({})", label, days)
        } else {
          name.to_string()
        }
      })
      .collect();
    let drugs_str = drug_parts.join(", ");

    let use_alt = i % 2 == 1;
    let (base, drug) = if use_alt {
      (&alt_fmt, &drug_wrap_alt_fmt)
    } else {
      (&data_fmt, &drug_wrap_fmt)
    };

    worksheet
      .set_row_height(row, 22.0)
      .map_err(|e| e.to_string())?;

    let cells: [(&str, bool); 8] = [
      (&rec.hn, false),
      (&rec.cid, false),
      (&rec.pt_name, false),
      (&rec.pttype_today, false),
      (&rec.last_weight, false),
      (&rec.last_blood_pressure, false),
      (&rec.last_pulse, false),
      (drugs_str.as_str(), true),
    ];

    for (col, (val, is_drug)) in cells.iter().enumerate() {
      let fmt = if *is_drug { drug } else { base };
      worksheet
        .write_with_format(row, col as u16, *val, fmt)
        .map_err(|e| e.to_string())?;
    }
  }

  workbook
    .save(&output_path)
    .map_err(|e| format!("ไม่สามารถบันทึกไฟล์ Excel: {}", e))?;

  Ok(format!(
    "ส่งออก Excel สำเร็จ: {} แถว → {}",
    records.len(),
    output_path
  ))
}

/// Export patient records as a proper .pdf report using printpdf.
#[tauri::command]
pub async fn cmd_export_pdf(
  records: Vec<PatientRecord>,
  process_date: String,
  output_path: String,
) -> Result<String, String> {
  use printpdf::*;
  use std::io::BufWriter;

  if output_path.is_empty() {
    return Err("กรุณาระบุที่อยู่ไฟล์ส่งออก".into());
  }

  let (doc, page1, layer1) = PdfDocument::new(
    format!("HerbReady {}", process_date),
    Mm(210.0),
    Mm(297.0),
    "Layer 1",
  );

  let font = load_thai_font(&doc);
  let mut layer = doc.get_page(page1).get_layer(layer1);
  let mut y = 275.0_f64;
  let x = 15.0_f64;

  // Document title
  layer.use_text(
    format!("HerbReady — รายงานการจ่ายยาสมุนไพร วันที่ {}", process_date),
    14.0,
    Mm(x),
    Mm(y),
    &font,
  );
  y -= 8.0;
  layer.use_text(
    format!("จำนวนผู้ป่วย: {} ราย", records.len()),
    10.0,
    Mm(x),
    Mm(y),
    &font,
  );
  y -= 12.0;

  for (i, rec) in records.iter().enumerate() {
    // Need at least 30 mm for a patient block
    if y < 40.0 {
      let (new_layer, new_y) = pdf_add_page(&doc);
      layer = new_layer;
      y = new_y;
    }

    // Patient name + HN
    let hn_str = if rec.hn.len() <= 7 {
      format!("{:0>7}", rec.hn)
    } else {
      rec.hn.clone()
    };
    layer.use_text(
      format!("{}. {} (HN: {})", i + 1, rec.pt_name, hn_str),
      11.0,
      Mm(x),
      Mm(y),
      &font,
    );
    y -= 6.0;

    // Info: CID / dept / rights
    let mut parts: Vec<String> = Vec::new();
    if !rec.cid.is_empty() && rec.cid != "0" {
      parts.push(format!("CID: {}", rec.cid));
    }
    if !rec.current_dept_name.is_empty() {
      parts.push(format!("หน่วยงาน: {}", rec.current_dept_name));
    }
    if !rec.pttype_today.is_empty() {
      parts.push(format!("สิทธิ์: {}", rec.pttype_today));
    }
    if !parts.is_empty() {
      if y < 20.0 {
        let (nl, ny) = pdf_add_page(&doc);
        layer = nl;
        y = ny;
      }
      layer.use_text(
        format!("   {}", parts.join("  |  ")),
        9.0,
        Mm(x),
        Mm(y),
        &font,
      );
      y -= 5.5;
    }

    // Vitals
    let vitals = format!(
      "   น้ำหนัก: {} kg  |  BP: {}  |  ชีพจร: {}",
      if rec.last_weight.is_empty() {
        "—"
      } else {
        &rec.last_weight
      },
      if rec.last_blood_pressure.is_empty() {
        "—"
      } else {
        &rec.last_blood_pressure
      },
      if rec.last_pulse.is_empty() {
        "—"
      } else {
        &rec.last_pulse
      },
    );
    if y < 20.0 {
      let (nl, ny) = pdf_add_page(&doc);
      layer = nl;
      y = ny;
    }
    layer.use_text(&vitals, 9.0, Mm(x), Mm(y), &font);
    y -= 5.5;

    // Selected drugs
    let selected: Vec<&str> = rec
      .eligible_drugs_raw
      .split(",")
      .map(|s| s.trim())
      .filter(|s| !s.is_empty() && rec.drug_selection.get(*s).copied().unwrap_or(false))
      .collect();
    if !selected.is_empty() {
      if y < 20.0 {
        let (nl, ny) = pdf_add_page(&doc);
        layer = nl;
        y = ny;
      }
      layer.use_text(
        format!("   ยาที่จ่าย: {}", selected.join(", ")),
        9.0,
        Mm(x),
        Mm(y),
        &font,
      );
      y -= 5.5;
    }

    // Gap between patients
    y -= 4.0;
  }

  let file = std::fs::File::create(&output_path).map_err(|e| format!("ไม่สามารถสร้างไฟล์: {}", e))?;
  doc
    .save(&mut BufWriter::new(file))
    .map_err(|e| format!("ไม่สามารถบันทึก PDF: {}", e))?;

  Ok(format!(
    "ส่งออก PDF สำเร็จ: {} ราย → {}",
    records.len(),
    output_path
  ))
}

// ---------------------------------------------------------------------------
// Dispensing history query (Tab 3 full listing)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn cmd_get_dispensing_history(
  date_from: String,
  date_to: String,
  hn: Option<String>,
  cid: Option<String>,
  name: Option<String>,
) -> Result<Vec<DispenseHistoryRecord>, String> {
  let app_cfg = crate::config::read_app_config().map_err(err_str)?;
  let (sql, params) = queries::build_dispensing_history_query(
    &date_from,
    &date_to,
    hn.as_deref(),
    cid.as_deref(),
    name.as_deref(),
    &app_cfg.drugs,
  );

  let rows = fetch_rows(&sql, &params).await?;
  let records = rows
    .iter()
    .map(|row| {
      let get = |k: &str| -> String {
        row
          .get(k)
          .and_then(|v| v.as_str())
          .map(|s| s.to_string())
          .unwrap_or_default()
      };
      let vstdate = row.get("vstdate").and_then(|v| {
        if v.is_null() {
          None
        } else {
          v.as_str().map(|s| s.to_string())
        }
      });
      let drug_count = row.get("drug_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
      DispenseHistoryRecord {
        vstdate,
        hn: get("hn"),
        cid: get("cid"),
        pt_name: get("pt_name"),
        drugs_dispensed: get("drugs_dispensed"),
        drug_count,
      }
    })
    .collect();
  Ok(records)
}

// ---------------------------------------------------------------------------
// Import / Export app_config.json
// ---------------------------------------------------------------------------

/// Export app_config.json content as a pretty-printed JSON string.
/// The frontend is responsible for saving it to a user-chosen file via the dialog plugin.
#[tauri::command]
pub async fn cmd_export_app_config() -> Result<String, String> {
  let cfg = crate::config::read_app_config().map_err(err_str)?;
  serde_json::to_string_pretty(&cfg).map_err(err_str)
}

/// Import app config from a JSON string.
/// The frontend reads the file via the dialog/fs plugin and passes the raw content here.
#[tauri::command]
pub async fn cmd_import_app_config(json_content: String) -> Result<AppConfig, String> {
  let cfg: AppConfig = serde_json::from_str(&json_content).map_err(err_str)?;
  crate::config::write_app_config(&cfg).map_err(err_str)?;
  Ok(cfg)
}

/// Check for Herb/Drug interaction alerts for a list of HNs on a given date.
///
/// Queries `opitemrece` to find any HN that has been prescribed one of the
/// configured modern drugs within the past year. Returns one alert per
/// (HN, matching interaction rule) pair.
#[tauri::command]
pub async fn cmd_check_herb_drug_interactions(
  process_date: String,
  hn_list: Vec<String>,
) -> Result<Vec<crate::models::HerbDrugInteractionAlert>, String> {
  let app_cfg = crate::config::read_app_config().map_err(err_str)?;
  if app_cfg.herb_drug_interactions.is_empty() || hn_list.is_empty() {
    return Ok(vec![]);
  }

  let query = queries::build_check_modern_drug_query(
    &process_date,
    &hn_list,
    &app_cfg.herb_drug_interactions,
  );
  let (sql, params) = match query {
    Some(q) => q,
    None => return Ok(vec![]),
  };

  let rows = fetch_rows(&sql, &params).await?;

  let mut alerts = Vec::new();
  for row in &rows {
    let hn = row
      .get("hn")
      .and_then(|v| v.as_str())
      .map(|s| s.to_string())
      .unwrap_or_default();
    let modern_icode = row
      .get("modern_icode")
      .map(|v| match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
      })
      .unwrap_or_default();

    for rule in &app_cfg.herb_drug_interactions {
      if rule.modern_drug_icode.trim() == modern_icode.trim() {
        alerts.push(crate::models::HerbDrugInteractionAlert {
          hn: hn.clone(),
          modern_drug_icode: rule.modern_drug_icode.clone(),
          modern_drug_name: rule.modern_drug_name.clone(),
          herb_drug_names: rule.herb_drugs.iter().map(|h| h.name.clone()).collect(),
          reason: rule.reason.clone(),
        });
      }
    }
  }
  Ok(alerts)
}
