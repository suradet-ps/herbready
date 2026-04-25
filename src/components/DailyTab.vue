<script setup lang="ts">
/**
 * DailyTab — ประมวลผลรายวัน
 * Shows a table of all patients in department on a date with interactive drug panels.
 */
import { ref, computed, watch } from "vue";
import {
    Play,
    Search,
    X,
    RefreshCw,
    CheckSquare,
    Square,
    Loader,
    FileSpreadsheet,
    StopCircle,
} from "lucide-vue-next";
import type { PatientRecord, LabResult } from "../types";
import { api } from "../api/tauri";
import { formatDateThaiShort, todayISO } from "../utils/dateHelper";
import { hnPadded } from "../utils/drugParser";
import DrugPanel from "./DrugPanel.vue";
import { appConfig } from "../stores/appConfig";

// ── State ─────────────────────────────────────────────────────────────────
const processDate = ref(todayISO());
const loading = ref(false);
const vitalsOnDate = ref(true);
const loadId = ref(0);
const records = ref<PatientRecord[]>([]);
const statsLabel = ref("");

// Lab results: map from hn → LabResult[]
const labResultsMap = ref<Map<string, LabResult[]>>(new Map());
const searchText = ref("");
const searchHint = ref("");
const highlightedRow = ref<number | null>(null);

// ── Computed ──────────────────────────────────────────────────────────────
const printCount = computed(
    () => records.value.filter((r) => r.print_selected).length,
);
const hasData = computed(() => records.value.length > 0);

const filteredRecords = computed(() => records.value); // all rows shown; search highlights

// ── Methods ───────────────────────────────────────────────────────────────
async function loadData() {
    if (!processDate.value) return;
    loadId.value += 1;
    const myId = loadId.value;
    loading.value = true;
    statsLabel.value = "";
    records.value = [];
    labResultsMap.value = new Map();
    searchText.value = "";
    highlightedRow.value = null;

    try {
        const rows = await api.getDailyRecords(
            processDate.value,
            vitalsOnDate.value,
        );
        if (loadId.value !== myId) return; // cancelled
        // Initialise drug_selection from eligible + never lists
        records.value = rows.map((rec) => initDrugSelection(rec));
        const count = rows.length;
        statsLabel.value = `วันที่ ${formatDateThaiShort(processDate.value)}  |  พบ ${count} ราย`;

        // Fetch lab results in background (non-blocking)
        if (rows.length > 0 && appConfig.value.lab_rules.length > 0) {
            const hnList = rows.map((r) => r.hn);
            try {
                const labResults = await api.getLabResults(
                    processDate.value,
                    hnList,
                );
                if (loadId.value !== myId) return;
                const map = new Map<string, LabResult[]>();
                for (const lr of labResults) {
                    if (!map.has(lr.hn)) map.set(lr.hn, []);
                    map.get(lr.hn)!.push(lr);
                }
                labResultsMap.value = map;
            } catch {
                // lab results are non-critical — silently ignore errors
            }
        }
    } catch (err: unknown) {
        if (loadId.value !== myId) return; // cancelled
        statsLabel.value = `❌ ${err instanceof Error ? err.message : String(err)}`;
    } finally {
        if (loadId.value === myId) loading.value = false;
    }
}

function cancelLoad() {
    loadId.value += 1; // invalidate in-flight
    loading.value = false;
    labResultsMap.value = new Map();
    statsLabel.value = "⏹ ยกเลิกการประมวลผล";
}

// ── Lab helpers ────────────────────────────────────────────────────────────

/**
 * Check if a single lab result is abnormal according to configured rules.
 */
function isLabAbnormal(lr: LabResult): boolean {
    const labCode = String(lr.lab_items_code).trim();
    const rule = appConfig.value.lab_rules.find(
        (r) => String(r.lab_items_code).trim() === labCode,
    );
    if (!rule) return false;
    const val = parseFloat(lr.lab_order_result);
    if (isNaN(val)) return false;
    if (rule.compare_gt && val > rule.threshold) return true;
    if (rule.compare_lt && val < rule.threshold) return true;
    if (rule.compare_eq && Math.abs(val - rule.threshold) < 0.001) return true;
    return false;
}

