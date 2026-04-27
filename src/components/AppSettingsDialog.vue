<script setup lang="ts">
/**
 * AppSettingsDialog — Manage drug list, departments, and lab rules.
 */
import { ref, reactive, onMounted } from "vue";
import {
    Settings,
    Plus,
    Trash2,
    X,
    Save,
    Search,
    Loader,
    Upload,
    Download,
    Eye,
    EyeOff,
    FlaskConical,
    Pill,
    Leaf,
} from "lucide-vue-next";
import type {
    DrugConfig,
    DeptConfig,
    LabRuleConfig,
    HerbDrugInteraction,
    HerbDrugEntry,
} from "../types";
import { api } from "../api/tauri";
import { appConfig, saveAppConfig } from "../stores/appConfig";

const emit = defineEmits<{
    (e: "close"): void;
}>();

type Page = "drugs" | "depts" | "labs" | "interactions";
const activePage = ref<Page>("drugs");
const saving = ref(false);

// Drug form
const drugForm = reactive<DrugConfig>({
    icode: "",
    abbr: "",
    course_days: 7,
    capsules: 0,
    drug_name: "",
    enabled: true,
});
const drugLookupDone = ref(false);
const drugLookupLoading = ref(false);
const drugs = ref<DrugConfig[]>([]);
const selectedDrugIdx = ref<number | null>(null);

// Dept form
const deptForm = reactive<DeptConfig>({ code: "", name: "" });
const deptLookupDone = ref(false);
const deptLookupLoading = ref(false);
const depts = ref<DeptConfig[]>([]);
const selectedDeptIdx = ref<number | null>(null);

// Lab rule form
const labForm = reactive<LabRuleConfig>({
    lab_items_code: "",
    lab_items_name: "",
    threshold: 0,
    compare_gt: false,
    compare_eq: false,
    compare_lt: false,
});
const thresholdInput = ref<string>("");
const labLookupDone = ref(false);
const labLookupLoading = ref(false);
const labRules = ref<LabRuleConfig[]>([]);
const selectedLabIdx = ref<number | null>(null);

// ── Herb/Drug interaction state ────────────────────────────────────────────

/** Draft for the per-herb entries in the form (includes lookup UI state). */
interface HerbDrugDraft {
    icode: string;
    name: string;
    lookupDone: boolean;
    lookupLoading: boolean;
}

const interactionModernIcode = ref("");
const interactionModernName = ref("");
const interactionModernLookupDone = ref(false);
const interactionModernLookupLoading = ref(false);
const interactionReason = ref("");
const herbDrugDrafts = ref<HerbDrugDraft[]>([
    { icode: "", name: "", lookupDone: false, lookupLoading: false },
]);
const interactions = ref<HerbDrugInteraction[]>([]);
const selectedInteractionIdx = ref<number | null>(null);

onMounted(() => {
    drugs.value = appConfig.value.drugs.map((d) => ({ ...d }));
    depts.value = appConfig.value.departments.map((d) => ({ ...d }));
    labRules.value = (appConfig.value.lab_rules ?? []).map((r) => ({ ...r }));
    interactions.value = (appConfig.value.herb_drug_interactions ?? []).map(
        (r) => ({ ...r, herb_drugs: r.herb_drugs.map((h) => ({ ...h })) }),
    );
});

// ── Drug operations ────────────────────────────────────────────────────────

/** Called on every keystroke in icode field — resets lookup state */
function onIcodeInput() {
    if (drugLookupDone.value) {
        drugLookupDone.value = false;
        drugForm.drug_name = "";
    }
}

async function lookupDrugName() {
    if (!drugForm.icode.trim()) return;
    drugLookupLoading.value = true;
    drugLookupDone.value = false;
    drugForm.drug_name = "";
    try {
        const name = await api.lookupDrugName(drugForm.icode.trim());
        drugForm.drug_name = name;
        drugLookupDone.value = true;
    } catch {
        drugForm.drug_name = "";
        drugLookupDone.value = false;
    } finally {
        drugLookupLoading.value = false;
    }
}

function selectDrug(idx: number) {
    selectedDrugIdx.value = idx;
    const d = drugs.value[idx];
    drugForm.icode = d.icode;
    drugForm.abbr = d.abbr;
    drugForm.course_days = d.course_days;
    drugForm.capsules = d.capsules;
    drugForm.drug_name = d.drug_name;
    drugForm.enabled = d.enabled !== false;
    drugLookupDone.value = true; // existing entry already verified
}

function addOrUpdateDrug() {
    if (!drugForm.icode.trim() || !drugForm.drug_name.trim()) return;
    if (!drugLookupDone.value) return;
    const entry: DrugConfig = {
        icode: drugForm.icode.trim(),
        abbr: drugForm.abbr.trim(),
        course_days: Number(drugForm.course_days) || 7,
        capsules: Number(drugForm.capsules) || 0,
        drug_name: drugForm.drug_name.trim(),
        enabled: drugForm.enabled !== false,
    };
    if (selectedDrugIdx.value !== null) {
        drugs.value[selectedDrugIdx.value] = entry;
        selectedDrugIdx.value = null;
    } else {
        drugs.value.push(entry);
    }
    clearDrugForm();
}

function toggleDrugEnabled(idx: number) {
    const d = drugs.value[idx];
    drugs.value[idx] = { ...d, enabled: d.enabled === false ? true : false };
    // If this item is selected in the form, sync the form enabled state
    if (selectedDrugIdx.value === idx) {
        drugForm.enabled = drugs.value[idx].enabled !== false;
    }
}

function deleteDrug(idx: number) {
    drugs.value.splice(idx, 1);
    if (selectedDrugIdx.value === idx) clearDrugForm();
    else if (selectedDrugIdx.value !== null && selectedDrugIdx.value > idx) {
        selectedDrugIdx.value -= 1;
    }
}

