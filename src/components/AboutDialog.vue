<script setup lang="ts">
import { X, Leaf } from 'lucide-vue-next'

const emit = defineEmits<{
  (e: 'close'): void
}>()
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')">
    <div class="about-dialog">
      <button class="about-dialog__close" @click="emit('close')" type="button">
        <X :size="16" />
      </button>

      <div class="about-dialog__icon">
        <Leaf :size="44" />
      </div>

      <h1 class="about-dialog__title">HerbReady</h1>
      <p class="about-dialog__subtitle">ระบบจ่ายยาสมุนไพรไทย</p>
      <p class="about-dialog__version">เวอร์ชัน 2.0.0  (Tauri Edition)</p>

      <div class="about-dialog__sep"></div>

      <p class="about-dialog__desc">
        ระบบช่วยคัดกรองและจ่ายยาสมุนไพรสำหรับผู้ป่วยในแผนกแพทย์แผนไทย<br />
        รองรับการค้นหาด้วย HN, เลขบัตรประชาชน, และชื่อ-นามสกุล
      </p>

      <div class="about-dialog__sep"></div>

      <div class="about-dialog__footer">
        <span>© 2026 rxdevman. All rights reserved.</span>
      </div>
      <div class="about-dialog__footer" style="margin-top: 6px;">
        <span>Built with Tauri 2 + Vue 3.5 + TypeScript</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.about-dialog {
  background: var(--bg-input);
  border: 1.5px solid var(--border-default);
  border-radius: 16px;
  width: 360px;
  max-width: 94vw;
  box-shadow: 0 20px 60px rgba(0,0,0,0.2);
  padding: 28px 28px 20px;
  text-align: center;
  position: relative;
  font-family: var(--font-thai);
}

.about-dialog__close {
  position: absolute;
  top: 14px;
  right: 14px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  padding: 4px;
  border-radius: 5px;
}
.about-dialog__close:hover { background: var(--bg-elevated); color: var(--text-primary); }

.about-dialog__icon {
  color: var(--accent-primary-dark);
  margin-bottom: 12px;
  display: flex;
  justify-content: center;
}

.about-dialog__title {
  font-size: 26px;
  font-weight: 800;
  color: var(--text-primary);
  margin: 0 0 4px;
  letter-spacing: -0.5px;
}

.about-dialog__subtitle {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0 0 4px;
}

.about-dialog__version {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0 0 16px;
}

.about-dialog__sep {
  height: 1px;
  background: var(--border-subtle);
  margin: 12px 0;
}

.about-dialog__desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.7;
  margin: 0;
}

.about-dialog__footer {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
}
</style>