/**
 * Returns all abnormal lab results for a given HN.
 */
function abnormalLabsForHn(hn: string): LabResult[] {
    const results = labResultsMap.value.get(hn) ?? [];
    return results.filter(isLabAbnormal);
}

/**
 * Build a full lab summary tooltip for any HN, showing every configured
 * lab rule with its latest result or "ไม่พบการตรวจ" if none found.
 */
function labFullTooltip(hn: string): string {
    const rules = appConfig.value.lab_rules;
    if (rules.length === 0) return "";
    const results = labResultsMap.value.get(hn) ?? [];
    const lines = rules.map((rule) => {
        const ruleCode = String(rule.lab_items_code).trim();
        const found = results.find(
            (r) => String(r.lab_items_code).trim() === ruleCode,
        );
        if (found) {
            const val = parseFloat(found.lab_order_result);
            let flag = "";
            if (!isNaN(val)) {
                if (rule.compare_gt && val > rule.threshold) flag = " ⚠️";
                else if (rule.compare_lt && val < rule.threshold) flag = " ⚠️";
                else if (
                    rule.compare_eq &&
                    Math.abs(val - rule.threshold) < 0.001
                )
                    flag = " ⚠️";
            }
            return `${rule.lab_items_name}: ${found.lab_order_result} (${found.order_date})${flag}`;
        }
        return `${rule.lab_items_name}: ไม่พบการตรวจ`;
    });
    return lines.join("\n");
}

function initDrugSelection(rec: PatientRecord): PatientRecord {
    if (Object.keys(rec.drug_selection).length > 0) return rec;
    // Initialize all drugs as unselected — user selects manually
    const selection: Record<string, boolean> = {};
    const eligible = rec.eligible_drugs_raw
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
    const never = rec.never_dispensed_drugs_raw
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
    for (const d of eligible) selection[d] = false;
    for (const d of never) selection[d] = false;
    return { ...rec, drug_selection: selection };
}

function updateRecord(idx: number, updated: PatientRecord) {
    records.value[idx] = updated;
}

function togglePrint(idx: number) {
    const rec = records.value[idx];
    records.value[idx] = { ...rec, print_selected: !rec.print_selected };
}

function setAllPrint(val: boolean) {
    records.value = records.value.map((r) => ({ ...r, print_selected: val }));
}

function onSearchTextChanged() {
    const t = searchText.value.trim();
    if (!t) {
        searchHint.value = "";
        highlightedRow.value = null;
        return;
    }
    if (/^\d+$/.test(t)) {
        if (t.length === 13) searchHint.value = "→ CID";
        else if (t.length >= 5 && t.length <= 9) searchHint.value = "→ HN";
        else searchHint.value = "→ ตัวเลข";
    } else {
        searchHint.value = "→ ชื่อ-นามสกุล";
    }
}

watch(searchText, onSearchTextChanged);

function doSearchJump() {
    const t = searchText.value.trim().toLowerCase();
    if (!t || records.value.length === 0) return;

    for (let i = 0; i < records.value.length; i++) {
        const rec = records.value[i];
        const hn = hnPadded(rec.hn).toLowerCase();
        const hnRaw = rec.hn.trim().toLowerCase();
        const cid = (rec.cid || "").toLowerCase();
        const name = rec.pt_name.toLowerCase();
        if (t === hn || t === hnRaw || cid.includes(t) || name.includes(t)) {
            highlightedRow.value = i;
            // Scroll into view
            const el = document.getElementById(`daily-row-${i}`);
            if (el) el.scrollIntoView({ behavior: "smooth", block: "center" });
            searchHint.value = `พบที่แถว ${i + 1}`;
            return;
        }
    }
    searchHint.value = "ไม่พบรายการ";
    highlightedRow.value = null;
}