function clearDrugForm() {
    Object.assign(drugForm, {
        icode: "",
        abbr: "",
        course_days: 7,
        capsules: 0,
        drug_name: "",
        enabled: true,
    });
    drugLookupDone.value = false;
    selectedDrugIdx.value = null;
}

// ── Dept operations ────────────────────────────────────────────────────────

/** Called on every keystroke in dept code field — resets lookup state */
function onDeptCodeInput() {
    if (deptLookupDone.value) {
        deptLookupDone.value = false;
        deptForm.name = "";
    }
}

async function lookupDeptName() {
    if (!deptForm.code.trim()) return;
    deptLookupLoading.value = true;
    deptLookupDone.value = false;
    deptForm.name = "";
    try {
        const name = await api.lookupDeptName(deptForm.code.trim());
        deptForm.name = name;
        deptLookupDone.value = true;
    } catch {
        deptForm.name = "";
        deptLookupDone.value = false;
    } finally {
        deptLookupLoading.value = false;
    }
}

function selectDept(idx: number) {
    selectedDeptIdx.value = idx;
    const d = depts.value[idx];
    deptForm.code = d.code;
    deptForm.name = d.name;
    deptLookupDone.value = true; // existing entry already verified
}

function addOrUpdateDept() {
    if (!deptForm.code.trim() || !deptForm.name.trim()) return;
    if (!deptLookupDone.value) return;
    const entry: DeptConfig = { ...deptForm };
    if (selectedDeptIdx.value !== null) {
        depts.value[selectedDeptIdx.value] = entry;
        selectedDeptIdx.value = null;
    } else {
        depts.value.push(entry);
    }
    clearDeptForm();
}

function deleteDept(idx: number) {
    depts.value.splice(idx, 1);
    if (selectedDeptIdx.value === idx) clearDeptForm();
    else if (selectedDeptIdx.value !== null && selectedDeptIdx.value > idx) {
        selectedDeptIdx.value -= 1;
    }
}

function clearDeptForm() {
    Object.assign(deptForm, { code: "", name: "" });
    deptLookupDone.value = false;
    selectedDeptIdx.value = null;
}

// ── Lab rule operations ────────────────────────────────────────────────────

function onLabCodeInput() {
    if (labLookupDone.value) {
        labLookupDone.value = false;
        labForm.lab_items_name = "";
    }
}

async function lookupLabItemName() {
    if (!labForm.lab_items_code.trim()) return;
    labLookupLoading.value = true;
    labLookupDone.value = false;
    labForm.lab_items_name = "";
    try {
        const name = await api.lookupLabItemName(labForm.lab_items_code.trim());
        labForm.lab_items_name = name;
        labLookupDone.value = true;
    } catch {
        labForm.lab_items_name = "";
        labLookupDone.value = false;
    } finally {
        labLookupLoading.value = false;
    }
}

function selectLabRule(idx: number) {
    selectedLabIdx.value = idx;
    const r = labRules.value[idx];
    labForm.lab_items_code = r.lab_items_code;
    labForm.lab_items_name = r.lab_items_name;
    thresholdInput.value = String(r.threshold);
    labForm.compare_gt = r.compare_gt;
    labForm.compare_eq = r.compare_eq;
    labForm.compare_lt = r.compare_lt;
    labLookupDone.value = true;
}

function addOrUpdateLabRule() {
    if (!labForm.lab_items_code.trim() || !labForm.lab_items_name.trim())
        return;
    if (!labLookupDone.value) return;
    if (!labForm.compare_gt && !labForm.compare_eq && !labForm.compare_lt)
        return;
    const entry: LabRuleConfig = {
        lab_items_code: labForm.lab_items_code.trim(),
        lab_items_name: labForm.lab_items_name.trim(),
        threshold: Number(thresholdInput.value) || 0,
        compare_gt: labForm.compare_gt,
        compare_eq: labForm.compare_eq,
        compare_lt: labForm.compare_lt,
    };
    if (selectedLabIdx.value !== null) {
        labRules.value[selectedLabIdx.value] = entry;
        selectedLabIdx.value = null;
    } else {
        // prevent duplicate codes
        const existing = labRules.value.findIndex(
            (r) => r.lab_items_code === entry.lab_items_code,
        );
        if (existing !== -1) {
            labRules.value[existing] = entry;
        } else {
            labRules.value.push(entry);
        }
    }
    clearLabForm();
}

function deleteLabRule(idx: number) {
    labRules.value.splice(idx, 1);
    if (selectedLabIdx.value === idx) clearLabForm();
    else if (selectedLabIdx.value !== null && selectedLabIdx.value > idx) {
        selectedLabIdx.value -= 1;
    }
}

function clearLabForm() {
    Object.assign(labForm, {
        lab_items_code: "",
        lab_items_name: "",
        threshold: 0,
        compare_gt: false,
        compare_eq: false,
        compare_lt: false,
    });
    thresholdInput.value = "";
    labLookupDone.value = false;
    selectedLabIdx.value = null;
}

/** Return a human-readable condition string for a rule */
function ruleConditionLabel(r: LabRuleConfig): string {
    const parts: string[] = [];
    if (r.compare_gt) parts.push(`> ${r.threshold}`);
    if (r.compare_eq) parts.push(`= ${r.threshold}`);
    if (r.compare_lt) parts.push(`< ${r.threshold}`);
    return parts.join(" หรือ ");
}

// ── Herb/Drug interaction operations ──────────────────────────────────────

function onModernIcodeInput() {
    if (interactionModernLookupDone.value) {
        interactionModernLookupDone.value = false;
        interactionModernName.value = "";
    }
}

