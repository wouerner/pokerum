<script setup lang="ts">
withDefaults(defineProps<{ keywords: string[]; variant?: 'neutral' | 'match' | 'miss'; empty?: string }>(), {
  variant: 'neutral',
  empty: 'Nenhuma palavra-chave.',
})
</script>

<template>
  <ul v-if="keywords.length" class="chips" :class="`chips-${variant}`">
    <li v-for="k in keywords" :key="k" class="chip">
      <span v-if="variant === 'match'" class="pi pi-check" aria-hidden="true"></span>
      <span v-else-if="variant === 'miss'" class="pi pi-cross" aria-hidden="true"></span>
      <span v-if="variant === 'match'" class="sr-only">encontrada:</span>
      <span v-else-if="variant === 'miss'" class="sr-only">faltando:</span>
      {{ k }}
    </li>
  </ul>
  <p v-else class="muted chips-empty">{{ empty }}</p>
</template>

<style scoped>
.chips {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  /* palavras-chave são conteúdo: grafia original, Nunito (§4) */
  font-family: var(--font);
  font-weight: 700;
  font-size: 14px;
  line-height: 1.4;
  padding: 4px 8px;
  border: var(--px) solid var(--frame-strong);
  background: var(--surface-2);
  color: var(--ink);
}
.chips-match .chip {
  color: var(--score-high-text);
  background: var(--score-high-bg);
  border-color: var(--score-high-text);
}
.chips-miss .chip {
  color: var(--score-low-text);
  background: var(--score-low-bg);
  border-color: var(--score-low-text);
}
.chips-empty {
  margin: 0;
}
</style>
