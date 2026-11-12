<script setup lang="ts">
import type { CriterionScore } from '../api'
import ScoreBar from './ScoreBar.vue'

withDefaults(defineProps<{ items: CriterionScore[]; compact?: boolean }>(), { compact: false })
</script>

<template>
  <ul class="bd" :class="{ compact }">
    <li v-for="(c, i) in items" :key="`${c.key}-${i}`" class="bd-item">
      <div class="bd-head">
        <span class="bd-label">{{ c.label || c.key }}</span>
        <span v-if="c.weight > 0" class="bd-weight label">peso {{ Math.round(c.weight * 100) }}%</span>
      </div>
      <ScoreBar :score="c.score" :height="compact ? 8 : 12" />
      <p v-if="!compact && c.detail" class="bd-detail">{{ c.detail }}</p>
    </li>
  </ul>
</template>

<style scoped>
.bd {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 20px;
}
.bd.compact {
  gap: 12px;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 220px), 1fr));
  column-gap: 24px;
}
.bd-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 4px;
}
.bd-label {
  font-weight: 700;
}
.compact .bd-label {
  font-weight: 600;
}
.bd-weight {
  color: var(--ink-3);
  white-space: nowrap;
}
.bd-detail {
  margin: 6px 0 0;
  color: var(--ink-2);
}
</style>
