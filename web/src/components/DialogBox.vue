<script setup lang="ts">
/**
 * Caixa de diálogo de RPG (§5.2): janela larga com MOLDURA DUPLA (linha interna),
 * aba de "quem fala", retrato opcional à esquerda (slot `portrait`) e um ▼
 * piscando quando há continuação/ação. O texto é sempre Nunito (nunca pixel).
 * Atributos extras (role, aria-live…) caem na raiz.
 */
withDefaults(
  defineProps<{
    speaker?: string
    more?: boolean
    variant?: 'default' | 'error' | 'info' | 'warn'
    compact?: boolean
  }>(),
  { speaker: 'POKERUM', more: false, variant: 'default', compact: false },
)
</script>

<template>
  <div
    class="dlg px-frame"
    :class="[`dlg-${variant}`, { 'dlg-compact': compact, 'dlg-has-portrait': !!$slots.portrait }]"
  >
    <span v-if="speaker" class="dlg-tab">{{ speaker }}</span>
    <div class="dlg-main">
      <div v-if="$slots.portrait" class="dlg-portrait" aria-hidden="true">
        <slot name="portrait" />
      </div>
      <div class="dlg-body">
        <slot />
      </div>
    </div>
    <span v-if="more" class="dlg-more pi pi-down blink" aria-hidden="true"></span>
  </div>
</template>

<style scoped>
.dlg {
  --sh: calc(var(--px) * 2);
  position: relative;
  background: var(--surface);
  padding: 0 20px 20px;
  color: var(--ink);
  animation: px-enter var(--step-mid) steps(3);
}
/* linha interna da moldura dupla (frames 9-slice de diálogo) */
.dlg::before {
  content: '';
  position: absolute;
  inset: calc(var(--px) * 2);
  border: var(--px) solid var(--fc, var(--line));
  opacity: 0.35;
  pointer-events: none;
}
.dlg-main {
  position: relative;
}
.dlg-has-portrait .dlg-main {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 16px;
  align-items: start;
}
.dlg-portrait {
  display: grid;
  place-items: center;
  width: 56px;
  height: 56px;
  background: var(--surface-2);
  box-shadow: 0 0 0 var(--px) var(--line);
}
.dlg-tab {
  display: inline-block;
  margin: 0 0 12px -20px;
  padding: 4px 12px 4px 20px;
  font-family: var(--font-label);
  font-synthesis: none;
  font-size: 16px;
  line-height: 1.25;
  text-transform: uppercase;
  background: var(--ink);
  color: var(--surface);
}
.dlg-body {
  font-family: var(--font);
  font-size: 18px;
  line-height: 1.7;
  max-width: 70ch;
  overflow-wrap: anywhere;
}
.dlg-body :deep(p) {
  margin: 0;
}
.dlg-body :deep(p + p),
.dlg-body :deep(* + .dlg-actions) {
  margin-top: 12px;
}
.dlg-more {
  position: absolute;
  right: 8px;
  bottom: 6px;
  color: var(--ink);
}
.dlg-compact {
  padding: 0 16px 16px;
}
.dlg-compact .dlg-tab {
  margin-left: -16px;
  padding-left: 16px;
}
.dlg-compact .dlg-body {
  font-size: 16px;
  line-height: 1.6;
}

.dlg-error {
  --fc: var(--score-low-text);
  --shc: var(--score-low-text);
  background: var(--score-low-bg);
}
.dlg-error .dlg-tab {
  background: var(--score-low-text);
  color: var(--score-low-bg);
}
.dlg-warn {
  --fc: var(--score-mid-text);
  --shc: var(--score-mid-text);
  background: var(--score-mid-bg);
}
.dlg-warn .dlg-tab {
  background: var(--score-mid-text);
  color: var(--score-mid-bg);
}
.dlg-info {
  background: var(--surface-2);
}
</style>