async function lookupModernDrugName() {
    if (!interactionModernIcode.value.trim()) return;
    interactionModernLookupLoading.value = true;
    interactionModernLookupDone.value = false;
    interactionModernName.value = "";
    try {
        const name = await api.lookupDrugName(
            interactionModernIcode.value.trim(),
        );
        interactionModernName.value = name;
        interactionModernLookupDone.value = true;
    } catch {
        interactionModernName.value = "";
        interactionModernLookupDone.value = false;
    } finally {
        interactionModernLookupLoading.value = false;
    }
}

function onHerbIcodeInput(idx: number) {
    const draft = herbDrugDrafts.value[idx];
    if (draft && draft.lookupDone) {
        herbDrugDrafts.value[idx] = {
            ...draft,
            lookupDone: false,
            name: "",
        };
    }
}

async function lookupHerbDrugName(idx: number) {
    const draft = herbDrugDrafts.value[idx];
    if (!draft || !draft.icode.trim()) return;
    herbDrugDrafts.value[idx] = {
        ...draft,
        lookupLoading: true,
        lookupDone: false,
        name: "",
    };
    try {
        const name = await api.lookupDrugName(draft.icode.trim());
        herbDrugDrafts.value[idx] = {
            ...herbDrugDrafts.value[idx],
            name,
            lookupDone: true,
            lookupLoading: false,
        };
    } catch {
        herbDrugDrafts.value[idx] = {
            ...herbDrugDrafts.value[idx],
            name: "",
            lookupDone: false,
            lookupLoading: false,
        };
    }
}

function addHerbDrugEntry() {
    herbDrugDrafts.value.push({
        icode: "",
        name: "",
        lookupDone: false,
        lookupLoading: false,
    });
}

function removeHerbDrugEntry(idx: number) {
    if (herbDrugDrafts.value.length <= 1) return;
    herbDrugDrafts.value.splice(idx, 1);
}

function selectInteraction(idx: number) {
    selectedInteractionIdx.value = idx;
    const r = interactions.value[idx];
    interactionModernIcode.value = r.modern_drug_icode;
    interactionModernName.value = r.modern_drug_name;
    interactionModernLookupDone.value = true;
    interactionReason.value = r.reason;
    herbDrugDrafts.value = r.herb_drugs.map((h) => ({
        icode: h.icode,
        name: h.name,
        lookupDone: true,
        lookupLoading: false,
    }));
    if (herbDrugDrafts.value.length === 0) {
        herbDrugDrafts.value = [
            { icode: "", name: "", lookupDone: false, lookupLoading: false },
        ];
    }
}

function addOrUpdateInteraction() {
    if (
        !interactionModernIcode.value.trim() ||
        !interactionModernName.value.trim()
    )
        return;
    if (!interactionModernLookupDone.value) return;
    const validHerbs = herbDrugDrafts.value.filter(
        (h) => h.icode.trim() && h.name.trim() && h.lookupDone,
    );
    if (validHerbs.length === 0) return;
    const herbEntries: HerbDrugEntry[] = validHerbs.map((h) => ({
        icode: h.icode.trim(),
        name: h.name.trim(),
    }));
    const entry: HerbDrugInteraction = {
        modern_drug_icode: interactionModernIcode.value.trim(),
        modern_drug_name: interactionModernName.value.trim(),
        herb_drugs: herbEntries,
        reason: interactionReason.value.trim(),
    };
    if (selectedInteractionIdx.value !== null) {
        interactions.value[selectedInteractionIdx.value] = entry;
        selectedInteractionIdx.value = null;
    } else {
        interactions.value.push(entry);
    }
    clearInteractionForm();
}

function deleteInteraction(idx: number) {
    interactions.value.splice(idx, 1);
    if (selectedInteractionIdx.value === idx) clearInteractionForm();
    else if (
        selectedInteractionIdx.value !== null &&
        selectedInteractionIdx.value > idx
    ) {
        selectedInteractionIdx.value -= 1;
    }
}

function clearInteractionForm() {
    interactionModernIcode.value = "";
    interactionModernName.value = "";
    interactionModernLookupDone.value = false;
    interactionModernLookupLoading.value = false;
    interactionReason.value = "";
    herbDrugDrafts.value = [
        { icode: "", name: "", lookupDone: false, lookupLoading: false },
    ];
    selectedInteractionIdx.value = null;
}

// ── Export config ──────────────────────────────────────────────────────────
async function exportConfig() {
    try {
        const jsonStr = await api.exportAppConfig();
        const filePath = await api.saveDialog("app_config.json", [
            { name: "JSON", extensions: ["json"] },
        ]);
        if (filePath) {
            const { writeTextFile } = await import("@tauri-apps/plugin-fs");
            await writeTextFile(filePath, jsonStr);
            alert("ส่งออกข้อมูลสำเร็จ");
        }
    } catch (err: unknown) {
        alert(
            `ส่งออกล้มเหลว: ${err instanceof Error ? err.message : String(err)}`,
        );
    }
}

// ── Import config ──────────────────────────────────────────────────────────
async function importConfig() {
    try {
        const filePath = await api.openDialog([
            { name: "JSON", extensions: ["json"] },
        ]);
        if (!filePath || Array.isArray(filePath)) return;
        const { readTextFile } = await import("@tauri-apps/plugin-fs");
        const content = await readTextFile(filePath);
        const cfg = await api.importAppConfig(content);
        drugs.value = cfg.drugs.map((d) => ({ ...d }));
        depts.value = cfg.departments.map((d) => ({ ...d }));
        labRules.value = (cfg.lab_rules ?? []).map((r) => ({ ...r }));
        interactions.value = (cfg.herb_drug_interactions ?? []).map((r) => ({
            ...r,
            herb_drugs: r.herb_drugs.map((h) => ({ ...h })),
        }));
        alert("นำเข้าข้อมูลสำเร็จ");
    } catch (err: unknown) {
        alert(
            `นำเข้าล้มเหลว: ${err instanceof Error ? err.message : String(err)}`,
        );
    }
}

