<script setup lang="ts">
/**
 * SearchTab — ค้นหารายบุคคล
 * Search patient by HN / CID / name and show drug eligibility.
 */
import { ref, computed } from "vue";
import {
    Search,
    X,
    User,
    Activity,
    Pill,
    Clock,
    ChevronRight,
    Calendar,
    FlaskConical,
} from "lucide-vue-next";
import type { PatientRecord, DrugDispenseItem, LabResult } from "../types";
import { api } from "../api/tauri";
import {
    parseSimpleDrugList,
    parseNotYetDrugList,
    thaiSortKey,
    hnPadded,
} from "../utils/drugParser";
import { formatDateThaiShort, todayISO } from "../utils/dateHelper";
import { appConfig, getAbbrByName, isDisabledDrug } from "../stores/appConfig";
import DrugPill from "./DrugPill.vue";

const emit = defineEmits<{
    (e: "patient-selected", record: PatientRecord): void;
    (e: "view-history", record: PatientRecord): void;
}>();

// ── state ─────────────────────────────────────────────
const searchText = ref("");
const loading = ref(false);
const results = ref<PatientRecord[]>([]);
const selectedRecord = ref<PatientRecord | null>(null);
const viewingPatient = ref(false);
const hint = ref("พิมพ์ HN / เลขบัตรประชาชน / ชื่อ-นามสกุล แล้วกด Enter");
const hintColor = ref<"muted" | "primary" | "error" | "amber">("muted");
const processDate = ref(todayISO());

// Lab results for the currently selected patient
const patientLabResults = ref<LabResult[]>([]);

// ── computed ────────────────────────────────────────────
const searchHint = computed(() => {
    const t = searchText.value.trim();
    if (!t) return "";
    if (/^\d+$/.test(t)) {
        if (t.length === 13) return "→ เลขบัตรประชาชน";
        if (t.length >= 5 && t.length <= 9) return "→ HN";
        return "→ ตัวเลข";
    }
    return "→ ชื่อ-นามสกุล";
});

const processDateLabel = computed(() => formatDateThaiShort(processDate.value));

function drugsSorted<T extends { drug_name: string }>(list: T[]): T[] {
    return [...list].sort((a, b) => {
        const ka = thaiSortKey(getAbbrByName(a.drug_name) || a.drug_name);
        const kb = thaiSortKey(getAbbrByName(b.drug_name) || b.drug_name);
        return ka < kb ? -1 : ka > kb ? 1 : 0;
    });
}

const eligibleDrugs = computed(() =>
    selectedRecord.value
        ? drugsSorted(
              parseSimpleDrugList(
                  selectedRecord.value.eligible_drugs_raw,
              ).filter((item) => !isDisabledDrug(item.drug_name)),
          )
        : [],
);
const neverDrugs = computed(() =>
    selectedRecord.value
        ? drugsSorted(
              parseSimpleDrugList(
                  selectedRecord.value.never_dispensed_drugs_raw,
              ).filter((item) => !isDisabledDrug(item.drug_name)),
          )
        : [],
);
const notYetDrugs = computed(() =>
    selectedRecord.value
        ? drugsSorted(
              parseNotYetDrugList(
                  selectedRecord.value.not_yet_eligible_drugs_raw,
              ).filter((item) => !isDisabledDrug(item.drug_name)),
          )
        : [],
);

// patientHistory stores dispense history items for the selected patient.
// We'll populate this when a record is selected so the Search tab can
// enrich the not-yet list with the last dispense date where available.
const patientHistory = ref<DrugDispenseItem[]>([]);

// notYetEnriched: take the parsed not-yet items and, when possible, attach
// a last_dispense_date pulled from the patient's history (matching by drug_name).
// This keeps the tooltip consistent with Daily tab where last-dispense info exists.
const notYetEnriched = computed(() => {
    // copy items so we don't mutate originals
    const base = notYetDrugs.value.map((it) => ({ ...it }));
    // build a map of latest vstdate by normalized drug name
    const latestMap = new Map<string, string>();
    for (const h of patientHistory.value) {
        if (!h.drug_name || !h.vstdate) continue;
        const key = h.drug_name.trim().toLowerCase();
        const prev = latestMap.get(key);
        // keep the latest date string (ISO comparison is safe)
        if (!prev || h.vstdate > prev) latestMap.set(key, h.vstdate);
    }
    return base.map((it) => {
        const key = it.drug_name.trim().toLowerCase();
        if (!it.last_dispense_date && latestMap.has(key)) {
            return { ...it, last_dispense_date: latestMap.get(key) };
        }
        return it;
    });
});

