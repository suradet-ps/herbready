//! lib.rs — HerbReady Tauri application entry point.
//!
//! Registers all plugins, commands, and modules then launches the Tauri runtime.

pub mod commands;
pub mod config;
pub mod crypto;
pub mod db;
pub mod models;
pub mod queries;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // Initialise the logger (uses RUST_LOG env var; defaults to info in release)
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![
      // ── Database connection ──────────────────────────────────────
      cmd_test_connection,
      cmd_connect_db,
      // ── Configuration ────────────────────────────────────────────
      cmd_get_db_config,
      cmd_save_db_config,
      cmd_get_app_config,
      cmd_save_app_config,
      cmd_export_app_config,
      cmd_import_app_config,
      // ── Daily tab ────────────────────────────────────────────────
      cmd_get_daily_records,
      // ── Search tab ───────────────────────────────────────────────
      cmd_search_patient,
      // ── History tab ──────────────────────────────────────────────
      cmd_get_patient_history,
      cmd_search_patient_name_for_history,
      cmd_find_patients_by_name, // patient name lookup
      cmd_find_patient_by_id,    // patient lookup by HN or CID
      cmd_get_dispensing_history,
      // ── Drug / dept lookups ──────────────────────────────────────
      cmd_lookup_drug_name,
      cmd_lookup_dept_name,
      // ── Lab results ───────────────────────────────────────────────
      cmd_lookup_lab_item_name,
      cmd_get_lab_results,
      cmd_check_herb_drug_interactions,
      // ── Export ───────────────────────────────────────────────────
      cmd_export_excel,
      cmd_export_pdf,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
