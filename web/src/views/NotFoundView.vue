<script setup lang="ts">
import { RouterLink } from 'vue-router'
import DialogBox from '../components/DialogBox.vue'
import PixelArt from '../components/PixelArt.vue'
import { NOT_FOUND } from '../pixelart'
</script>

<template>
  <div class="container page nf">
    <div class="nf-art" aria-hidden="true">
      <PixelArt :rows="NOT_FOUND.rows" :palette="NOT_FOUND.palette" :scale="3" class="nf-hop" />
    </div>
    <p class="eyebrow">Erro 404</p>
    <h1>Esta página fugiu para o mato.</h1>
    <DialogBox speaker="SISTEMA" more class="nf-box">
      <p>Não encontramos o que você procurava. Que tal voltar para a lista de vagas?</p>
      <div class="dlg-actions">
        <RouterLink to="/" class="btn btn-primary">Ver vagas</RouterLink>
      </div>
    </DialogBox>
  </div>
</template>

<style scoped>
.nf {
  padding-top: 64px;
  max-width: 720px;
}
.nf h1 {
  margin: 8px 0 32px;
}
.dlg-actions {
  margin-top: 16px;
}
.nf-art {
  margin-bottom: 24px;
}
.nf-art :deep(svg) {
  width: 240px; /* 20 colunas × 12: escala inteira */
  height: auto;
}
/* pulinho em 2 quadros (§7); desligado por prefers-reduced-motion no style.css */
.nf-hop {
  animation: nf-hop 1.2s steps(1, end) infinite;
}
@keyframes nf-hop {
  0%,
  70% {
    transform: translateY(0);
  }
  80% {
    transform: translateY(-8px);
  }
  90% {
    transform: translateY(-4px);
  }
}
</style>