// ── missing config drugs (bug fix) ───────────────────
function normKey(s: string): string {
    return s.trim().replace(/\s+/g, " ").toLowerCase();
}
function isDrugRepresented(
    configName: string,
    patientNames: string[],
): boolean {
    const cn = normKey(configName);
    return patientNames.some((pn) => {
        const pk = normKey(pn);
        return (
            cn.startsWith(pk) ||
            pk.startsWith(cn) ||
            cn.includes(pk) ||
            pk.includes(cn)
        );
    });
}
const allPatientDrugNames = computed(() => [
    ...(eligibleDrugs.value ?? []).map((d) => d.drug_name),
    ...(neverDrugs.value ?? []).map((d) => d.drug_name),
    ...(notYetDrugs.value ?? []).map((d) => d.drug_name),
]);
const missingConfigDrugs = computed(() =>
    selectedRecord.value
        ? appConfig.value.drugs
              .filter(
                  (cfg) =>
                      cfg.enabled !== false &&
                      !isDrugRepresented(
                          cfg.drug_name,
                          allPatientDrugNames.value,
                      ),
              )
              .map((cfg) => ({
                  drug_name: cfg.drug_name,
                  days_remaining: null as null,
              }))
        : [],
);
const allNeverDrugs = computed(() => [
    ...(neverDrugs.value ?? []),
    ...(missingConfigDrugs.value ?? []),
]);

// ── ready date helper ─────────────────────────────────
function computeReadyDate(daysRemaining: number | null): string | undefined {
    if (daysRemaining == null) return undefined;
    const base = new Date(processDate.value + "T00:00:00");
    base.setDate(base.getDate() + daysRemaining);
    return base.toISOString().split("T")[0];
}

// ── methods ───────────────────────────────────────────
async function doSearch() {
    const t = searchText.value.trim();
    if (!t) return;

    loading.value = true;
    hint.value = "กำลังค้นหา…";
    hintColor.value = "muted";
    selectedRecord.value = null;
    viewingPatient.value = false;
    results.value = [];

    try {
        const rows = await api.searchPatient(processDate.value, t);
        if (rows.length === 0) {
            hint.value = "🔍 ไม่พบข้อมูลผู้ป่วย";
            hintColor.value = "amber";
        } else if (rows.length === 1) {
            selectRecord(rows[0]);
            hint.value = `พบผู้ป่วย: ${rows[0].pt_name}  (HN: ${hnPadded(rows[0].hn)})`;
            hintColor.value = "primary";
        } else {
            results.value = rows;
            hint.value = `พบ ${rows.length} รายการ — กรุณาเลือกผู้ป่วย`;
            hintColor.value = "primary";
        }
    } catch (err: unknown) {
        hint.value = `❌ ${err instanceof Error ? err.message : String(err)}`;
        hintColor.value = "error";
    } finally {
        loading.value = false;
    }
}

async function selectRecord(rec: PatientRecord) {
    selectedRecord.value = rec;
    viewingPatient.value = true;
    patientLabResults.value = [];
    emit("patient-selected", rec);

    // Load patient history so we can enrich not-yet items with last dispense dates.
    try {
        const history = await api.getPatientHistory(rec.hn, null);
        patientHistory.value = Array.isArray(history) ? history : [];
    } catch {
        patientHistory.value = [];
    }

    // Load latest lab results for this patient (non-blocking)
    if (appConfig.value.lab_rules.length > 0) {
        try {
            const results = await api.getLabResults(processDate.value, [
                rec.hn,
            ]);
            patientLabResults.value = Array.isArray(results) ? results : [];
        } catch {
            patientLabResults.value = [];
        }
    }
}

function backToResults() {
    selectedRecord.value = null;
    viewingPatient.value = false;
}

function clearSearch() {
    searchText.value = "";
    selectedRecord.value = null;
    results.value = [];
    viewingPatient.value = false;
    patientLabResults.value = [];
    hint.value = "พิมพ์ HN / เลขบัตรประชาชน / ชื่อ-นามสกุล แล้วกด Enter";
    hintColor.value = "muted";
}

