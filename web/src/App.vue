<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import PixelArt from './components/PixelArt.vue'
import { BRAND_MARK } from './pixelart'
</script>

<template>
  <header class="topbar">
    <div class="container topbar-inner">
      <RouterLink to="/" class="brand" aria-label="Pokerum — início">
        <PixelArt :rows="BRAND_MARK.rows" :palette="BRAND_MARK.palette" :scale="2" />
        <span class="brand-name">Poke<b>rum</b></span>
      </RouterLink>
      <nav class="nav" aria-label="Principal">
        <RouterLink to="/" class="nav-link menu-item">
          <span class="cur pi pi-cursor" aria-hidden="true"></span>
          Vagas
        </RouterLink>
        <RouterLink to="/vagas/nova" class="btn btn-primary nav-cta">Publicar vaga</RouterLink>
      </nav>
    </div>
  </header>
  <main>
    <RouterView />
  </main>
  <footer class="footer">
    <div class="container muted">
      Pokerum simula como motores ATS com IA pontuariam seu currículo. Ranking público e anônimo —
      nenhum dado pessoal é exibido e o texto original não é salvo.
    </div>
  </footer>
</template>

<style scoped>
.topbar {
  border-bottom: calc(var(--px) * 2) solid var(--frame);
  background: var(--surface);
  position: sticky;
  top: 0;
  z-index: 10;
}
.topbar-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 64px;
  gap: 12px;
}
.brand {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 44px;
  text-decoration: none;
}
.brand-name {
  font-family: var(--font-display);
  font-synthesis: none;
  font-weight: 700;
  font-size: 24px;
  line-height: 1;
}
.brand b {
  color: var(--accent-strong);
  font-weight: 700;
}
.nav {
  display: flex;
  align-items: center;
  gap: 16px;
}
.nav-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-height: 44px;
  padding: 0 8px 0 4px;
  font-family: var(--font-label);
  font-synthesis: none;
  font-size: 16px;
  text-transform: uppercase;
  text-decoration: none;
}
.nav-link[aria-current='page'] {
  text-decoration: underline;
  text-decoration-thickness: calc(var(--px) * 2);
  text-underline-offset: 6px;
}
.footer {
  padding: 24px 0 40px;
  border-top: var(--px) dashed var(--line-soft);
}
@media (max-width: 420px) {
  .nav-link {
    display: none;
  }
}
</style>