function clearSearch() {
    searchText.value = "";
    searchHint.value = "";
    highlightedRow.value = null;
}

async function exportExcel() {
    const selected = records.value.filter((r) => r.print_selected);
    if (selected.length === 0) return;
    try {
        const path = await api.saveDialog(
            `HerbReady_${processDate.value}.xlsx`,
            [{ name: "Excel Files", extensions: ["xlsx"] }],
        );
        if (!path) return;
        const msg = await api.exportExcel(selected, processDate.value, path);
        alert(msg || "ส่งออก Excel สำเร็จ");
    } catch (err: unknown) {
        alert(
            `เกิดข้อผิดพลาด: ${err instanceof Error ? err.message : String(err)}`,
        );
    }
}

function selectedDrugCount(rec: PatientRecord): number {
    return Object.values(rec.drug_selection).filter(Boolean).length;
}
</script>

<template>
    <div class="daily-tab">
        <!-- ── Toolbar ──────────────────────────────────────────────────────── -->
        <div class="daily-tab__toolbar">
            <span class="toolbar-label">วันที่ประมวลผล:</span>
            <input
                v-model="processDate"
                type="date"
                class="daily-tab__date-input"
                @keydown.enter="loadData"
            />

            <button
                class="btn btn--primary"
                :disabled="loading"
                @click="loadData"
                type="button"
            >
                <Loader v-if="loading" :size="14" class="spin" />
                <Play v-else :size="14" />
                {{ loading ? "กำลังโหลด…" : "ประมวลผล" }}
            </button>
            <button
                v-if="loading"
                class="btn btn--stop"
                @click="cancelLoad"
                type="button"
                title="หยุดการประมวลผล"
            >
                <StopCircle :size="14" />
                หยุด
            </button>

            <span class="daily-tab__stats">{{ statsLabel }}</span>

            <span class="daily-tab__toolbar-spacer" />

            <div class="daily-tab__field-wrap">
                <span class="daily-tab__field-icon"><Search :size="14" /></span>
                <input
                    v-model="searchText"
                    type="text"
                    class="daily-tab__search-input"
                    placeholder="ค้นหา HN / CID / ชื่อ…"
                    @keydown.enter="doSearchJump"
                    autocomplete="off"
                    spellcheck="false"
                />
                <span
                    v-if="searchHint"
                    class="daily-tab__search-hint"
                    :class="{
                        'hint--found': searchHint.startsWith('พบที่'),
                        'hint--notfound': searchHint === 'ไม่พบรายการ',
                    }"
                >
                    {{ searchHint }}
                </span>
                <button
                    v-if="searchText"
                    class="daily-tab__clear-btn"
                    @click="clearSearch"
                    type="button"
                >
                    <X :size="13" />
                </button>
            </div>
        </div>

        <!-- Options + Legend (single row) -->
        <div
            class="daily-legend-and-options"
            role="group"
            aria-label="options and legend"
        >
            <label
                class="vitals-toggle"
                title="แสดงค่า Vital signs ของวันที่ประมวลผล ถ้าไม่ติ๊กจะแสดงค่าล่าสุด"
            >
                <input
                    type="checkbox"
                    v-model="vitalsOnDate"
                    class="vitals-toggle__check"
                />
                <span class="vitals-toggle__label"
                    >Vital signs ตามวันที่ประมวลผล</span
                >
            </label>

            <!-- Inline legend items -->
            <div class="daily-legend" aria-hidden="false">
                <span class="daily-legend__item">
                    <span
                        class="daily-legend__dot daily-legend__dot--green"
                    ></span>
                    ยาที่จ่ายได้
                </span>
                <span class="daily-legend__item">
                    <span
                        class="daily-legend__dot daily-legend__dot--gray"
                    ></span>
                    ยังไม่เคยจ่าย
                </span>
                <span class="daily-legend__item">
                    <span
                        class="daily-legend__dot daily-legend__dot--red"
                    ></span>
                    ยังไม่ถึงกำหนด
                </span>
                <span class="daily-legend__item">
                    <span
                        class="daily-legend__dot daily-legend__dot--dark"
                    ></span>
                    เลือกแล้ว
                </span>
            </div>
        </div>

        <!-- ── Table ──────────────────────────────────────────────────────── -->
        <div class="daily-tab__table-wrap">
            <table class="daily-table">
                <thead class="daily-table__head">
                    <tr>
                        <th class="daily-table__th th--hn">HN</th>
                        <th class="daily-table__th th--name">ชื่อ-นามสกุล</th>
                        <th class="daily-table__th th--pttype">สิทธิ์</th>
                        <th class="daily-table__th th--bp">BP</th>
                        <th class="daily-table__th th--pulse">Pulse</th>
                        <th class="daily-table__th th--weight">Weight</th>
                        <th class="daily-table__th th--drugs">แผงคัดกรองยา</th>
                        <th class="daily-table__th th--count">เลือก</th>
                        <th class="daily-table__th th--print">พิมพ์</th>
                    </tr>
                </thead>
                <tbody>
                    <tr
                        v-for="(rec, idx) in filteredRecords"
                        :key="rec.vn || rec.hn + idx"
                        :id="`daily-row-${idx}`"
                        class="daily-table__row"
                        :class="{
                            'daily-table__row--alt': idx % 2 === 1,
                            'daily-table__row--highlighted':
                                highlightedRow === idx,
                        }"
                    >
                        <!-- HN -->
                        <td class="daily-table__td td--hn">
                            <span
                                v-if="appConfig.lab_rules.length > 0"
                                :class="{
                                    'hn-abnormal-lab':
                                        abnormalLabsForHn(rec.hn).length > 0,
                                    'hn-has-lab': true,
                                }"
                                :title="labFullTooltip(rec.hn)"
                            >
                                {{ hnPadded(rec.hn) }}
                            </span>
                            <span v-else>{{ hnPadded(rec.hn) }}</span>
                        </td>

                        <!-- ชื่อ -->
                        <td class="daily-table__td td--name">
                            {{ rec.pt_name }}
                        </td>

                        <!-- สิทธิ์ -->
                        <td
                            class="daily-table__td td--pttype"
                            :title="rec.pttype_today"
                        >
                            {{ rec.pttype_today || "—" }}
                        </td>

                        <!-- Vitals -->
                        <td class="daily-table__td td--center">
                            {{ rec.last_blood_pressure || "—" }}
                        </td>
                        <td class="daily-table__td td--center">
                            {{ rec.last_pulse || "—" }}
                        </td>
                        <td class="daily-table__td td--center">
                            {{ rec.last_weight || "—" }}
                        </td>

                        <!-- Drug panel -->
                        <td class="daily-table__td td--drugs">
                            <DrugPanel
                                :record="rec"
                                :process-date="processDate"
                                @update:record="updateRecord(idx, $event)"
                                @count-changed="() => {}"
                            />
                        </td>

                        <!-- เลือก count -->
                        <td class="daily-table__td td--count">
                            <span
                                class="count-badge"
                                :class="
                                    selectedDrugCount(rec) > 0
                                        ? 'count-badge--active'
                                        : 'count-badge--zero'
                                "
                            >
                                {{ selectedDrugCount(rec) }}
                            </span>
                        </td>

                        <!-- พิมพ์ checkbox -->
                        <td class="daily-table__td td--center">
                            <button
                                class="print-check"
                                :class="{
                                    'print-check--on': rec.print_selected,
                                }"
                                @click="togglePrint(idx)"
                                type="button"
                                :title="
                                    rec.print_selected
                                        ? 'ยกเลิกพิมพ์'
                                        : 'เลือกพิมพ์'
                                "
                            >
                                <CheckSquare
                                    v-if="rec.print_selected"
                                    :size="16"
                                />
                                <Square v-else :size="16" />
                            </button>
                        </td>
                    </tr>

                    <!-- Empty state row -->
                    <tr v-if="!loading && records.length === 0">
                        <td colspan="9" class="daily-table__empty">
                            <div>
                                <RefreshCw :size="32" />
                                <p>
                                    กดปุ่ม "ประมวลผล"
                                    เพื่อโหลดรายการผู้ป่วยประจำวัน
                                </p>
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <!-- ── Footer ────────────────────────────────────────────────────── -->
        <div class="daily-tab__footer">
            <button
                class="btn btn--ghost btn--sm"
                @click="setAllPrint(true)"
                type="button"
            >
                <CheckSquare :size="13" />
                เลือกพิมพ์ทั้งหมด
            </button>
            <button
                class="btn btn--ghost btn--sm"
                @click="setAllPrint(false)"
                type="button"
            >
                <Square :size="13" />
                ยกเลิกทั้งหมด
            </button>

            <span class="daily-tab__footer-spacer" />

            <span class="footer-export-label">ส่งออก:</span>

            <button
                class="btn btn--export"
                :disabled="!hasData || printCount === 0"
                @click="exportExcel"
                type="button"
                :title="
                    printCount === 0
                        ? 'กรุณาเลือกรายการที่ต้องการพิมพ์ก่อน'
                        : `ส่งออก ${printCount} รายการเป็น Excel`
                "
            >
                <FileSpreadsheet :size="14" />
                Excel
            </button>

            <span v-if="printCount > 0" class="footer-print-count">
                เลือก {{ printCount }} ราย
            </span>
        </div>
    </div>