// ── Lab result helpers ─────────────────────────────────────────────────────

/**
 * For each configured lab rule, find the latest result from patientLabResults
 * (which is already the latest per code from the backend).
 * Returns an array of display items — one per configured rule.
 */
interface LabDisplayItem {
    lab_items_code: string;
    lab_items_name: string;
    result: string | null; // null = no record found
    order_date: string | null;
    is_abnormal: boolean;
}

const labDisplayItems = computed((): LabDisplayItem[] => {
    return appConfig.value.lab_rules.map((rule) => {
        const found = patientLabResults.value.find(
            (r) => String(r.lab_items_code) === String(rule.lab_items_code),
        );
        if (!found) {
            return {
                lab_items_code: rule.lab_items_code,
                lab_items_name: rule.lab_items_name,
                result: null,
                order_date: null,
                is_abnormal: false,
            };
        }
        const val = parseFloat(found.lab_order_result);
        let is_abnormal = false;
        if (!isNaN(val)) {
            if (rule.compare_gt && val > rule.threshold) is_abnormal = true;
            if (rule.compare_lt && val < rule.threshold) is_abnormal = true;
            if (rule.compare_eq && Math.abs(val - rule.threshold) < 0.001)
                is_abnormal = true;
        }
        return {
            lab_items_code: rule.lab_items_code,
            lab_items_name: found.lab_items_name || rule.lab_items_name,
            result: found.lab_order_result,
            order_date: found.order_date,
            is_abnormal,
        };
    });
});

// debug helper removed

function viewHistory() {
    if (selectedRecord.value) emit("view-history", selectedRecord.value);
}
</script>

