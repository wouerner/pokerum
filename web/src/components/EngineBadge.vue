<script setup lang="ts">
import { computed } from 'vue'
import { engineKindLabel } from '../util'

const props = withDefaults(
  defineProps<{ name: string; kind?: string | null; isDefault?: boolean; showKind?: boolean }>(),
  { kind: null, isDefault: false, showKind: true },
)

// Glifo + matiz por tipo de motor (sem bibliotecas de ícones).
const STYLES: Record<string, { glyph: string; hue: number }> = {
  heuristic: { glyph: 'ƒ', hue: 205 },
  llm: { glyph: '✦', hue: 285 },
  external_api: { glyph: '⇄', hue: 28 },
  database: { glyph: '▤', hue: 150 },
}
const style = computed(() => STYLES[props.kind ?? ''] ?? { glyph: '•', hue: 230 })
</script>

<template>
  <span class="eng" :style="{ '--h': style.hue }">
    <span class="eng-name">{{ name }}</span>
    <span v-if="showKind && kind" class="eng-kind">
      <span class="eng-glyph" aria-hidden="true">{{ style.glyph }}</span>
      {{ engineKindLabel(kind) }}
    </span>
    <span v-if="isDefault" class="eng-default">padrão</span>
  </span>
</template>

<style scoped>
.eng {
  display: inline-flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  min-width: 0;
  vertical-align: middle;
}
.eng-name {
  font-family: var(--font-label);
  font-synthesis: none;
  font-size: 16px;
  line-height: 1.25;
  text-transform: uppercase;
  overflow-wrap: anywhere;
}
.eng-kind,
.eng-default {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: var(--font);
  font-weight: 800;
  font-size: 14px;
  line-height: 1.4;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 0 6px;
  white-space: nowrap;
}
.eng-glyph {
  font-family: var(--font);
  font-weight: 700;
}
.eng-kind {
  color: hsl(var(--h) 75% var(--type-text-l));
  background: hsl(var(--h) 80% var(--type-bg-l));
  border: var(--px) solid hsl(var(--h) 60% var(--type-l));
}
.eng-default {
  color: var(--ink-2);
  border: var(--px) dashed var(--ink-3);
}
</style>