</template>

<style scoped>
.daily-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: #ffffff;
}

/* Lab HN styles */
.hn-has-lab {
    cursor: help;
    display: inline-block;
}

.hn-abnormal-lab {
    color: #d03238;
    font-weight: 700;
    border-bottom: 1.5px dashed #d03238;
}

/* Toolbar */
.daily-tab__toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    flex-shrink: 0;
    flex-wrap: nowrap;
    border-bottom: 1px solid rgba(14, 15, 12, 0.1);
    background: #ffffff;
}

.daily-tab__toolbar-spacer {
    flex: 1;
}

/* Options + legend row */
.daily-legend-and-options {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 16px;
    background: #ffffff;
    border-bottom: 1px solid rgba(14, 15, 12, 0.07);
    flex-shrink: 0;
    flex-wrap: nowrap;
}
.daily-legend {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0;
    background: transparent;
    flex-shrink: 0;
    flex-wrap: nowrap;
}
.daily-legend__item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: #454745;
    font-weight: 600;
}
.daily-legend__dot {
    width: 12px;
    height: 12px;
    border-radius: 9999px;
    border: 1px solid;
    flex-shrink: 0;
}

.daily-legend__dot--green {
    background: #e2f6d5;
    border-color: #9fe870;
}

/* Never dispensed — soft gray-green (muted) */
.daily-legend__dot--gray {
    background: #f4fdf6;
    border-color: #d1f3dd;
}

