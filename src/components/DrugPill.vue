<script setup lang="ts">
/**
 * DrugPill — a colour-coded badge for a single herb drug.
 * status: 'eligible' = green, 'never' = white/neutral, 'not_yet' = red/amber
 */
import { computed } from "vue";
import { formatDateThaiShort } from "../utils/dateHelper";

const props = defineProps<{
    drugName: string;
    abbr?: string;
    status: "eligible" | "never" | "not_yet";
    daysRemaining?: number | null;
    lastDispenseDate?: string;
    readyDate?: string;
    selected?: boolean;
    clickable?: boolean;
}>();

const emit = defineEmits<{
    (e: "toggle", name: string): void;
}>();

const pillClass = computed(() => {
    return [
        "drug-pill",
        `drug-pill--${props.status}`,
        props.selected ? "drug-pill--selected" : "",
        props.clickable ? "drug-pill--clickable" : "",
    ];
});

const tooltip = computed(() => {
    if (props.status === "not_yet" && props.daysRemaining != null) {
        const lines: string[] = [
            `${props.drugName} — อีก ${props.daysRemaining} วัน`,
        ];
        if (props.readyDate) {
            lines.push(`พร้อมจ่าย: ${formatDateThaiShort(props.readyDate)}`);
        }
        if (props.lastDispenseDate) {
            lines.push(
                `จ่ายล่าสุด: ${formatDateThaiShort(props.lastDispenseDate)}`,
            );
        }
        return lines.join("\n");
    }
    return props.drugName;
});

function handleClick() {
    if (props.clickable) {
        emit("toggle", props.drugName);
    }
}
</script>

<template>
    <button
        :class="pillClass"
        :title="tooltip"
        :aria-pressed="selected"
        @click="handleClick"
        type="button"
    >
        <span class="drug-pill__abbr">{{ abbr || drugName }}</span>
    </button>
</template>

<style scoped>
.drug-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 2px 6px;
    min-width: 36px;
    height: 22px;
    box-sizing: border-box;
    border-radius: 9999px;
    border: 1px solid;
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
    font-size: 11px;
    font-weight: 600;
    line-height: 1;
    cursor: default;
    white-space: nowrap;
    transition:
        opacity 0.15s,
        transform 0.1s,
        box-shadow 0.15s;
    user-select: none;
    background: none;
    outline: none;
}

.drug-pill--clickable {
    cursor: pointer;
}

.drug-pill--clickable:hover {
    opacity: 0.85;
    transform: translateY(-1px);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
}

.drug-pill--clickable:active {
    transform: translateY(0);
}

/* Eligible — green (ready to dispense) */
.drug-pill--eligible {
    background-color: #e2f6d5;
    border-color: #9fe870;
    color: #163300;
}

/* Never dispensed — soft gray-green (muted) */
.drug-pill--never {
    background-color: #f4fdf6;
    border-color: #d1f3dd;
    color: #3e6b4a;
}

/* Not yet eligible — light gray muted */
.drug-pill--not_yet {
    background-color: #f3f4f6;
    border-color: #e5e7eb;
    color: #6b7280;
}

/* Selected state — dark near-black with green text; distinct from all status variants */
.drug-pill--selected.drug-pill--eligible,
.drug-pill--selected.drug-pill--never {
    background-color: #0e0f0c;
    border-color: #0e0f0c;
    color: #9fe870;
    box-shadow: none;
}
</style>
