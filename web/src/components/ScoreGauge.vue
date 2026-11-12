<script setup lang="ts">
import { computed } from 'vue'
import { scoreTextColor, scoreTier, fmtScore } from '../util'
import ScoreBar from './ScoreBar.vue'

/** Medidor principal (§5.5): número grande + barra segmentada + rótulo de faixa. */
const props = withDefaults(defineProps<{ score: number; size?: number }>(), { size: 200 })

const pct = computed(() => Math.max(0, Math.min(100, props.score)))
const tier = computed(() => scoreTier(pct.value))
const label = computed(() => ({ low: 'Em risco', mid: 'Na disputa', high: 'Forte candidato' })[tier.value])
</script>

<template>
  <div class="gauge" :style="{ width: `${size}px` }" role="group" :aria-label="`Pontuação ${fmtScore(pct)} de 100: ${label}`">
    <div class="g-top" aria-hidden="true">
      <span class="g-num num" :style="{ color: scoreTextColor(pct) }">{{ fmtScore(pct) }}</span>
      <span class="g-of label">/100</span>
    </div>
    <ScoreBar :score="pct" :show-value="false" :height="20" class="g-bar" />
    <div class="g-label tag-px" :class="`g-${tier}`">{{ label }}</div>
  </div>
</template>

<style scoped>
.gauge {
  max-width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}
.g-top {
  display: flex;
  align-items: baseline;
  gap: 4px;
}
.g-num {
  font-size: 64px;
  font-weight: 600;
  line-height: 1;
}
.g-of {
  color: var(--ink-2);
}
.g-bar {
  width: 100%;
}
.g-label {
  font-size: 16px;
  padding: 4px 12px;
  border: var(--px) solid currentColor;
}
.g-high {
  color: var(--score-high-text);
  background: var(--score-high-bg);
}
.g-mid {
  color: var(--score-mid-text);
  background: var(--score-mid-bg);
}
.g-low {
  color: var(--score-low-text);
  background: var(--score-low-bg);
}
</style>