/* Not yet due — light gray muted */
.daily-legend__dot--red {
    background: #f3f4f6;
    border-color: #e5e7eb;
}

.daily-legend__dot--dark {
    background: #0e0f0c;
    border-color: #0e0f0c;
}

/* Toolbar label */
.toolbar-label {
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
    color: #454745;
    font-weight: 600;
    white-space: nowrap;
}

.daily-tab__date-input {
    height: 34px;
    padding: 0 14px;
    border: 1px solid rgba(14, 15, 12, 0.15);
    border-radius: 9999px;
    background: #ffffff;
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
    font-size: 13px;
    outline: none;
    transition:
        border-color 0.15s,
        box-shadow 0.15s;
}
.daily-tab__date-input:focus {
    border-color: #9fe870;
    box-shadow: 0 0 0 2px rgba(159, 232, 112, 0.25);
    outline: none;
}

.daily-tab__stats {
    font-size: 13px;
    color: #868685;
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
    margin-left: 4px;
}

/* Search bar */
.daily-tab__field-wrap {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1;
    max-width: 340px;
    min-width: 180px;
}

.daily-tab__field-icon {
    position: absolute;
    left: 12px;
    color: #868685;
    pointer-events: none;
    display: flex;
}

.daily-tab__search-input {
    width: 100%;
    height: 32px;
    padding: 0 100px 0 34px;
    border: 1px solid rgba(14, 15, 12, 0.15);
    border-radius: 9999px;
    background: #ffffff;
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
    font-size: 12px;
    outline: none;
    transition:
        border-color 0.15s,
        box-shadow 0.15s;
}
.daily-tab__search-input::placeholder {
    color: #868685;
}
.daily-tab__search-input:focus {
    border-color: #9fe870;
    box-shadow: 0 0 0 2px rgba(159, 232, 112, 0.25);
    outline: none;
}