// ── Save ───────────────────────────────────────────────────────────────────
async function save() {
    saving.value = true;
    try {
        await saveAppConfig({
            drugs: drugs.value,
            departments: depts.value,
            lab_rules: labRules.value,
            herb_drug_interactions: interactions.value,
        });
        emit("close");
    } catch (err: unknown) {
        alert(
            `บันทึกล้มเหลว: ${err instanceof Error ? err.message : String(err)}`,
        );
    } finally {
        saving.value = false;
    }
}
</script>

<template>
    <div class="dialog-overlay" @click.self="emit('close')">
        <div class="dialog dialog--wide">
            <!-- Header -->
            <div class="dialog__header">
                <Settings :size="18" />
                <span class="dialog__title">ตั้งค่าแอปพลิเคชัน</span>
                <button
                    class="dialog__close"
                    @click="emit('close')"
                    type="button"
                >
                    <X :size="16" />
                </button>
            </div>

            <!-- Tab selector -->
            <div class="app-settings__tabs">
                <button
                    class="app-tab-btn"
                    :class="{ 'app-tab-btn--active': activePage === 'drugs' }"
                    @click="activePage = 'drugs'"
                    type="button"
                >
                    รายการยาสมุนไพร ({{ drugs.length }})
                </button>
                <button
                    class="app-tab-btn"
                    :class="{ 'app-tab-btn--active': activePage === 'depts' }"
                    @click="activePage = 'depts'"
                    type="button"
                >
                    แผนก ({{ depts.length }})
                </button>
                <button
                    class="app-tab-btn"
                    :class="{ 'app-tab-btn--active': activePage === 'labs' }"
                    @click="activePage = 'labs'"
                    type="button"
                >
                    ผลแลป ({{ labRules.length }})
                </button>
                <button
                    class="app-tab-btn"
                    :class="{
                        'app-tab-btn--active': activePage === 'interactions',
                    }"
                    @click="activePage = 'interactions'"
                    type="button"
                >
                    Herb/Drug ({{ interactions.length }})
                </button>
            </div>

            <!-- Body -->
            <div class="dialog__body dialog__body--split">
                <!-- ── Drug page ────────────────────────────────────────────── -->
                <template v-if="activePage === 'drugs'">
                    <!-- Drug list -->
                    <div class="settings-list">
                        <div
                            v-for="(d, idx) in drugs"
                            :key="d.icode"
                            class="settings-list__item"
                            :class="{
                                'settings-list__item--selected':
                                    selectedDrugIdx === idx,
                            }"
                            @click="selectDrug(idx)"
                        >
                            <div class="settings-list__main">
                                <span class="settings-list__primary">{{
                                    d.drug_name
                                }}</span>
                                <span class="settings-list__secondary">
                                    {{ d.icode }} | abbr: {{ d.abbr }}
                                    <span
                                        v-if="d.enabled === false"
                                        class="disabled-badge"
                                        >ปิดใช้งาน</span
                                    >
                                </span>
                            </div>
                            <button
                                class="settings-list__toggle"
                                @click.stop="toggleDrugEnabled(idx)"
                                type="button"
                                :title="
                                    d.enabled === false
                                        ? 'เปิดใช้งาน'
                                        : 'ปิดใช้งาน'
                                "
                            >
                                <EyeOff v-if="d.enabled === false" :size="13" />
                                <Eye v-else :size="13" />
                            </button>
                            <button
                                class="settings-list__del"
                                @click.stop="deleteDrug(idx)"
                                type="button"
                                title="ลบ"
                            >
                                <Trash2 :size="13" />
                            </button>
                        </div>
                        <div
                            v-if="drugs.length === 0"
                            class="settings-list__empty"
                        >
                            ยังไม่มีรายการยา
                        </div>
                    </div>

                    <!-- Drug form -->
                    <div class="settings-form">
                        <h3 class="settings-form__title">
                            {{
                                selectedDrugIdx !== null
                                    ? "แก้ไขยา"
                                    : "เพิ่มยาใหม่"
                            }}
                        </h3>
                        <div class="form-row">
                            <label class="form-label">iCode</label>
                            <div class="icode-wrap">
                                <input
                                    v-model="drugForm.icode"
                                    class="form-input"
                                    type="text"
                                    placeholder="1580004"
                                    @input="onIcodeInput"
                                />
                                <button
                                    class="btn btn--ghost btn--sm"
                                    @click="lookupDrugName"
                                    :disabled="drugLookupLoading"
                                    type="button"
                                >
                                    <Loader
                                        v-if="drugLookupLoading"
                                        :size="12"
                                        class="spin"
                                    />
                                    <Search v-else :size="12" />
                                </button>
                            </div>
                        </div>
                        <div class="form-row">
                            <label class="form-label">ชื่อยา</label>
                            <!-- ชื่อยา — auto-filled, readonly -->
                            <input
                                v-model="drugForm.drug_name"
                                class="form-input form-input--readonly"
                                type="text"
                                placeholder="กดค้นหา iCode ก่อน"
                                readonly
                            />
                        </div>
                        <div class="form-row">
                            <label class="form-label">ตัวย่อ</label>
                            <input
                                v-model="drugForm.abbr"
                                class="form-input"
                                type="text"
                                placeholder="ฟ้า"
                                :disabled="!drugLookupDone"
                            />
                        </div>
                        <div class="form-row">
                            <label class="form-label">วันที่จ่าย</label>
                            <input
                                v-model.number="drugForm.course_days"
                                class="form-input"
                                type="number"
                                min="1"
                                :disabled="!drugLookupDone"
                            />
                            <span class="form-unit">วัน</span>
                        </div>
                        <div class="form-row">
                            <label class="form-label">จำนวน</label>
                            <input
                                v-model.number="drugForm.capsules"
                                class="form-input"
                                type="number"
                                min="0"
                                :disabled="!drugLookupDone"
                            />
                            <span class="form-unit">หน่วยนับ</span>
                        </div>
                        <div class="form-actions">
                            <button
                                class="btn btn--ghost btn--sm"
                                @click="clearDrugForm"
                                type="button"
                            >
                                ล้าง
                            </button>
                            <button
                                class="btn btn--primary btn--sm"
                                @click="addOrUpdateDrug"
                                :disabled="!drugLookupDone"
                                type="button"
                            >
                                <Plus :size="13" />
                                {{
                                    selectedDrugIdx !== null
                                        ? "อัพเดต"
                                        : "เพิ่ม"
                                }}
                            </button>
                        </div>
                    </div>
                </template>

                <!-- ── Dept page ────────────────────────────────────────────── -->
                <template v-if="activePage === 'depts'">
                    <!-- Dept list -->
                    <div class="settings-list">
                        <div
                            v-for="(d, idx) in depts"
                            :key="d.code"
                            class="settings-list__item"
                            :class="{
                                'settings-list__item--selected':
                                    selectedDeptIdx === idx,
                            }"
                            @click="selectDept(idx)"
                        >
                            <div class="settings-list__main">
                                <span class="settings-list__primary">{{
                                    d.name
                                }}</span>
                                <span class="settings-list__secondary"
                                    >รหัส: {{ d.code }}</span
                                >
                            </div>
                            <button
                                class="settings-list__del"
                                @click.stop="deleteDept(idx)"
                                type="button"
                                title="ลบ"
                            >
                                <Trash2 :size="13" />
                            </button>
                        </div>
                        <div
                            v-if="depts.length === 0"
                            class="settings-list__empty"
                        >
                            ยังไม่มีรายการแผนก
                        </div>
                    </div>

                    <!-- Dept form -->
                    <div class="settings-form">
                        <h3 class="settings-form__title">
                            {{
                                selectedDeptIdx !== null
                                    ? "แก้ไขแผนก"
                                    : "เพิ่มแผนกใหม่"
                            }}
                        </h3>
                        <div class="form-row">
                            <label class="form-label">รหัสแผนก</label>
                            <div class="icode-wrap">
                                <input
                                    v-model="deptForm.code"
                                    class="form-input"
                                    type="text"
                                    placeholder="011"
                                    @input="onDeptCodeInput"
                                />
                                <button
                                    class="btn btn--ghost btn--sm"
                                    @click="lookupDeptName"
                                    :disabled="deptLookupLoading"
                                    type="button"
                                >
                                    <Loader
                                        v-if="deptLookupLoading"
                                        :size="12"
                                        class="spin"
                                    />
                                    <Search v-else :size="12" />
                                </button>
                            </div>
                        </div>
                        <div class="form-row">
                            <label class="form-label">ชื่อแผนก</label>
                            <!-- ชื่อแผนก — auto-filled, readonly -->
                            <input
                                v-model="deptForm.name"
                                class="form-input form-input--readonly"
                                type="text"
                                placeholder="กดค้นหารหัสแผนกก่อน"
                                readonly
                            />
                        </div>
                        <div class="form-actions">
                            <button
                                class="btn btn--ghost btn--sm"
                                @click="clearDeptForm"
                                type="button"
                            >
                                ล้าง
                            </button>
                            <button
                                class="btn btn--primary btn--sm"
                                @click="addOrUpdateDept"
                                :disabled="!deptLookupDone"
                                type="button"
                            >
                                <Plus :size="13" />
                                {{
                                    selectedDeptIdx !== null
                                        ? "อัพเดต"
                                        : "เพิ่ม"
                                }}
                            </button>
                        </div>
                    </div>
                </template>

                <!-- ── Lab rules page ──────────────────────────────────────── -->
                <template v-if="activePage === 'labs'">
                    <!-- Lab rules list -->
                    <div class="settings-list">
                        <div
                            v-for="(r, idx) in labRules"
                            :key="r.lab_items_code"
                            class="settings-list__item"
                            :class="{
                                'settings-list__item--selected':
                                    selectedLabIdx === idx,
                            }"
                            @click="selectLabRule(idx)"
                        >
                            <div class="settings-list__main">
                                <span class="settings-list__primary">
                                    {{ r.lab_items_name }}
                                    <span class="lab-code-badge">{{
                                        r.lab_items_code
                                    }}</span>
                                </span>
                                <span class="settings-list__secondary">
                                    แจ้งเตือนเมื่อ {{ ruleConditionLabel(r) }}
                                </span>
                            </div>
                            <button
                                class="settings-list__del"
                                @click.stop="deleteLabRule(idx)"
                                type="button"
                                title="ลบ"
                            >
                                <Trash2 :size="13" />
                            </button>
                        </div>
                        <div
                            v-if="labRules.length === 0"
                            class="settings-list__empty"
                        >
                            ยังไม่มีการตั้งค่าผลแลป
                        </div>
                    </div>

                    <!-- Lab rule form -->
                    <div class="settings-form">
                        <h3 class="settings-form__title">
                            <FlaskConical
                                :size="14"
                                style="
                                    display: inline;
                                    vertical-align: middle;
                                    margin-right: 4px;
                                "
                            />
                            {{
                                selectedLabIdx !== null
                                    ? "แก้ไขเกณฑ์ผลแลป"
                                    : "เพิ่มเกณฑ์ผลแลปใหม่"
                            }}
                        </h3>

                        <!-- Lab code + lookup -->
                        <div class="form-row">
                            <label class="form-label">รหัสแลป</label>
                            <div class="icode-wrap">
                                <input
                                    v-model="labForm.lab_items_code"
                                    class="form-input"
                                    type="text"
                                    placeholder="เช่น 659"
                                    @input="onLabCodeInput"
                                    @keydown.enter="lookupLabItemName"
                                />
                                <button
                                    class="btn btn--ghost btn--sm"
                                    @click="lookupLabItemName"
                                    :disabled="labLookupLoading"
                                    type="button"
                                    title="ค้นหาชื่อแลป"
                                >
                                    <Loader
                                        v-if="labLookupLoading"
                                        :size="12"
                                        class="spin"
                                    />
                                    <Search v-else :size="12" />
                                </button>
                            </div>
                        </div>

                        <!-- Lab item name (readonly, filled by lookup) -->
                        <div class="form-row">
                            <label class="form-label">ชื่อแลป</label>
                            <input
                                v-model="labForm.lab_items_name"
                                class="form-input form-input--readonly"
                                type="text"
                                placeholder="กดค้นหารหัสแลปก่อน"
                                readonly
                            />
                        </div>

                        <!-- Threshold value -->
                        <div class="form-row">
                            <label class="form-label">ค่าเกณฑ์</label>
                            <input
                                v-model="thresholdInput"
                                class="form-input form-input--short"
                                type="text"
                                placeholder="เช่น 130"
                            />
                        </div>

                        <!-- Condition checkboxes -->
                        <div class="form-row">
                            <label class="form-label">แจ้งเมื่อ</label>
                            <div class="lab-condition-group">
                                <label class="lab-cond-check">
                                    <input
                                        type="checkbox"
                                        v-model="labForm.compare_gt"
                                    />
                                    <span>มากกว่า (&gt;)</span>
                                </label>
                                <label class="lab-cond-check">
                                    <input
                                        type="checkbox"
                                        v-model="labForm.compare_eq"
                                    />
                                    <span>เท่ากับ (=)</span>
                                </label>
                                <label class="lab-cond-check">
                                    <input
                                        type="checkbox"
                                        v-model="labForm.compare_lt"
                                    />
                                    <span>น้อยกว่า (&lt;)</span>
                                </label>
                            </div>
                        </div>

                        <!-- Hint when no condition selected -->
                        <div
                            v-if="
                                labLookupDone &&
                                !labForm.compare_gt &&
                                !labForm.compare_eq &&
                                !labForm.compare_lt
                            "
                            class="lab-cond-hint"
                        >
                            กรุณาเลือกเงื่อนไขอย่างน้อย 1 รายการ
                        </div>

                        <div class="form-actions">
                            <button
                                class="btn btn--ghost btn--sm"
                                @click="clearLabForm"
                                type="button"
                            >
                                ล้าง
                            </button>
                            <button
                                class="btn btn--primary btn--sm"
                                @click="addOrUpdateLabRule"
                                :disabled="
                                    !labLookupDone ||
                                    (!labForm.compare_gt &&
                                        !labForm.compare_eq &&
                                        !labForm.compare_lt)
                                "
                                type="button"
                            >
                                <Plus :size="13" />
                                {{
                                    selectedLabIdx !== null ? "อัพเดต" : "เพิ่ม"
                                }}
                            </button>
                        </div>
                    </div>
                </template>

                <!-- ── Herb/Drug Interaction page ───────────────────────── -->
                <template v-if="activePage === 'interactions'">
                    <!-- List -->
                    <div class="settings-list">
                        <div
                            v-for="(rule, idx) in interactions"
                            :key="idx"
                            class="settings-list__item"
                            :class="{
                                'settings-list__item--selected':
                                    selectedInteractionIdx === idx,
                            }"
                            @click="selectInteraction(idx)"
                        >
                            <div class="settings-list__main">
                                <span class="settings-list__primary">
                                    {{ rule.modern_drug_name }}
                                </span>
                                <span class="settings-list__secondary">
                                    ห้ามใช้ร่วมกับ:
                                    {{
                                        rule.herb_drugs
                                            .map((h) => h.name)
                                            .join(", ")
                                    }}
                                </span>
                            </div>
                            <button
                                class="settings-list__del"
                                @click.stop="deleteInteraction(idx)"
                                type="button"
                                title="ลบ"
                            >
                                <Trash2 :size="13" />
                            </button>
                        </div>
                        <div
                            v-if="interactions.length === 0"
                            class="settings-list__empty"
                        >
                            ยังไม่มีการตั้งค่า
                        </div>
                    </div>

                    <!-- Form -->
                    <div class="settings-form">
                        <h3 class="settings-form__title">
                            {{
                                selectedInteractionIdx !== null
                                    ? "แก้ไข Interaction"
                                    : "เพิ่ม Herb/Drug Interaction"
                            }}
                        </h3>

                        <!-- Modern drug icode -->
                        <h4 class="settings-section-heading">
                            <Pill
                                :size="14"
                                style="
                                    vertical-align: middle;
                                    margin-right: 8px;
                                "
                            />ยาแผนปัจจุบัน
                        </h4>
                        <div class="form-row">
                            <label class="form-label">icode</label>
                            <div class="icode-wrap">
                                <input
                                    class="form-input"
                                    v-model="interactionModernIcode"
                                    @input="onModernIcodeInput"
                                    @keydown.enter.prevent="
                                        lookupModernDrugName
                                    "
                                    placeholder="เช่น 1490016"
                                    type="text"
                                />
                                <button
                                    class="btn btn--sm btn--ghost"
                                    @click="lookupModernDrugName"
                                    :disabled="
                                        !interactionModernIcode.trim() ||
                                        interactionModernLookupLoading
                                    "
                                    type="button"
                                    title="ค้นหาชื่อยา"
                                >
                                    <Loader
                                        v-if="interactionModernLookupLoading"
                                        :size="12"
                                        class="spin"
                                    />
                                    <Search v-else :size="12" />
                                </button>
                            </div>
                        </div>

                        <!-- Modern drug name (readonly) -->
                        <div class="form-row">
                            <label class="form-label">ชื่อยา</label>
                            <input
                                class="form-input form-input--readonly"
                                :value="interactionModernName"
                                readonly
                                placeholder="กดค้นหาเพื่อดึงชื่อยา"
                                type="text"
                            />
                        </div>

                        <!-- Herb drugs list -->
                        <h4 class="settings-section-heading">
                            <Leaf
                                :size="14"
                                style="
                                    vertical-align: middle;
                                    margin-right: 8px;
                                "
                            />ยาสมุนไพร
                        </h4>

                        <div
                            v-for="(draft, hidx) in herbDrugDrafts"
                            :key="hidx"
                        >
                            <div class="form-row">
                                <label class="form-label"></label>
                                <div class="icode-wrap">
                                    <input
                                        class="form-input"
                                        v-model="draft.icode"
                                        @input="onHerbIcodeInput(hidx)"
                                        @keydown.enter.prevent="
                                            lookupHerbDrugName(hidx)
                                        "
                                        placeholder="icode เช่น 1580004"
                                        type="text"
                                    />
                                    <button
                                        class="btn btn--sm btn--ghost"
                                        @click="lookupHerbDrugName(hidx)"
                                        :disabled="
                                            !draft.icode.trim() ||
                                            draft.lookupLoading
                                        "
                                        type="button"
                                        title="ค้นหาชื่อยา"
                                    >
                                        <Loader
                                            v-if="draft.lookupLoading"
                                            :size="12"
                                            class="spin"
                                        />
                                        <Search v-else :size="12" />
                                    </button>
                                </div>
                            </div>
                            <div class="form-row">
                                <label class="form-label"></label>
                                <input
                                    class="form-input form-input--readonly herb-name-input"
                                    :value="draft.name"
                                    readonly
                                    placeholder="กดค้นหาเพื่อดึงชื่อยา"
                                    type="text"
                                />
                                <button
                                    class="btn btn--sm settings-list__del"
                                    @click="removeHerbDrugEntry(hidx)"
                                    :disabled="herbDrugDrafts.length <= 1"
                                    type="button"
                                    title="ลบยาสมุนไพรนี้"
                                >
                                    <Trash2 :size="12" />
                                </button>
                            </div>
                        </div>
                        <div class="form-row">
                            <button
                                class="btn btn--sm btn--ghost"
                                @click="addHerbDrugEntry"
                                type="button"
                            >
                                <Plus :size="12" />
                                เพิ่มยาสมุนไพร
                            </button>
                        </div>

                        <!-- Reason -->
                        <div class="form-row">
                            <label class="form-label">เหตุผล / คำอธิบาย</label>
                            <textarea
                                class="form-input form-textarea"
                                v-model="interactionReason"
                                placeholder="เช่น ยา A ลดประสิทธิภาพของ ยา B"
                                rows="3"
                            />
                        </div>

                        <!-- Actions -->
                        <div class="form-actions">
                            <button
                                class="btn btn--sm btn--ghost"
                                @click="clearInteractionForm"
                                type="button"
                            >
                                ล้างฟอร์ม
                            </button>
                            <button
                                class="btn btn--sm btn--primary"
                                @click="addOrUpdateInteraction"
                                :disabled="
                                    !interactionModernLookupDone ||
                                    herbDrugDrafts.every((h) => !h.lookupDone)
                                "
                                type="button"
                            >
                                <Plus :size="13" />
                                {{
                                    selectedInteractionIdx !== null
                                        ? "อัพเดต"
                                        : "เพิ่ม"
                                }}
                            </button>
                        </div>
                    </div>
                </template>
            </div>

            <!-- Footer -->
            <div class="dialog__footer">
                <div class="dialog__footer-left">
                    <button
                        class="btn btn--ghost"
                        @click="importConfig"
                        type="button"
                    >
                        <Upload :size="13" />
                        นำเข้า
                    </button>
                    <button
                        class="btn btn--ghost"
                        @click="exportConfig"
                        type="button"
                    >
                        <Download :size="13" />
                        ส่งออก
                    </button>
                </div>
                <div class="dialog__footer-spacer" />
                <div class="dialog__footer-right">
                    <button
                        class="btn btn--ghost"
                        @click="emit('close')"
                        type="button"
                    >
                        ยกเลิก
                    </button>
                    <button
                        class="btn btn--primary"
                        @click="save"
                        :disabled="saving"
                        type="button"
                    >
                        <Loader v-if="saving" :size="13" class="spin" />
                        <Save v-else :size="13" />
                        บันทึก
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.dialog {
    background: var(--bg-input);
    border: 1.5px solid var(--border-default);
    border-radius: 14px;
    width: 640px;
    max-width: 96vw;
    max-height: 92vh;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.dialog--wide {
    width: 720px;
}

.dialog__header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 14px 16px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    color: var(--accent-primary-dark);
    flex-shrink: 0;
}