<template>
    <div class="search-tab">
        <!-- ── Toolbar ────────────────────────────────────────── -->
        <div class="search-tab__toolbar">
            <!-- Date picker -->
            <div class="search-tab__date-wrap">
                <Calendar :size="14" class="search-tab__date-icon" />
                <span class="search-tab__date-label">{{
                    processDateLabel
                }}</span>
                <input
                    v-model="processDate"
                    type="date"
                    class="search-tab__date-input"
                    title="เลือกวันที่ประเมินสิทธิ์ยา"
                />
            </div>

            <!-- Search field -->
            <div class="search-tab__field-wrap">
                <span class="search-tab__field-icon"
                    ><Search :size="15"
                /></span>
                <input
                    v-model="searchText"
                    class="search-tab__input"
                    type="text"
                    placeholder="HN  /  เลขบัตรประชาชน 13 หลัก  /  ชื่อ-นามสกุล"
                    @keydown.enter="doSearch"
                    :disabled="loading"
                    autocomplete="off"
                    spellcheck="false"
                />
                <span v-if="searchHint" class="search-tab__field-hint">{{
                    searchHint
                }}</span>
                <button
                    v-if="searchText"
                    class="search-tab__clear-btn"
                    @click="clearSearch"
                    type="button"
                    title="ล้าง"
                >
                    <X :size="13" />
                </button>
            </div>

            <button
                class="btn btn--primary"
                :disabled="loading || !searchText.trim()"
                @click="doSearch"
                type="button"
            >
                <span v-if="loading" class="btn-spinner" />
                <Search v-else :size="14" />
                {{ loading ? "กำลังค้นหา…" : "ค้นหา" }}
            </button>
        </div>

        <!-- ── Hint bar ──────────────────────────────────────── -->
        <div class="search-tab__hint" :class="`hint--${hintColor}`">
            {{ hint }}
        </div>

        <!-- ── Multiple results list ─────────────────────────── -->
        <div
            v-if="results.length > 1 && !viewingPatient"
            class="search-tab__results"
        >
            <div
                v-for="rec in results"
                :key="rec.vn || rec.hn"
                class="result-row"
                @click="selectRecord(rec)"
            >
                <User :size="15" class="result-row__icon" />
                <span class="result-row__hn">{{ hnPadded(rec.hn) }}</span>
                <span class="result-row__name">{{ rec.pt_name }}</span>
                <span class="result-row__dept">{{
                    rec.current_dept_name
                }}</span>
                <ChevronRight :size="13" class="result-row__arrow" />
            </div>
        </div>

        <!-- ── Patient card ─────────────────────────────────────── -->
        <div v-if="selectedRecord && viewingPatient" class="patient-card">
            <!-- Header -->
            <div class="patient-card__header">
                <div class="patient-card__avatar-wrap">
                    <User :size="22" class="patient-card__avatar" />
                </div>
                <div class="patient-card__main">
                    <div class="patient-card__name">
                        {{ selectedRecord.pt_name }}
                    </div>
                    <div class="patient-card__meta">
                        <span class="chip"
                            >HN: {{ hnPadded(selectedRecord.hn) }}</span
                        >
                        <span
                            v-if="
                                selectedRecord.cid && selectedRecord.cid !== '0'
                            "
                            class="chip"
                            >CID: {{ selectedRecord.cid }}</span
                        >
                        <span
                            v-if="selectedRecord.current_dept_name"
                            class="chip"
                            >{{ selectedRecord.current_dept_name }}</span
                        >
                        <span v-if="selectedRecord.pttype_today" class="chip">
                            สิทธิ์: {{ selectedRecord.pttype_today }}
                        </span>
                    </div>
                </div>
                <div class="patient-card__actions">
                    <button
                        v-if="results.length > 0"
                        class="btn btn--ghost btn--sm"
                        @click="backToResults"
                        type="button"
                        title="กลับไปยังรายการผลการค้นหา"
                    >
                        ← กลับ
                    </button>
                    <button
                        class="btn btn--outline btn--sm"
                        @click="viewHistory"
                        type="button"
                        title="ดูประวัติรับยาสมุนไพร"
                    >
                        <Clock :size="13" />
                        ประวัติรับยาสมุนไพร
                    </button>
                </div>
            </div>

            <!-- Vitals -->
            <div class="patient-card__vitals">
                <div v-if="selectedRecord.last_visit_date" class="vital-chip">
                    <Clock :size="12" />
                    <span class="vital-chip__label">ล่าสุด</span>
                    <span class="vital-chip__val">{{
                        formatDateThaiShort(selectedRecord.last_visit_date)
                    }}</span>
                </div>
                <div class="vital-chip">
                    <Activity :size="12" />
                    <span class="vital-chip__label">BP</span>
                    <span class="vital-chip__val">{{
                        selectedRecord.last_blood_pressure || "—"
                    }}</span>
                </div>
                <div class="vital-chip">
                    <Activity :size="12" />
                    <span class="vital-chip__label">Pulse</span>
                    <span class="vital-chip__val">{{
                        selectedRecord.last_pulse || "—"
                    }}</span>
                </div>
                <div class="vital-chip">
                    <Activity :size="12" />
                    <span class="vital-chip__label">Weight</span>
                    <span class="vital-chip__val"
                        >{{ selectedRecord.last_weight || "—" }} kg</span
                    >
                </div>
            </div>

            <!-- Drug sections -->
            <div class="patient-card__drugs">
                <!-- Eligible -->
                <div v-if="eligibleDrugs.length > 0" class="drug-section">
                    <div
                        class="drug-section__header drug-section__header--green"
                    >
                        <div class="drug-section__title">
                            <Pill :size="13" />
                            ยาที่จ่ายได้
                        </div>
                        <span
                            class="drug-section__count drug-section__count--green"
                            >{{ eligibleDrugs.length }}</span
                        >
                    </div>
                    <div class="drug-section__pills">
                        <DrugPill
                            v-for="item in eligibleDrugs"
                            :key="item.drug_name"
                            :drug-name="item.drug_name"
                            :abbr="getAbbrByName(item.drug_name)"
                            status="eligible"
                        />
                    </div>
                </div>

                <!-- Never dispensed -->
                <div v-if="allNeverDrugs.length > 0" class="drug-section">
                    <div
                        class="drug-section__header drug-section__header--gray"
                    >
                        <div class="drug-section__title">
                            <Pill :size="13" />
                            ยาที่ยังไม่เคยจ่าย
                        </div>
                        <span
                            class="drug-section__count drug-section__count--gray"
                            >{{ allNeverDrugs.length }}</span
                        >
                    </div>
                    <div class="drug-section__pills">
                        <DrugPill
                            v-for="item in drugsSorted(allNeverDrugs)"
                            :key="item.drug_name"
                            :drug-name="item.drug_name"
                            :abbr="getAbbrByName(item.drug_name)"
                            status="never"
                        />
                    </div>
                </div>

                <!-- Not yet eligible -->
                <div v-if="notYetDrugs.length > 0" class="drug-section">
                    <div class="drug-section__header drug-section__header--red">
                        <div class="drug-section__title">
                            <Clock :size="13" />
                            ยาที่ยังไม่ถึงกำหนดจ่าย
                        </div>
                        <span
                            class="drug-section__count drug-section__count--red"
                            >{{ notYetDrugs.length }}</span
                        >
                    </div>
                    <div class="drug-section__pills">
                        <DrugPill
                            v-for="item in notYetEnriched"
                            :key="item.drug_name"
                            :drug-name="item.drug_name"
                            :abbr="getAbbrByName(item.drug_name)"
                            status="not_yet"
                            :days-remaining="item.days_remaining"
                            :last-dispense-date="item.last_dispense_date"
                            :ready-date="computeReadyDate(item.days_remaining)"
                        />
                    </div>
                </div>

                <!-- Empty state -->
                <div
                    v-if="
                        eligibleDrugs.length === 0 &&
                        allNeverDrugs.length === 0 &&
                        notYetDrugs.length === 0
                    "
                    class="drug-section__empty"
                >
                    ไม่พบข้อมูลยาสมุนไพรสำหรับผู้ป่วยรายนี้
                </div>
            </div>

            <!-- ── Lab Results Section ─────────────────────────────────── -->
            <div
                v-if="appConfig.lab_rules.length > 0"
                class="patient-card__labs"
            >
                <div class="lab-section__header">
                    <FlaskConical :size="13" />
                    <span>ผลแลปล่าสุด</span>
                </div>
                <div class="lab-section__grid">
                    <div
                        v-for="item in labDisplayItems"
                        :key="item.lab_items_code"
                        class="lab-item"
                        :class="{ 'lab-item--abnormal': item.is_abnormal }"
                    >
                        <span class="lab-item__name">{{
                            item.lab_items_name
                        }}</span>
                        <span
                            v-if="item.result !== null"
                            class="lab-item__result"
                            :class="{
                                'lab-item__result--abnormal': item.is_abnormal,
                            }"
                        >
                            {{ item.result }}
                            <span class="lab-item__date"
                                >({{ item.order_date }})</span
                            >
                        </span>
                        <span v-else class="lab-item__not-found">
                            ไม่พบการตรวจ
                        </span>
                    </div>
                </div>
            </div>
        </div>

        <!-- ── Empty state ──────────────────────────────────────── -->
        <div
            v-if="!selectedRecord && results.length === 0 && !loading"
            class="search-tab__empty"
        >
            <Search :size="40" class="search-tab__empty-icon" />
            <p>ค้นหาผู้ป่วยเพื่อดูข้อมูลสิทธิ์ยาสมุนไพร</p>
        </div>
    </div>