.daily-tab__search-hint {
    position: absolute;
    right: 28px;
    font-size: 11px;
    color: #868685;
    pointer-events: none;
}
.hint--found {
    color: #054d28;
    font-weight: 600;
}
.hint--notfound {
    color: #d03238;
}

.daily-tab__clear-btn {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    cursor: pointer;
    color: #868685;
    display: flex;
    align-items: center;
    padding: 2px;
    border-radius: 9999px;
    transition: color 0.15s;
}
.daily-tab__clear-btn:hover {
    color: #d03238;
}

/* Table wrapper */
.daily-tab__table-wrap {
    flex: 1;
    overflow: auto;
    background: #ffffff;
}

/* Table */
.daily-table {
    width: 100%;
    border-collapse: collapse;
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
    table-layout: fixed;
}

.daily-table__head {
    position: sticky;
    top: 0;
    z-index: 2;
}

.daily-table__th {
    text-align: center;
    padding: 8px 6px;
    background: #f5f7f3;
    color: #454745;
    font-weight: 600;
    font-size: 13px;
    border-bottom: 1px solid rgba(14, 15, 12, 0.1);
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
}

.th--hn {
    width: 90px;
}
.th--name {
    width: 200px;
    text-align: left;
    padding-left: 10px;
}
.th--pttype {
    width: 64px;
    min-width: 54px;
    max-width: 64px;
}
.th--bp {
    width: 82px;
}
.th--pulse {
    width: 60px;
}
.th--weight {
    width: 62px;
}
.th--drugs {
    /* takes all remaining space */
    min-width: 260px;
}
.th--count {
    width: 52px;
}
.th--print {
    width: 52px;
}

.daily-table__row {
    border-bottom: 1px solid rgba(14, 15, 12, 0.05);
    transition: background 0.1s;
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
}
.daily-table__row:hover {
    background: #f0f7eb;
}
.daily-table__row--alt {
    background: #f9fbf7;
}
.daily-table__row--highlighted {
    background: #e2f6d5 !important;
}

.daily-table__td {
    padding: 5px 6px;
    color: #0e0f0c;
    vertical-align: middle;
    border-bottom: 1px solid rgba(14, 15, 12, 0.05);
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
}
.td--hn {
    text-align: center;
    color: #454745;
    font-weight: 500;
    font-size: 12px;
    font-variant-numeric: tabular-nums;
}
.td--name {
    text-align: left;
    padding-left: 10px;
    color: #0e0f0c;
    font-weight: 600;
}
.td--pttype {
    text-align: center;
    font-size: 11px;
    width: 64px;
    max-width: 64px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: default;
}
.td--center {
    text-align: center;
    font-size: 12px;
}
.td--drugs {
    padding: 3px 6px;
    /* allow drug pills to wrap inside this cell */
    min-width: 260px;
}
.td--count {
    text-align: center;
}