.dialog__title {
    flex: 1;
    font-size: 15px;
    font-weight: 700;
    color: var(--text-primary);
    font-family: var(--font-thai);
}

.dialog__close {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    padding: 3px;
    border-radius: 5px;
}
.dialog__close:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
}

/* Page tabs */
.app-settings__tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border-default);
    background: var(--bg-surface);
    flex-shrink: 0;
}

.app-tab-btn {
    padding: 8px 18px;
    border: none;
    background: none;
    font-family: var(--font-thai);
    font-size: 13px;
    color: var(--text-muted);
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition:
        color 0.1s,
        border-color 0.1s;
}
.app-tab-btn:hover {
    color: var(--text-primary);
}
.app-tab-btn--active {
    color: var(--accent-primary-dark);
    border-bottom-color: var(--accent-primary);
    font-weight: 700;
}

/* Body split */
.dialog__body--split {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
}

/* List panel */
.settings-list {
    width: 340px;
    flex-shrink: 0;
    overflow-y: auto;
    border-right: 1px solid var(--border-subtle);
    padding: 6px 0;
}

.settings-list__item {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    cursor: pointer;
    border-bottom: 1px solid var(--border-subtle);
    gap: 8px;
    transition: background 0.1s;
}
.settings-list__item:hover {
    background: var(--bg-row-alt);
}
.settings-list__item--selected {
    background: var(--drug-green-bg);
}
.settings-list__item--disabled {
    opacity: 0.55;
}
.settings-list__item--disabled .settings-list__primary {
    text-decoration: line-through;
    color: var(--text-muted);
}

