<script setup lang="ts">
/**
 * AboutDialog — App info modal.
 */
import { X } from 'lucide-vue-next'

const emit = defineEmits<{
  (e: 'close'): void
}>()
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="dialog" role="dialog" aria-modal="true">
      <div class="dialog__header">
        <span class="dialog__title">เกี่ยวกับโปรแกรม</span>
        <button class="dialog__close" type="button" @click="emit('close')">
          <X :size="15" />
        </button>
      </div>

      <div class="dialog__body">
        <div class="logo-area">
          <span class="app-icon">🌿</span>
          <span class="app-name">HerbReady</span>
          <span class="app-version">เวอร์ชัน 2.0.0</span>
        </div>

        <p class="app-desc">
          ระบบจ่ายยาสมุนไพร<br />สำหรับคลินิกแพทย์แผนไทย
        </p>

        <div class="meta-table">
          <div class="meta-row">
            <span class="meta-key">พัฒนาโดย</span>
            <span class="meta-val">rxdevman</span>
          </div>
          <div class="meta-row">
            <span class="meta-key">ลิขสิทธิ์</span>
            <span class="meta-val">© 2026 rxdevman. All rights reserved.</span>
          </div>
          <div class="meta-row">
            <span class="meta-key">เทคโนโลยี</span>
            <span class="meta-val">Tauri 2 · Vue 3.5 · TypeScript</span>
          </div>
          <div class="meta-row">
            <span class="meta-key">เวอร์ชัน</span>
            <span class="meta-val">2.0.0</span>
          </div>
        </div>
      </div>

      <div class="dialog__footer">
        <button class="btn btn--primary" type="button" @click="emit('close')">ปิด</button>
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
  z-index: 1100;
  font-family: var(--font-thai, 'Sarabun', 'Tahoma', Arial, sans-serif);
}

.dialog {
  background: var(--bg-input, #fff);
  border: 1.5px solid var(--border-default, #BFEFD3);
  border-radius: 14px;
  width: 360px;
  max-width: 95vw;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.dialog__header {
  display: flex;
  align-items: center;
  padding: 13px 16px;
  background: var(--bg-surface, #E8FFF1);
  border-bottom: 1px solid var(--border-default, #BFEFD3);
}

.dialog__title {
  flex: 1;
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary, #1B2B23);
}

.dialog__close {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-muted, #6FAF95);
  display: flex;
  align-items: center;
  padding: 3px;
  border-radius: 5px;
}
.dialog__close:hover {
  background: var(--bg-elevated, #D6F8E4);
  color: var(--text-primary, #1B2B23);
}

.dialog__body {
  padding: 28px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
}

.logo-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.app-icon {
  font-size: 56px;
  line-height: 1;
  filter: drop-shadow(0 3px 8px rgba(102, 222, 147, 0.45));
}

.app-name {
  font-size: 24px;
  font-weight: 800;
  color: var(--text-primary, #1B2B23);
  letter-spacing: 0.5px;
}

.app-version {
  font-size: 12px;
  color: var(--text-muted, #6FAF95);
  background: var(--bg-elevated, #D6F8E4);
  padding: 2px 12px;
  border-radius: 12px;
  border: 1px solid var(--border-default, #BFEFD3);
  font-weight: 500;
}

.app-desc {
  margin: 0;
  font-size: 14px;
  color: var(--text-secondary, #355F4A);
  text-align: center;
  line-height: 1.7;
  border-top: 1px solid var(--border-default, #BFEFD3);
  border-bottom: 1px solid var(--border-default, #BFEFD3);
  padding: 14px 0;
  width: 100%;
}

.meta-table {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.meta-row {
  display: flex;
  gap: 10px;
  font-size: 12px;
}

.meta-key {
  width: 80px;
  flex-shrink: 0;
  color: var(--text-muted, #6FAF95);
  font-weight: 600;
}

.meta-val {
  color: var(--text-secondary, #355F4A);
}

.dialog__footer {
  display: flex;
  justify-content: center;
  padding: 12px 16px;
  border-top: 1px solid var(--border-default, #BFEFD3);
  background: var(--bg-surface, #E8FFF1);
}

.btn {
  display: inline-flex;
  align-items: center;
  height: 32px;
  padding: 0 32px;
  border-radius: 7px;
  font-size: 13px;
  font-family: inherit;
  font-weight: 600;
  cursor: pointer;
  border: 1.5px solid transparent;
  transition: all 0.15s;
}

.btn--primary {
  background: var(--accent-primary, #66DE93);
  border-color: var(--accent-primary-dim, #4CCB7A);
  color: #14532D;
}
.btn--primary:hover {
  background: var(--accent-primary-dim, #4CCB7A);
}
</style>
