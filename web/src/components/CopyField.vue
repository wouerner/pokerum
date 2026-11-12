<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{ value: string }>()
const copied = ref(false)

async function copy() {
  try {
    await navigator.clipboard.writeText(props.value)
  } catch {
    const ta = document.createElement('textarea')
    ta.value = props.value
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    ta.remove()
  }
  copied.value = true
  setTimeout(() => (copied.value = false), 2000)
}
</script>

<template>
  <div class="cf">
    <code class="cf-val">{{ value }}</code>
    <button type="button" class="btn" @click="copy">{{ copied ? 'Copiado!' : 'Copiar' }}</button>
  </div>
</template>

<style scoped>
.cf {
  display: flex;
  gap: 12px;
  align-items: stretch;
  flex-wrap: wrap;
}
.cf-val {
  flex: 1 1 220px;
  min-width: 0;
  display: flex;
  align-items: center;
  min-height: 44px;
  font-family: var(--mono);
  font-size: 16px;
  padding: 8px 12px;
  background: var(--surface);
  border: calc(var(--px) * 2) dashed var(--frame-strong);
  word-break: break-all;
  user-select: all;
}
</style>