.settings-list__main {
    flex: 1;
    min-width: 0;
}
.settings-list__primary {
    display: block;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    font-family: var(--font-thai);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.settings-list__secondary {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-thai);
    margin-top: 1px;
}

.disabled-badge {
    display: inline-block;
    background: #fee2e2;
    color: #b91c1c;
    font-size: 10px;
    font-weight: 600;
    border-radius: 4px;
    padding: 0 5px;
    margin-left: 4px;
    vertical-align: middle;
}

.settings-list__toggle {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    padding: 4px;
    border-radius: 5px;
    opacity: 0.6;
    transition:
        opacity 0.1s,
        color 0.1s;
    flex-shrink: 0;
}
.settings-list__toggle:hover {
    opacity: 1;
    color: var(--accent-primary-dark);
}

.settings-list__del {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    align-items: center;
    padding: 4px;
    border-radius: 5px;
    opacity: 0.5;
    transition:
        opacity 0.1s,
        color 0.1s;
    flex-shrink: 0;
}
.settings-list__del:hover {
    opacity: 1;
    color: var(--accent-red);
}

.settings-list__empty {
    padding: 20px;
    text-align: center;
    color: var(--text-muted);
    font-family: var(--font-thai);
    font-size: 13px;
}

/* Form panel */
.settings-form {
    flex: 1;
    padding: 16px 18px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.settings-form__title {
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
    font-family: var(--font-thai);
    margin: 0 0 4px;
}

.settings-section-heading {
    margin: 12px 0 6px;
    font-size: 13px;
    font-weight: 700;
    color: var(--accent-primary-dark);
    font-family: var(--font-thai);
    display: flex;
    align-items: center;
    gap: 8px;
}

.form-row {
    display: flex;
    align-items: center;
    gap: 8px;
}

.form-label {
    width: 72px;
    flex-shrink: 0;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    font-family: var(--font-thai);
}

.form-input {
    flex: 1;
    height: 32px;
    padding: 0 9px;
    border: 1.5px solid var(--border-default);
    border-radius: 7px;
    background: var(--bg-window);
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--font-thai);
    outline: none;
    transition: border-color 0.15s;
}
.form-input:focus {
    border-color: var(--border-focus);
}
.form-input:disabled {
    background: var(--bg-surface);
    color: var(--text-muted);
    cursor: not-allowed;
    opacity: 0.6;
}
.form-input--readonly {
    background: var(--bg-surface);
    color: var(--text-secondary);
    cursor: default;
}