/* Count badge */
.count-badge {
    display: inline-block;
    border-radius: 9999px;
    padding: 2px 8px;
    font-size: 12px;
    font-weight: 600;
    min-width: 26px;
    text-align: center;
    background: #e8ebe6;
    color: #454745;
    border: none;
}
.count-badge--active {
    background: #e2f6d5;
    border: 1px solid #9fe870;
    color: #163300;
}
.count-badge--zero {
    background: transparent;
    border: none;
    color: #868685;
}

/* Print check */
.print-check {
    background: none;
    border: none;
    cursor: pointer;
    color: #868685;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 3px;
    border-radius: 4px;
    margin: 0 auto;
    transition: color 0.15s;
}
.print-check--on {
    color: #163300;
}
.print-check:hover {
    color: #163300;
}

/* Empty row */
.daily-table__empty {
    text-align: center;
    padding: 48px 16px;
}
.daily-table__empty > div {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    color: #868685;
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

/* Footer */
.daily-tab__footer {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border-top: 1px solid rgba(14, 15, 12, 0.1);
    background: #f5f7f3;
    flex-shrink: 0;
}

.daily-tab__footer-spacer {
    flex: 1;
}

.footer-export-label {
    font-size: 13px;
    color: #868685;
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

.footer-print-count {
    font-size: 12px;
    color: #163300;
    font-weight: 600;
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
    margin-left: 4px;
    background: #e2f6d5;
    padding: 2px 10px;
    border-radius: 9999px;
}

.spin {
    animation: spin 1s linear infinite;
}
@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

/* Primary / ghost / export shared base */
.btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 14px;
    border-radius: 9999px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition:
        background 0.15s,
        color 0.15s,
        transform 0.1s;
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
.btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
    transform: none !important;
}

/* Primary action button (Load / ประมวลผล) */
.btn--primary {
    background: #9fe870;
    color: #163300;
    border: none;
}
.btn--primary:not(:disabled):hover {
    background: #8fdc60;
    transform: scale(1.05);
}
.btn--primary:not(:disabled):active {
    transform: scale(0.95);
}

/* Stop button */
.btn--stop {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 14px;
    border-radius: 9999px;
    border: none;
    background: rgba(208, 50, 56, 0.1);
    color: #d03238;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition:
        background 0.15s,
        transform 0.1s;
}
.btn--stop:hover {
    background: rgba(208, 50, 56, 0.15);
    transform: scale(1.05);
}
.btn--stop:active {
    transform: scale(0.95);
}

/* Ghost (select-all / deselect-all) */
.btn--ghost {
    background: transparent;
    color: #454745;
    border: 1px solid rgba(14, 15, 12, 0.15);
}
.btn--ghost:not(:disabled):hover {
    background: #f0f7eb;
    border-color: #9fe870;
    color: #163300;
    transform: scale(1.03);
}
.btn--ghost:not(:disabled):active {
    transform: scale(0.97);
}

.btn--sm {
    padding: 4px 10px;
    font-size: 12px;
}

/* Export button */
.btn--export {
    background: #ffffff;
    color: #0e0f0c;
    border: 1px solid rgba(14, 15, 12, 0.15);
    border-radius: 9999px;
}
.btn--export:not(:disabled):hover {
    background: #f0f7eb;
    border-color: #9fe870;
    color: #163300;
    transform: scale(1.05);
}
.btn--export:not(:disabled):active {
    transform: scale(0.95);
}

/* ── Vitals Toggle ───────────────────────────────────────────────────────── */
.vitals-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    user-select: none;
}
.vitals-toggle:hover .vitals-toggle__label {
    color: #163300;
}
.vitals-toggle__check {
    width: 13px;
    height: 13px;
    accent-color: #163300;
    cursor: pointer;
}
.vitals-toggle__label {
    font-size: 12px;
    color: #454745;
    font-weight: 600;
    white-space: nowrap;
}
</style>
