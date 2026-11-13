<script setup lang="ts">
/**
 * Aviso de cookies do Google Analytics (LGPD). Fica fixo no rodapé da tela até a pessoa escolher;
 * reabre pelo link "Cookies" do rodapé. Sem aceite, o GA não grava cookies.
 * Um espaçador no fim da página, da altura do aviso, garante que ele nunca esconda conteúdo.
 */
import { onBeforeUnmount, ref, watch } from 'vue'
import DialogBox from './DialogBox.vue'
import { choose, consentOpen } from '../analytics'

const bar = ref<HTMLElement | null>(null)
const spacer = ref(0)
let ro: ResizeObserver | null = null

watch(bar, (el) => {
  ro?.disconnect()
  ro = null
  if (!el) {
    spacer.value = 0
    return
  }
  const measure = () => (spacer.value = el.offsetHeight + 32)
  measure()
  if (typeof ResizeObserver !== 'undefined') {
    ro = new ResizeObserver(measure)
    ro.observe(el)
  }
})
onBeforeUnmount(() => ro?.disconnect())
</script>

<template>
  <div aria-live="polite">
    <template v-if="consentOpen">
      <div class="consent-spacer" :style="{ height: `${spacer}px` }" aria-hidden="true"></div>
      <div ref="bar" class="consent container">
        <DialogBox speaker="COOKIES" compact role="region" aria-label="Aviso de cookies">
          <p>
            Usamos cookies do Google Analytics só para saber quantas pessoas visitam o Pokerum.
            Nada é usado para anúncios.
          </p>
          <div class="dlg-actions">
            <button type="button" class="btn small" @click="choose('denied')">Recusar</button>
            <button type="button" class="btn btn-primary small" @click="choose('granted')">
              Aceitar
            </button>
          </div>
        </DialogBox>
      </div>
    </template>
  </div>
</template>

<style scoped>
.consent {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 16px;
  z-index: 20;
  max-width: 640px;
}
.dlg-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 12px;
}
</style>
