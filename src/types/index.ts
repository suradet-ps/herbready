// ─────────────────────────────────────────────────────────────────────────────
// HerbReady — shared TypeScript types
// ─────────────────────────────────────────────────────────────────────────────

export interface PatientRecord {
  vn: string;
  hn: string;
  cid: string;
  pt_name: string;
  current_dept_name: string;
  pttype_today: string;
  last_visit_date: string | null;
  last_weight: string;
  last_blood_pressure: string;
  last_pulse: string;
  eligible_drugs_raw: string;
  never_dispensed_drugs_raw: string;
  not_yet_eligible_drugs_raw: string;
  print_selected: boolean;
  drug_selection: Record<string, boolean>;
}

export interface DrugItem {
  drug_name: string;
  days_remaining: number | null;
  last_dispense_date?: string;
}

export interface DrugConfig {
  icode: string;
  abbr: string;
  course_days: number;
  capsules: number;
  drug_name: string;
  enabled?: boolean;
}

export interface DeptConfig {
  code: string;
  name: string;
}

export interface HerbDrugEntry {
  icode: string;
  name: string;
}

export interface HerbDrugInteraction {
  modern_drug_icode: string;
  modern_drug_name: string;
  herb_drugs: HerbDrugEntry[];
  reason: string;
}

export interface HerbDrugInteractionAlert {
  hn: string;
  modern_drug_icode: string;
  modern_drug_name: string;
  herb_drug_names: string[];
  reason: string;
}

export interface LabRuleConfig {
  lab_items_code: string;
  lab_items_name: string;
  threshold: number;
  compare_gt: boolean;
  compare_eq: boolean;
  compare_lt: boolean;
}

export interface LabResult {
  hn: string;
  lab_items_code: string;
  lab_items_name: string;
  lab_order_result: string;
  order_date: string;
}

export interface AppConfig {
  drugs: DrugConfig[];
  departments: DeptConfig[];
  lab_rules: LabRuleConfig[];
  herb_drug_interactions: HerbDrugInteraction[];
}

export interface DatabaseConfig {
  host: string;
  port: number;
  name: string;
  user: string;
  password: string;
}

export interface DrugDispenseItem {
  vstdate: string | null;
  drug_name: string;
  qty: string;
  units: string;
}

export type ConnectionStatus =
  | "disconnected"
  | "connecting"
  | "connected"
  | "error";
