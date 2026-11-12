<script setup lang="ts">
import { computed } from 'vue'

/**
 * Pixel art própria em SVG (§5.10): cada string de `rows` é uma linha da grade,
 * cada caractere um pixel. '.' (ou espaço) é transparente; os demais são
 * mapeados para uma cor CSS em `palette` (tokens, para funcionar no modo escuro).
 * Renderiza com shape-rendering crispEdges e escala inteira (×2 ou ×3).
 */
const props = withDefaults(
  defineProps<{ rows: string[]; palette: Record<string, string>; scale?: 1 | 2 | 3; label?: string }>(),
  { scale: 2, label: undefined },
)

const w = computed(() => Math.max(...props.rows.map((r) => r.length)))
const h = computed(() => props.rows.length)

// Junta pixels vizinhos da mesma cor numa linha em um único <rect>.
const runs = computed(() => {
  const out: { x: number; y: number; w: number; fill: string }[] = []
  props.rows.forEach((row, y) => {
    let x = 0
    while (x < row.length) {
      const ch = row[x]
      let end = x + 1
      while (end < row.length && row[end] === ch) end++
      const fill = props.palette[ch]
      if (ch !== '.' && ch !== ' ' && fill) out.push({ x, y, w: end - x, fill })
      x = end
    }
  })
  return out
})
</script>

<template>
  <svg
    class="pxart"
    :viewBox="`0 0 ${w} ${h}`"
    :width="w * scale"
    :height="h * scale"
    shape-rendering="crispEdges"
    :role="label ? 'img' : undefined"
    :aria-label="label"
    :aria-hidden="label ? undefined : 'true'"
    focusable="false"
  >
    <rect v-for="(r, i) in runs" :key="i" :x="r.x" :y="r.y" :width="r.w" height="1" :style="{ fill: r.fill }" />
  </svg>
</template>

<style scoped>
.pxart {
  display: block;
  flex: none;
  image-rendering: pixelated;
}
</style>
