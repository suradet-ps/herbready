// ─────────────────────────────────────────────────────────────────────────────
// HerbReady — app config (drugs + departments) composable
// ─────────────────────────────────────────────────────────────────────────────
import { ref } from "vue";
import type { AppConfig } from "../types";
import { api } from "../api/tauri";

// Reactive singleton
export const appConfig = ref<AppConfig>({
  drugs: [],
  departments: [],
  lab_rules: [],
  herb_drug_interactions: [],
});

/**
 * Load application config (drugs, departments) from Rust backend.
 */
export async function loadAppConfig(): Promise<void> {
  try {
    const cfg = await api.getAppConfig();
    appConfig.value = cfg;
  } catch (err: unknown) {
    console.error("loadAppConfig failed:", err);
  }
}

/**
 * Persist updated application config to Rust backend.
 *
 * Ensure every drug entry has the `enabled` property present (default true)
 * before sending to the backend so disabled state persists across restarts.
 */
export async function saveAppConfig(cfg: AppConfig): Promise<void> {
  // Normalise drugs to always include `enabled` (default true unless explicitly false).
  const cfgWithEnabled: AppConfig = {
    drugs: cfg.drugs.map((d) => ({
      ...d,
      enabled: d.enabled === false ? false : true,
    })),
    departments: cfg.departments.map((d) => ({ ...d })),
    lab_rules: (cfg.lab_rules ?? []).map((r) => ({ ...r })),
    herb_drug_interactions: (cfg.herb_drug_interactions ?? []).map((r) => ({
      ...r,
      herb_drugs: (r.herb_drugs ?? []).map((h) => ({ ...h })),
    })),
  };

  await api.saveAppConfig(cfgWithEnabled);
  appConfig.value = cfgWithEnabled;
}

/**
 * Lookup abbreviated name (abbr) for a full drug name.
 * Normalises whitespace and compares case-insensitively.
 * Falls back to the full drug name (no truncation) if no abbr configured.
 */
export function getAbbrByName(drugName: string): string {
  const normalise = (s: string) => s.trim().replace(/\s+/g, " ").toLowerCase();
  const key = normalise(drugName);

  // 1. Exact match
  let found = appConfig.value.drugs.find((d) => normalise(d.drug_name) === key);

  // 2. Partial match: config name starts with key OR key starts with config name
  if (!found) {
    found = appConfig.value.drugs.find((d) => {
      const cn = normalise(d.drug_name);
      return cn.startsWith(key) || key.startsWith(cn);
    });
  }

  // 3. Substring match (key contains config name or vice versa)
  if (!found) {
    found = appConfig.value.drugs.find((d) => {
      const cn = normalise(d.drug_name);
      return cn.includes(key) || key.includes(cn);
    });
  }

  if (found && found.abbr && found.abbr.trim() !== "") return found.abbr.trim();
  // No match — return full name
  return drugName.trim();
}

/**
 * Return true when a drug name matches a config entry whose enabled flag is
 * explicitly set to false.  Uses the same fuzzy matching as getAbbrByName.
 */
export function isDisabledDrug(drugName: string): boolean {
  const normalise = (s: string) => s.trim().replace(/\s+/g, " ").toLowerCase();
  const key = normalise(drugName);
  return appConfig.value.drugs.some((d) => {
    if (d.enabled !== false) return false;
    const cn = normalise(d.drug_name);
    return (
      cn === key ||
      cn.startsWith(key) ||
      key.startsWith(cn) ||
      cn.includes(key) ||
      key.includes(cn)
    );
  });
}
