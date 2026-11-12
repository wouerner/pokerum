<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'
import type { RankingEntry } from '../api'
import CodenameBadge from './CodenameBadge.vue'
import ScoreBar from './ScoreBar.vue'
import BreakdownList from './BreakdownList.vue'

defineProps<{ entries: RankingEntry[]; slug: string; highlight?: string }>()
const open = ref<string | null>(null)
function toggle(c: string) {
  open.value = open.value === c ? null : c
}
</script>

<template>
  <ol class="rk">
    <li
      v-for="e in entries"
      :key="e.codename"
      class="rk-row menu-item"
      :class="{ 'is-open': open === e.codename, 'is-me': highlight === e.codename, [`top-${e.position}`]: e.position <= 3 }"
    >
      <div class="rk-main">
        <span class="cur pi pi-cursor" aria-hidden="true"></span>
        <span class="rk-pos num">{{ e.position }}º</span>
        <RouterLink :to="`/vagas/${slug}/candidatos/${e.codename}`" class="rk-who">
          <CodenameBadge :codename="e.codename" :alias="e.alias" size="sm" :show-type="false" />
        </RouterLink>
        <ScoreBar :score="e.score" class="rk-bar" />
        <button
          type="button"
          class="rk-toggle"
          :aria-expanded="open === e.codename"
          :aria-label="open === e.codename ? 'Ocultar detalhes' : 'Ver detalhes da pontuação'"
          @click="toggle(e.codename)"
        >
          {{ open === e.codename ? '-' : '+' }}
        </button>
      </div>
      <div v-if="open === e.codename" class="rk-detail">
        <BreakdownList :items="e.breakdown" compact />
      </div>
    </li>
  </ol>
</template>

<style scoped>
.rk {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 8px;
}
.rk-row {
  border: var(--px) solid var(--line-soft);
  background: var(--surface);
}
.rk-row.is-open {
  border-color: var(--frame-strong);
}
.rk-row.is-me {
  border: calc(var(--px) * 2) solid var(--accent-strong);
}
.rk-main {
  display: grid;
  grid-template-columns: 16px 3ch minmax(0, 1.3fr) minmax(100px, 1fr) 44px;
  align-items: center;
  gap: 12px;
  padding: 8px 12px 8px 8px;
}
.cur {
  color: var(--ink);
}
.rk-pos {
  font-size: 20px;
  font-weight: 600;
  color: var(--ink-2);
  text-align: right;
}
.top-1 .rk-pos {
  color: var(--accent-strong);
}
.top-2 .rk-pos,
.top-3 .rk-pos {
  color: var(--ink);
}
.rk-who {
  display: flex;
  align-items: center;
  min-height: 44px;
  text-decoration: none;
  min-width: 0;
}
.rk-who:hover :deep(.cn-name) {
  text-decoration: underline;
  text-decoration-thickness: var(--px);
}
.rk-toggle {
  width: 44px;
  height: 44px;
  border: var(--px) solid var(--frame-strong);
  border-radius: 0;
  background: var(--surface-2);
  color: var(--ink);
  font-family: var(--font-label);
  font-synthesis: none;
  font-size: 16px;
  cursor: pointer;
  box-shadow: var(--shadow-sm);
  transition:
    transform var(--step-fast) steps(2),
    box-shadow var(--step-fast) steps(2);
}
.rk-toggle:hover {
  transform: translate(calc(var(--px) * -1), calc(var(--px) * -1));
  box-shadow: calc(var(--px) * 3) calc(var(--px) * 3) 0 var(--shadow-color);
}
.rk-toggle:active {
  transform: translate(calc(var(--px) * 2), calc(var(--px) * 2));
  box-shadow: none;
}
.rk-detail {
  border-top: var(--px) dashed var(--line-soft);
  padding: 12px 16px 16px;
}
@media (max-width: 560px) {
  .rk-main {
    grid-template-columns: 16px 3ch minmax(0, 1fr) 44px;
    row-gap: 4px;
    column-gap: 8px;
  }
  .rk-bar {
    grid-column: 3 / 5;
    grid-row: 2;
  }
}
</style>