/* Hide number input spinner arrows */
.form-input[type="number"]::-webkit-outer-spin-button,
.form-input[type="number"]::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
}
.form-input[type="number"] {
    -moz-appearance: textfield;
    appearance: textfield;
}

.form-unit {
    font-size: 12px;
    color: var(--text-muted);
    font-family: var(--font-thai);
    flex-shrink: 0;
}

.icode-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
}
.icode-wrap .form-input {
    flex: 1;
}

.form-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 4px;
}

/* Buttons */
.btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 8px 16px;
    border-radius: 9999px;
    font-size: 13px;
    font-family: var(--font-thai);
    font-weight: 600;
    cursor: pointer;
    transition:
        background 0.15s,
        color 0.15s,
        transform 0.1s;
    border: none;
    outline: none;
}
.btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}
.btn--sm {
    padding: 4px 12px;
    font-size: 12px;
}
.btn--primary {
    background: #9fe870;
    color: #163300;
    border: none;
}
.btn--primary:hover:not(:disabled) {
    background: #cdffad;
    transform: scale(1.05);
}
.btn--primary:active:not(:disabled) {
    transform: scale(0.95);
}
.btn--ghost {
    background: transparent;
    color: #454745;
    border: 1px solid rgba(14, 15, 12, 0.15);
}
.btn--ghost:hover:not(:disabled) {
    background: rgba(14, 15, 12, 0.06);
    transform: scale(1.05);
}

