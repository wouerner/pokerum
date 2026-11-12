<script setup lang="ts">
import { computed } from 'vue'
import { codenameType } from '../util'
import { creatureRows, CREATURE_PALETTE } from '../creatures'
import PixelArt from './PixelArt.vue'

/**
 * Ficha do candidato (§5.6): tile quadrado com um monstrinho ORIGINAL gerado a partir do
 * codinome (creatures.ts), codinome, apelido e selo de "tipo". Nunca a imagem do Pokémon (§9).
 */
const props = withDefaults(
  defineProps<{ codename: string; alias?: string; size?: 'sm' | 'md' | 'lg'; showType?: boolean }>(),
  { size: 'md', showType: true },
)

const t = computed(() => codenameType(props.codename))
const rows = computed(() => creatureRows(props.codename))
// escala inteira do sprite 16×16 dentro do tile (§3.1): sm/md 32px, lg 48px
const scale = computed(() => (props.size === 'lg' ? 3 : 2))
const style = computed(() => ({
  '--h': String(t.value.hue),
}))
</script>

<template>
  <span class="cn" :class="`cn-${size}`" :style="style">
    <span class="cn-tile" aria-hidden="true"><PixelArt :rows="rows" :palette="CREATURE_PALETTE" :scale="scale" /></span>
    <span class="cn-text">
      <span class="cn-name">{{ codename }}</span>
      <span v-if="alias" class="cn-alias">{{ alias }}</span>
    </span>
    <span v-if="showType" class="cn-type" :title="`Tipo: ${t.label}`">{{ t.label }}</span>
  </span>
</template>

<style scoped>
.cn {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
  max-width: 100%;
}
.cn-tile {
  flex: none;
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  overflow: hidden;
  background: hsl(var(--h) 80% var(--type-bg-l));
  border: calc(var(--px) * 2) solid hsl(var(--h) 60% var(--type-l));
  box-shadow: var(--shadow-sm);
}
.cn-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  line-height: 1.25;
}
.cn-name {
  font-family: var(--font-display);
  font-synthesis: none;
  font-weight: 700;
  font-size: 24px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.cn-alias {
  font-size: 16px;
  color: var(--ink-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.cn-type {
  flex: none;
  font-family: var(--font);
  font-weight: 800;
  font-size: 14px;
  line-height: 1.4;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 2px 8px;
  color: hsl(var(--h) 75% var(--type-text-l));
  background: hsl(var(--h) 80% var(--type-bg-l));
  border: var(--px) solid hsl(var(--h) 60% var(--type-l));
}
.cn-sm {
  gap: 10px;
}
.cn-sm .cn-tile {
  width: 36px;
  height: 36px;
  border-width: var(--px);
}
.cn-sm .cn-name {
  font-size: 20px;
}
.cn-lg .cn-tile {
  width: 64px;
  height: 64px;
  box-shadow: var(--shadow);
}
.cn-lg .cn-name {
  font-size: 32px;
}
.cn-lg .cn-alias {
  font-size: 18px;
}
@media (max-width: 480px) {
  .cn-lg {
    flex-wrap: wrap;
  }
}
</style>
