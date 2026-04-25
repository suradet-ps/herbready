<script setup lang="ts">
/**
 * AppSettingsDialog — Manage drug list and department list.
 * Two-tab layout: ยาสมุนไพร | แผนก
 */
import { ref, reactive, onMounted } from "vue";
import { X, Search as SearchIcon, Trash2, Plus } from "lucide-vue-next";
import { api } from "../../api/tauri";
import {
    appConfig as globalAppConfig,
    saveAppConfig,
} from "../../stores/appConfig";
import type { AppConfig, DrugConfig, DeptConfig } from "../../types";

const emit = defineEmits<{
    (e: "saved", config: AppConfig): void;
    (e: "close"): void;
}>();

const activeTab = ref<"drugs" | "depts">("drugs");
const saving = ref(false);
const drugs = ref<DrugConfig[]>([]);
const depts = ref<DeptConfig[]>([]);

const newDrug = reactive({
    icode: "",
    abbr: "",
    course_days: 30,
    capsules: 1,
    drug_name: "",
});
const newDept = reactive({ code: "", name: "" });
const lookingUpDrug = ref(false);
const lookingUpDept = ref(false);

onMounted(() => {
    // Start with a deep copy of the global config
    drugs.value = globalAppConfig.value.drugs.map((d) => ({ ...d }));
    depts.value = globalAppConfig.value.departments.map((d) => ({ ...d }));
});