/* Footer */
.dialog__footer-left,
.dialog__footer-right {
    display: flex;
    gap: 8px;
    align-items: center;
}

.dialog__footer-spacer {
    flex: 1;
}

.dialog__footer {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    justify-content: flex-end;
    flex-shrink: 0;
}

.spin {
    animation: spin 1s linear infinite;
}
@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
/* Lab rule styles */
.lab-code-badge {
    display: inline-block;
    background: var(--bg-elevated, #e8f0e4);
    color: var(--text-muted);
    font-size: 10px;
    font-weight: 600;
    border-radius: 4px;
    padding: 0 5px;
    margin-left: 5px;
    vertical-align: middle;
}

.lab-condition-group {
    display: flex;
    gap: 14px;
    flex-wrap: wrap;
}

.lab-cond-check {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 13px;
    font-family: var(--font-thai);
    color: var(--text-primary);
    cursor: pointer;
    user-select: none;
}

.lab-cond-check input[type="checkbox"] {
    width: 14px;
    height: 14px;
    cursor: pointer;
    accent-color: var(--accent-primary, #9fe870);
}

.lab-cond-hint {
    font-size: 12px;
    color: #d03238;
    font-family: var(--font-thai);
    padding: 4px 0;
}

.form-input--short {
    max-width: 120px;
}

/* Herb/Drug interaction form */
.herb-drug-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 6px;
}

.herb-drug-row .icode-wrap {
    flex-shrink: 0;
    width: 170px;
}

.herb-name-input {
    flex: 1;
    min-width: 0;
}

.form-textarea {
    resize: vertical;
    min-height: 64px;
    width: 100%;
    font-family: var(--font-thai);
    line-height: 1.5;
}
</style>
