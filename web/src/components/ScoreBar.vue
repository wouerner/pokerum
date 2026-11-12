<script setup lang="ts">
import { computed } from 'vue'
import { scoreColor, scoreTextColor, fmtScore } from '../util'

/**
 * Barra de nota segmentada (§5.4): 10 blocos de 10 pontos; o último bloco parcial
 * é preenchido em proporção (na grade de 2px). Enche em quadros ao montar.
 */
const props = withDefaults(defineProps<{ score: number; showValue?: boolean; height?: number }>(), {
  showValue: true,
  height: 10,
})
const pct = computed(() => Math.max(0, Math.min(100, props.score)))
// Altura sempre par (grade de 2px) e no mínimo 6px.
const h = computed(() => Math.max(6, Math.round(props.height / 2) * 2))
const cells = computed(() =>
  Array.from({ length: 10 }, (_, i) => Math.max(0, Math.min(1, (pct.value - i * 10) / 10))),
)
</script>

<template>
  <div class="sb" role="meter" :aria-valuenow="Math.round(pct)" aria-valuemin="0" aria-valuemax="100" :aria-label="`Nota ${fmtScore(pct)} de 100`">
    <div class="sb-track" :style="{ '--fill': scoreColor(pct) }">
      <span v-for="(f, i) in cells" :key="i" class="sb-cell" :style="{ height: `${h}px` }">
        <span v-if="f > 0" class="sb-f" :style="{ '--f': f, '--i': i }"></span>
      </span>
    </div>
    <span v-if="showValue" class="sb-val num" :style="{ color: scoreTextColor(pct) }">{{ fmtScore(pct) }}</span>
  </div>
</template>

<style scoped>
.sb {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
.sb-track {
  flex: 1;
  min-width: 80px;
  display: grid;
  grid-template-columns: repeat(10, minmax(0, 1fr));
  gap: var(--px);
  padding: var(--px);
  border: var(--px) solid var(--frame-strong);
  background: var(--surface);
}
.sb-cell {
  display: block;
  background: var(--line-soft);
  overflow: hidden;
}
.sb-f {
  display: block;
  height: 100%;
  width: calc(var(--f) * 100%);
  width: round(down, calc(var(--f) * 100%), 2px);
  background: var(--fill);
  /* cada bloco "acende" no seu quadro: 10 quadros em ~400ms (steps(10), §7) */
  animation: sb-hold calc((var(--i) + 1) * var(--step-slow) / 10) steps(1, end);
}
@keyframes sb-hold {
  from,
  to {
    visibility: hidden;
  }
}
.sb-val {
  font-size: 20px;
  font-weight: 600;
  line-height: 1;
  min-width: 3ch;
  text-align: right;
}
</style>
