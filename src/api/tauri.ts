// ─────────────────────────────────────────────────────────────────────────────
// HerbReady — Tauri invoke wrappers
// ─────────────────────────────────────────────────────────────────────────────
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  AppConfig,
  DatabaseConfig,
  DrugDispenseItem,
  HerbDrugInteractionAlert,
  PatientRecord,
} from "../types";

export const api = {
  // ── Database connection ────────────────────────────────────────────────────
  testConnection: (
    host: string,
    port: number,
    dbname: string,
    user: string,
    password: string,
  ) =>
    invoke<string>("cmd_test_connection", {
      host,
      port,
      dbname,
      user,
      password,
    }),

  connectDb: (
    host: string,
    port: number,
    dbname: string,
    user: string,
    password: string,
  ) => invoke<string>("cmd_connect_db", { host, port, dbname, user, password }),

  // ── Config persistence ─────────────────────────────────────────────────────
  getDbConfig: () => invoke<DatabaseConfig>("cmd_get_db_config"),

  saveDbConfig: (
    host: string,
    port: number,
    name: string,
    user: string,
    password: string,
  ) => invoke<void>("cmd_save_db_config", { host, port, name, user, password }),

  getAppConfig: () => invoke<AppConfig>("cmd_get_app_config"),

  // ── Lab results ────────────────────────────────────────────────────────────
  lookupLabItemName: (code: string) =>
    invoke<string>("cmd_lookup_lab_item_name", { code }),

  getLabResults: (processDate: string, hnList: string[]) =>
    invoke<import("../types").LabResult[]>("cmd_get_lab_results", {
      processDate,
      hnList,
    }),

  checkHerbDrugInteractions: (processDate: string, hnList: string[]) =>
    invoke<HerbDrugInteractionAlert[]>("cmd_check_herb_drug_interactions", {
      processDate,
      hnList,
    }),

  saveAppConfig: (config: AppConfig) =>
    invoke<void>("cmd_save_app_config", { config }),

  // ── Patient data ───────────────────────────────────────────────────────────
  getDailyRecords: (processDate: string, vitalsOnDate: boolean = false) =>
    invoke<PatientRecord[]>("cmd_get_daily_records", {
      processDate,
      vitalsOnDate,
    }),

  searchPatient: (processDate: string, searchText: string) =>
    invoke<PatientRecord[]>("cmd_search_patient", { processDate, searchText }),

  getPatientHistory: (hn: string, yearsBack: number | null) =>
    invoke<DrugDispenseItem[]>("cmd_get_patient_history", { hn, yearsBack }),

  /**
   * Search history by patient name.
   * Returns tuple [items, resolved_hn, resolved_name].
   */
  searchPatientNameForHistory: (name: string, yearsBack: number | null) =>
    invoke<[DrugDispenseItem[], string, string]>(
      "cmd_search_patient_name_for_history",
      {
        name,
        yearsBack,
      },
    ),
  /**
   * Find all patients whose name contains the given text.
   * Returns up to 50 lightweight matches (hn, cid, pt_name).
   */
  findPatientsByName: (name: string) =>
    invoke<
      Array<{ hn: string; cid: string; pt_name: string; pttype_name: string }>
    >("cmd_find_patients_by_name", { name }),

  /**
   * Look up a single patient by HN (5-9 digits) or CID (13 digits).
   * Returns null if not found.
   */
  findPatientById: (id: string) =>
    invoke<{
      hn: string;
      cid: string;
      pt_name: string;
      pttype_name: string;
    } | null>("cmd_find_patient_by_id", { id }),

  // ── Lookup helpers ─────────────────────────────────────────────────────────
  lookupDrugName: (icode: string) =>
    invoke<string>("cmd_lookup_drug_name", { icode }),

  lookupDeptName: (code: string) =>
    invoke<string>("cmd_lookup_dept_name", { code }),

  // ── Exports ────────────────────────────────────────────────────────────────
  exportExcel: (
    records: PatientRecord[],
    processDate: string,
    outputPath: string,
  ) => invoke<string>("cmd_export_excel", { records, processDate, outputPath }),

  exportPdf: (
    records: PatientRecord[],
    processDate: string,
    outputPath: string,
  ) => invoke<string>("cmd_export_pdf", { records, processDate, outputPath }),

  // ── Dialog helpers ─────────────────────────────────────────────────────────
  saveDialog: (
    defaultPath: string,
    filters: Array<{ name: string; extensions: string[] }>,
  ) => save({ defaultPath, filters }),

  openDialog: (filters: Array<{ name: string; extensions: string[] }>) =>
    open({ filters, multiple: false }),

  // ── App config import/export ───────────────────────────────────────────────
  exportAppConfig: () => invoke<string>("cmd_export_app_config"),

  importAppConfig: (jsonContent: string) =>
    invoke<AppConfig>("cmd_import_app_config", { jsonContent }),
};