</template>

<style scoped>
/* ── Layout ────────────────────────────────────────────────── */
.search-tab {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 16px;
    height: 100%;
    overflow: hidden;
    background: #f5f7f3;
}

/* ── Toolbar ──────────────────────────────────────────────── */
.search-tab__toolbar {
    display: flex;
    gap: 8px;
    align-items: center;
    background: #ffffff;
    border-bottom: 1px solid rgba(14, 15, 12, 0.1);
    padding: 10px 16px;
    margin: -14px -16px 0;
    flex-wrap: wrap;
}

/* Date picker */
.search-tab__date-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: #f5f7f3;
    border: 1px solid rgba(14, 15, 12, 0.12);
    border-radius: 9999px;
    padding: 4px 10px;
    cursor: pointer;
    flex-shrink: 0;
}

.search-tab__date-icon {
    color: #163300;
    flex-shrink: 0;
}

.search-tab__date-label {
    font-size: 12px;
    font-weight: 600;
    color: #0e0f0c;
    white-space: nowrap;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    pointer-events: none;
    min-width: 80px;
}

.search-tab__date-input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
    width: 100%;
    height: 100%;
    border: none;
    background: none;
}

/* Search field */
.search-tab__field-wrap {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 200px;
    border: 1px solid rgba(14, 15, 12, 0.15);
    border-radius: 9999px;
    background: #ffffff;
    transition: border-color 0.15s;
}
.search-tab__field-wrap:focus-within {
    border-color: #9fe870;
}
.search-tab__field-wrap .search-tab__input {
    border: none;
    box-shadow: none;
    background: transparent;
}
.search-tab__field-wrap .search-tab__input:focus {
    border: none;
    box-shadow: none;
}

