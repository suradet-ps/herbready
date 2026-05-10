<script setup lang="ts">
/**
 * ConnectionBar — sticky footer showing hospital name (left) and DB connection status (right).
 */
import { computed } from "vue";
import { Wifi, WifiOff, Loader } from "lucide-vue-next";
import { connectionStatus, connectionMessage } from "../stores/connection";

const statusLabel = computed(() => {
    switch (connectionStatus.value) {
        case "connected":
            return "เชื่อมต่อแล้ว";
        case "connecting":
            return "กำลังเชื่อมต่อ…";
        case "disconnected":
            return "ยังไม่ได้เชื่อมต่อ";
        case "error":
            return "เชื่อมต่อล้มเหลว";
        default:
            return "ไม่ทราบสถานะ";
    }
});

const statusClass = computed(() => `status--${connectionStatus.value}`);
</script>

<template>
    <footer class="connection-bar" :class="statusClass">
        <div class="connection-bar__inner">
            <!-- LEFT: Hospital name -->
            <span class="connection-bar__hospital">
                © 2026 rxdevman. All rights reserved.
            </span>

            <!-- Spacer -->
            <span class="connection-bar__spacer" />

            <!-- RIGHT: Connection status -->
            <span class="connection-bar__icon">
                <Loader
                    v-if="connectionStatus === 'connecting'"
                    class="spin"
                    :size="14"
                />
                <Wifi v-else-if="connectionStatus === 'connected'" :size="14" />
                <WifiOff v-else :size="14" />
            </span>

            <span class="connection-bar__status">{{ statusLabel }}</span>

            <span class="connection-bar__sep" v-if="connectionMessage">|</span>

            <span class="connection-bar__msg" v-if="connectionMessage">
                {{ connectionMessage }}
            </span>
        </div>
    </footer>
</template>

<style scoped>
.connection-bar {
    height: 26px;
    border-top: 1px solid rgba(14, 15, 12, 0.1);
    background: #f5f7f3;
    flex-shrink: 0;
    transition: background 0.2s;
}

.connection-bar__inner {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 100%;
    padding: 0 12px;
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
    color: #868685;
}

.connection-bar__hospital {
    font-size: 11px;
    font-weight: 600;
    color: #868685;
    white-space: nowrap;
    flex-shrink: 0;
}

.connection-bar__spacer {
    flex: 1;
}

.connection-bar__icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
}

.connection-bar__status {
    font-weight: 600;
    flex-shrink: 0;
}

.connection-bar__sep {
    color: var(--border-default);
    flex-shrink: 0;
}

.connection-bar__msg {
    flex: 0 1 auto;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 400px;
}

/* Status-based colour overrides */
.status--connected .connection-bar__status {
    color: #054d28;
}
.status--connecting .connection-bar__status {
    color: #ffd11a;
}
.status--error .connection-bar__status {
    color: #d03238;
}
.status--disconnected .connection-bar__status {
    color: #868685;
}

.spin {
    animation: spin 1s linear infinite;
}
@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