async function lookupDrugName() {
    if (!newDrug.icode) return;
    lookingUpDrug.value = true;
    try {
        const name = await api.lookupDrugName(newDrug.icode);
        newDrug.drug_name = name;
    } catch (err: unknown) {
        alert(`ไม่พบยา: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
        lookingUpDrug.value = false;
    }
}

async function lookupDeptName() {
    if (!newDept.code) return;
    lookingUpDept.value = true;
    try {
        const name = await api.lookupDeptName(newDept.code);
        newDept.name = name;
    } catch (err: unknown) {
        alert(`ไม่พบแผนก: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
        lookingUpDept.value = false;
    }
}

function addDrug() {
    if (
        !newDrug.icode.trim() ||
        !newDrug.abbr.trim() ||
        !newDrug.drug_name.trim()
    )
        return;
    drugs.value.push({ ...newDrug });
    Object.assign(newDrug, {
        icode: "",
        abbr: "",
        course_days: 30,
        capsules: 1,
        drug_name: "",
    });
}

function removeDrug(idx: number) {
    drugs.value.splice(idx, 1);
}

function addDept() {
    if (!newDept.code.trim() || !newDept.name.trim()) return;
    depts.value.push({ ...newDept });
    Object.assign(newDept, { code: "", name: "" });
}

function removeDept(idx: number) {
    depts.value.splice(idx, 1);
}

async function save() {
    saving.value = true;
    try {
        const config: AppConfig = {
            drugs: drugs.value,
            departments: depts.value,
            lab_rules: globalAppConfig.value.lab_rules ?? [],
        };
        await saveAppConfig(config);
        emit("saved", config);
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
    <div class="overlay" @click.self="emit('close')">
        <div class="dialog" role="dialog" aria-modal="true">
            <!-- Header -->
            <div class="dialog__header">
                <span class="dialog__title">ตั้งค่าแอปพลิเคชัน</span>
                <button
                    class="dialog__close"
                    type="button"
                    @click="emit('close')"
                >
                    <X :size="15" />
                </button>
            </div>

            <!-- Tabs -->
            <div class="tab-bar">
                <button
                    class="tab-btn"
                    :class="{ 'tab-btn--active': activeTab === 'drugs' }"
                    type="button"
                    @click="activeTab = 'drugs'"
                >
                    🌿 ยาสมุนไพร ({{ drugs.length }})
                </button>
                <button
                    class="tab-btn"
                    :class="{ 'tab-btn--active': activeTab === 'depts' }"
                    type="button"
                    @click="activeTab = 'depts'"
                >
                    🏥 แผนก ({{ depts.length }})
                </button>
            </div>

            <!-- Body -->
            <div class="dialog__body">
                <!-- ─── DRUGS TAB ─────────────────────────────────────────────────── -->
                <template v-if="activeTab === 'drugs'">
                    <div class="table-scroll">
                        <table class="settings-table">
                            <thead>
                                <tr>
                                    <th>รหัสยา</th>
                                    <th>ตัวย่อ</th>
                                    <th class="right">ระยะ (วัน)</th>
                                    <th class="right">แคปซูล</th>
                                    <th>ชื่อยา</th>
                                    <th class="center"></th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr v-for="(drug, i) in drugs" :key="i">
                                    <td class="mono">{{ drug.icode }}</td>
                                    <td>{{ drug.abbr }}</td>
                                    <td class="right">
                                        {{ drug.course_days }}
                                    </td>
                                    <td class="right">{{ drug.capsules }}</td>
                                    <td>{{ drug.drug_name }}</td>
                                    <td class="center">
                                        <button
                                            class="icon-btn icon-btn--danger"
                                            type="button"
                                            @click="removeDrug(i)"
                                            title="ลบ"
                                        >
                                            <Trash2 :size="13" />
                                        </button>
                                    </td>
                                </tr>
                                <tr v-if="drugs.length === 0">
                                    <td colspan="6" class="empty-row">
                                        ยังไม่มีรายการยา
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>

                    <!-- Add drug row -->
                    <div class="add-form">
                        <span class="add-label">เพิ่มยา:</span>
                        <input
                            v-model="newDrug.icode"
                            class="mini-input"
                            style="width: 80px"
                            placeholder="รหัส"
                        />
                        <button
                            class="btn btn--lookup btn--sm"
                            type="button"
                            @click="lookupDrugName"
                            :disabled="lookingUpDrug"
                        >
                            <SearchIcon :size="12" />
                            ค้นหาชื่อ
                        </button>
                        <input
                            v-model="newDrug.abbr"
                            class="mini-input"
                            style="width: 64px"
                            placeholder="ย่อ"
                        />
                        <input
                            v-model.number="newDrug.course_days"
                            type="number"
                            class="mini-input"
                            style="width: 56px"
                            placeholder="วัน"
                        />
                        <input
                            v-model.number="newDrug.capsules"
                            type="number"
                            class="mini-input"
                            style="width: 56px"
                            placeholder="แคป"
                        />
                        <input
                            v-model="newDrug.drug_name"
                            class="mini-input flex1"
                            placeholder="ชื่อยา (จากการค้นหา)"
                        />
                        <button
                            class="btn btn--add btn--sm"
                            type="button"
                            @click="addDrug"
                        >
                            <Plus :size="13" /> เพิ่ม
                        </button>
                    </div>
                </template>

                <!-- ─── DEPTS TAB ─────────────────────────────────────────────────── -->
                <template v-if="activeTab === 'depts'">
                    <div class="table-scroll">
                        <table class="settings-table">
                            <thead>
                                <tr>
                                    <th style="width: 100px">รหัส</th>
                                    <th>ชื่อแผนก</th>
                                    <th class="center"></th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr v-for="(dept, i) in depts" :key="i">
                                    <td class="mono">{{ dept.code }}</td>
                                    <td>{{ dept.name }}</td>
                                    <td class="center">
                                        <button
                                            class="icon-btn icon-btn--danger"
                                            type="button"
                                            @click="removeDept(i)"
                                            title="ลบ"
                                        >
                                            <Trash2 :size="13" />
                                        </button>
                                    </td>
                                </tr>
                                <tr v-if="depts.length === 0">
                                    <td colspan="3" class="empty-row">
                                        ยังไม่มีรายการแผนก
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>

                    <!-- Add dept row -->
                    <div class="add-form">
                        <span class="add-label">เพิ่มแผนก:</span>
                        <input
                            v-model="newDept.code"
                            class="mini-input"
                            style="width: 100px"
                            placeholder="รหัส"
                        />
                        <button
                            class="btn btn--lookup btn--sm"
                            type="button"
                            @click="lookupDeptName"
                            :disabled="lookingUpDept"
                        >
                            <SearchIcon :size="12" />
                            ค้นหาชื่อ
                        </button>
                        <input
                            v-model="newDept.name"
                            class="mini-input flex1"
                            placeholder="ชื่อแผนก"
                        />
                        <button
                            class="btn btn--add btn--sm"
                            type="button"
                            @click="addDept"
                        >
                            <Plus :size="13" /> เพิ่ม
                        </button>
                    </div>
                </template>
            </div>

            <!-- Footer -->
            <div class="dialog__footer">
                <button
                    class="btn btn--ghost"
                    type="button"
                    @click="emit('close')"
                    :disabled="saving"
                >
                    ยกเลิก
                </button>
                <button
                    class="btn btn--primary"
                    type="button"
                    @click="save"
                    :disabled="saving"
                >
                    บันทึก
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    font-family: var(--font-thai, "Sarabun", "Tahoma", Arial, sans-serif);
}

.dialog {
    background: var(--bg-input, #fff);
    border: 1.5px solid var(--border-default, #bfefd3);
    border-radius: 14px;
    width: 740px;
    max-width: 97vw;
    max-height: 88vh;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.dialog__header {
    display: flex;
    align-items: center;
    padding: 13px 16px;
    background: var(--bg-surface, #e8fff1);
    border-bottom: 1px solid var(--border-default, #bfefd3);
}

.dialog__title {
    flex: 1;
    font-size: 15px;
    font-weight: 700;
    color: var(--text-primary, #1b2b23);
}

.dialog__close {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted, #6faf95);
    display: flex;
    align-items: center;
    padding: 3px;
    border-radius: 5px;
}
.dialog__close:hover {
    background: var(--bg-elevated, #d6f8e4);
    color: var(--text-primary, #1b2b23);
}

.tab-bar {
    display: flex;
    background: var(--bg-window, #f7fff9);
    border-bottom: 1px solid var(--border-default, #bfefd3);
}

.tab-btn {
    padding: 8px 20px;
    border: none;
    background: transparent;
    font-size: 13px;
    font-family: inherit;
    font-weight: 500;
    color: var(--text-muted, #6faf95);
    cursor: pointer;
    border-bottom: 2.5px solid transparent;
    transition: all 0.15s;
}
.tab-btn:hover:not(.tab-btn--active) {
    background: var(--bg-surface, #e8fff1);
    color: var(--text-secondary, #355f4a);
}
.tab-btn--active {
    color: var(--text-primary, #1b2b23);
    border-bottom-color: var(--border-focus, #34c38f);
    font-weight: 700;
}

.dialog__body {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    padding: 12px;
    gap: 10px;
}

.table-scroll {
    flex: 1;
    overflow-y: auto;
    border: 1.5px solid var(--border-default, #bfefd3);
    border-radius: 8px;
}

.settings-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
}

.settings-table thead tr {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--bg-elevated, #d6f8e4);
}

.settings-table th {
    padding: 7px 10px;
    text-align: left;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary, #355f4a);
    border-bottom: 1px solid var(--border-default, #bfefd3);
}

.settings-table td {
    padding: 6px 10px;
    border-bottom: 1px solid var(--bg-elevated, #d6f8e4);
    color: var(--text-primary, #1b2b23);
}
.settings-table tr:last-child td {
    border-bottom: none;
}
.settings-table tbody tr:hover td {
    background: var(--bg-surface, #e8fff1);
}

.mono {
    font-size: 12px;
    color: var(--text-secondary, #355f4a);
    font-weight: 600;
}
.right {
    text-align: right;
}
.center {
    text-align: center;
}
.empty-row {
    text-align: center;
    color: var(--text-muted, #6faf95);
    font-size: 13px;
    padding: 16px;
}

.icon-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 3px 5px;
    border-radius: 4px;
    display: inline-flex;
    align-items: center;
    color: var(--text-muted, #6faf95);
    transition: all 0.12s;
}
.icon-btn--danger:hover {
    color: #ef4444;
    background: #fee2e2;
}

.add-form {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    padding-top: 6px;
    border-top: 1px solid var(--border-default, #bfefd3);
}

.add-label {
    font-size: 12px;
    color: var(--text-muted, #6faf95);
    white-space: nowrap;
}

.mini-input {
    height: 30px;
    padding: 0 8px;
    border: 1.5px solid var(--border-default, #bfefd3);
    border-radius: 6px;
    background: var(--bg-window, #f7fff9);
    color: var(--text-primary, #1b2b23);
    font-size: 12px;
    font-family: inherit;
    outline: none;
    min-width: 0;
    transition: border-color 0.15s;
}
.mini-input:focus {
    border-color: var(--border-focus, #34c38f);
}
.flex1 {
    flex: 1;
    min-width: 120px;
}

.btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 32px;
    padding: 0 14px;
    border-radius: 7px;
    font-size: 13px;
    font-family: inherit;
    font-weight: 500;
    cursor: pointer;
    border: 1.5px solid transparent;
    transition: all 0.15s;
    white-space: nowrap;
}
.btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.btn--sm {
    height: 30px;
    padding: 0 10px;
    font-size: 12px;
}

.btn--primary {
    background: var(--accent-primary, #66de93);
    border-color: var(--accent-primary-dim, #4ccb7a);
    color: #14532d;
}
.btn--primary:hover:not(:disabled) {
    background: var(--accent-primary-dim, #4ccb7a);
}

.btn--ghost {
    background: transparent;
    border-color: var(--border-default, #bfefd3);
    color: var(--text-muted, #6faf95);
}
.btn--ghost:hover:not(:disabled) {
    background: var(--bg-elevated, #d6f8e4);
    color: var(--text-primary, #1b2b23);
}

.btn--lookup {
    background: var(--bg-surface, #e8fff1);
    border-color: var(--border-default, #bfefd3);
    color: var(--text-secondary, #355f4a);
}
.btn--lookup:hover:not(:disabled) {
    background: var(--bg-elevated, #d6f8e4);
}

.btn--add {
    background: #d9fbe8;
    border-color: #4ccb7a;
    color: #14532d;
}
.btn--add:hover:not(:disabled) {
    background: #c3f5d9;
}

.dialog__footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding: 10px 16px;
    border-top: 1px solid var(--border-default, #bfefd3);
    background: var(--bg-surface, #e8fff1);
}
</style>