.search-tab__field-icon {
    position: absolute;
    left: 10px;
    color: #a39e98;
    pointer-events: none;
    display: flex;
}

.search-tab__input {
    width: 100%;
    height: 36px;
    padding: 0 88px 0 34px;
    border: 1px solid rgba(14, 15, 12, 0.15);
    border-radius: 9999px;
    background: #ffffff;
    color: rgba(0, 0, 0, 0.9);
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    font-size: 13px;
    outline: none;
    transition: border-color 0.15s;
}
.search-tab__input::placeholder {
    color: #a39e98;
}
.search-tab__input:focus {
    border-color: #9fe870;
    box-shadow: 0 0 0 2px rgba(159, 232, 112, 0.25);
}
.search-tab__input:disabled {
    opacity: 0.6;
}

.search-tab__field-hint {
    position: absolute;
    right: 36px;
    font-size: 11px;
    color: #a39e98;
    pointer-events: none;
}

.search-tab__clear-btn {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    cursor: pointer;
    color: #a39e98;
    display: flex;
    align-items: center;
    padding: 2px;
    border-radius: 4px;
}
.search-tab__clear-btn:hover {
    color: rgba(0, 0, 0, 0.95);
}

/* ── Hint bar ─────────────────────────────────────────────── */
.search-tab__hint {
    font-size: 12px;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    padding: 2px 2px;
    min-height: 18px;
    flex-shrink: 0;
}
.hint--muted {
    color: #a39e98;
}
.hint--primary {
    color: #163300;
}
.hint--error {
    color: #ef4444;
}
.hint--amber {
    color: #dd5b00;
}

/* ── Results list ───────────────────────────────────────────── */
.search-tab__results {
    border: 1px solid rgba(14, 15, 12, 0.12);
    border-radius: 16px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    background: #ffffff;
    box-shadow: rgba(14, 15, 12, 0.12) 0px 0px 0px 1px;
}
.result-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 14px;
    cursor: pointer;
    font-size: 13px;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    border-bottom: 1px solid rgba(14, 15, 12, 0.08);
    transition: background 0.1s;
}
.result-row:last-child {
    border-bottom: none;
}
.result-row:hover {
    background: #f0f7eb;
}
.result-row__icon {
    color: #a39e98;
    flex-shrink: 0;
}
.result-row__hn {
    color: #868685;
    font-weight: 600;
    min-width: 75px;
    font-size: 12px;
}
.result-row__name {
    flex: 1;
    color: #0e0f0c;
    font-weight: 600;
}
.result-row__dept {
    color: #a39e98;
    font-size: 12px;
}
.result-row__arrow {
    color: #9fe870;
    margin-left: auto;
}

/* ── Patient card ──────────────────────────────────────────── */
.patient-card {
    border: 1px solid rgba(14, 15, 12, 0.12);
    border-radius: 20px;
    background: #ffffff;
    overflow: hidden;
    box-shadow: rgba(14, 15, 12, 0.12) 0px 0px 0px 1px;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
}

/* Header */
.patient-card__header {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 14px 16px 12px;
    background: #f8f8f7;
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
}

.patient-card__avatar-wrap {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background: #e2f6d5;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: #163300;
    border: 1px solid rgba(159, 232, 112, 0.3);
}

.patient-card__main {
    flex: 1;
    min-width: 0;
}

.patient-card__name {
    font-size: 16px;
    font-weight: 700;
    color: #0e0f0c;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    margin-bottom: 7px;
    line-height: 1.2;
}

.patient-card__meta {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
}

.patient-card__actions {
    display: flex;
    flex-direction: row;
    gap: 5px;
    flex-shrink: 0;
    align-items: center;
}

/* Vitals */
.patient-card__vitals {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 10px 16px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.06);
    background: #fafafa;
}

.vital-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: #f5f7f3;
    border: 1px solid rgba(14, 15, 12, 0.1);
    border-radius: 9999px;
    padding: 4px 10px;
    font-size: 12px;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    color: #615d59;
}
.vital-chip svg {
    color: #a39e98;
}
.vital-chip__label {
    color: #868685;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.3px;
}
.vital-chip__val {
    font-weight: 600;
    color: #0e0f0c;
    font-size: 12px;
}

/* Drug sections container */
.patient-card__drugs {
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    background: #ffffff;
    flex: 1;
    overflow-y: auto;
}

.drug-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

/* Drug section header row */
.drug-section__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 10px 5px 8px;
    border-radius: 8px;
    border-left: 3px solid transparent;
}
.drug-section__header--green {
    background: #e2f6d5;
    border: 1px solid rgba(159, 232, 112, 0.5);
    border-left: 3px solid #9fe870;
}
.drug-section__header--gray {
    background: #f4fdf6;
    border: 1px solid #d1f3dd;
    border-left: 3px solid #d1f3dd;
}
.drug-section__header--red {
    background: #f3f4f6;
    border: 1px solid #e5e7eb;
    border-left: 3px solid #e5e7eb;
}

.drug-section__title {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    font-weight: 600;
    color: #454745;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
}
.drug-section__header--green .drug-section__title {
    color: #163300;
}
.drug-section__header--gray .drug-section__title {
    color: #3e6b4a;
}
.drug-section__header--red .drug-section__title {
    color: #6b7280;
}

.drug-section__count {
    font-size: 11px;
    font-weight: 700;
    border-radius: 9999px;
    padding: 1px 8px;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
}
.drug-section__count--green {
    background: #9fe870;
    color: #163300;
}
.drug-section__count--gray {
    background: #d1f3dd;
    color: #3e6b4a;
}
.drug-section__count--red {
    background: #f3f4f6;
    color: #6b7280;
    border: 1px solid #e5e7eb;
}

.drug-section__pills {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    padding-left: 4px;
}

.drug-section__empty {
    text-align: center;
    color: #a39e98;
    font-size: 13px;
    padding: 20px;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
}

/* Chips */
.chip {
    display: inline-flex;
    align-items: center;
    background: #f6f5f4;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 12px;
    padding: 2px 9px;
    font-size: 11px;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    color: #615d59;
    white-space: nowrap;
}
.chip--hn {
    background: #e2f6d5;
    border-color: rgba(159, 232, 112, 0.4);
    color: #163300;
    font-weight: 600;
}

/* Buttons */
.btn--outline {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border-radius: 9999px;
    border: 1px solid rgba(14, 15, 12, 0.2);
    background: #ffffff;
    color: #454745;
    font-size: 12px;
    font-weight: 500;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    cursor: pointer;
    transition:
        background 0.15s,
        border-color 0.15s,
        color 0.15s,
        transform 0.1s;
    white-space: nowrap;
}
.btn--outline:hover {
    background: #e2f6d5;
    border-color: #9fe870;
    color: #163300;
    transform: scale(1.05);
}
.btn--outline:active {
    transform: scale(0.95);
}

/* ── Lab results section ──────────────────────────────────── */
.patient-card__labs {
    border-top: 1px solid rgba(14, 15, 12, 0.07);
    padding: 12px 16px 14px;
    background: #fafbf9;
    flex-shrink: 0;
}

.lab-section__header {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    font-weight: 700;
    color: #454745;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    margin-bottom: 8px;
}

.lab-section__date-note {
    font-size: 11px;
    font-weight: 400;
    color: #a39e98;
    margin-left: 2px;
}

.lab-section__grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
}

.lab-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: #f5f7f3;
    border: 1px solid rgba(14, 15, 12, 0.1);
    border-radius: 9999px;
    padding: 4px 10px;
    font-size: 12px;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
}

.lab-item--abnormal {
    background: #fde8e8;
    border-color: rgba(208, 50, 56, 0.25);
}

.lab-item__name {
    color: #615d59;
    font-weight: 500;
}

.lab-item__result {
    font-weight: 700;
    color: #163300;
}

.lab-item__result--abnormal {
    color: #d03238;
}

.lab-item__date {
    font-size: 10px;
    font-weight: 400;
    color: #a39e98;
    margin-left: 2px;
}

.lab-item__not-found {
    color: #a39e98;
    font-size: 11px;
    font-style: italic;
}

/* Empty state */
.search-tab__empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: #a39e98;
    font-family:
        Inter,
        -apple-system,
        system-ui,
        "Segoe UI",
        Helvetica,
        Arial,
        "Tahoma",
        "TH Sarabun New",
        sans-serif;
    font-size: 14px;
}
.search-tab__empty-icon {
    color: rgba(0, 0, 0, 0.08);
}
</style>
