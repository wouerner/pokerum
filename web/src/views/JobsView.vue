<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { api, DEFAULT_ENGINE, getEnginesCached } from '../api'
import { useAsync } from '../useAsync'
import { fmtDate, fmtScore, scoreTextColor } from '../util'
import LoadState from '../components/LoadState.vue'
import CodenameBadge from '../components/CodenameBadge.vue'
import DialogBox from '../components/DialogBox.vue'
import PixelArt from '../components/PixelArt.vue'
import { MUG, PLANT } from '../pixelart'

const { data: jobs, loading, error, run } = useAsync(() => api.listJobs())
onMounted(run)

// id → nome dos motores (só para o rótulo "por <motor>"; falha silenciosa).
const engineNames = ref<Record<string, string>>({})
onMounted(async () => {
  try {
    const list = await getEnginesCached()
    engineNames.value = Object.fromEntries((list ?? []).map((e) => [e.id, e.name]))
  } catch {
    /* backend antigo */
  }
})
function engineLabel(id: string): string {
  return engineNames.value[id] ?? id
}
</script>

<template>
  <div class="container page">
    <section class="hero">
      <div class="hero-text">
        <p class="eyebrow">Simulador de ATS · anônimo</p>
        <h1>Quanto seu currículo pontuaria nos <span class="hl">robôs de triagem</span>?</h1>
        <p class="lead muted">
          Escolha uma vaga, envie seu currículo e descubra sua posição entre os candidatos — com sugestões
          práticas para subir no ranking. Seu nome vira um alias e você ganha um codinome de Pokémon.
        </p>
      </div>
      <div class="cozy" aria-hidden="true">
        <PixelArt :rows="PLANT.rows" :palette="PLANT.palette" :scale="3" />
        <PixelArt :rows="MUG.rows" :palette="MUG.palette" :scale="3" />
      </div>
    </section>

    <div class="page-head">
      <h2>Vagas abertas</h2>
      <RouterLink to="/vagas/nova" class="btn btn-primary">+ Publicar vaga</RouterLink>
    </div>

    <LoadState :loading="loading" :error="error" @retry="run">
      <div v-if="jobs && jobs.length" class="grid">
        <RouterLink v-for="j in jobs" :key="j.id" :to="`/vagas/${j.slug}`" class="job card menu-item">
          <div class="job-top">
            <h3 class="job-title">
              <span class="cur pi pi-cursor" aria-hidden="true"></span>
              <span>{{ j.title }}</span>
            </h3>
            <p class="muted">{{ j.company || 'Empresa não informada' }}</p>
          </div>
          <ol v-if="j.top?.length" class="podium" aria-label="Top 3 candidatos">
            <li v-for="c in j.top" :key="c.codename" class="podium-row" :class="`p${c.position}`">
              <span class="podium-pos num">{{ c.position }}º</span>
              <CodenameBadge :codename="c.codename" :alias="c.alias" size="sm" :show-type="false" class="podium-who" />
              <span class="podium-score num" :style="{ color: scoreTextColor(c.score) }">{{ fmtScore(c.score) }}</span>
            </li>
          </ol>
          <p v-else class="podium-empty muted">Nenhum candidato ainda — seja o primeiro.</p>
          <p v-if="j.ranking_engine && j.ranking_engine !== DEFAULT_ENGINE" class="by-engine label">
            por {{ engineLabel(j.ranking_engine) }}
          </p>
          <div class="job-meta">
            <span class="pill label">
              {{ j.applicants }} {{ j.applicants === 1 ? 'candidato' : 'candidatos' }}
            </span>
            <span class="muted">{{ fmtDate(j.created_at) }}</span>
          </div>
        </RouterLink>
      </div>
      <DialogBox v-else speaker="POKERUM" more class="empty">
        <p>Nenhuma vaga por aqui ainda. Cole a descrição de uma vaga que te interessa e seja o primeiro a testar seu currículo.</p>
        <div class="dlg-actions">
          <RouterLink to="/vagas/nova" class="btn btn-primary">Publicar vaga</RouterLink>
        </div>
      </DialogBox>
    </LoadState>
  </div>
</template>

<style scoped>
.hero {
  position: relative;
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 32px;
  padding: 16px 0 40px;
}
.hero-text {
  max-width: 760px;
}
.hero h1 {
  margin: 12px 0 16px;
}
.hl {
  /* marca-texto em faixa sólida (sem degradê suave) */
  background: linear-gradient(transparent 60%, color-mix(in srgb, var(--accent) 35%, transparent) 60%);
}
.lead {
  font-size: 18px;
  margin: 0;
}
.cozy {
  flex: none;
  display: flex;
  align-items: flex-end;
  gap: 8px;
  padding: 0 8px 4px;
  border-bottom: calc(var(--px) * 2) solid var(--frame);
}
@media (max-width: 860px) {
  .cozy {
    display: none;
  }
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr));
  gap: 24px;
}
.job {
  text-decoration: none;
  display: flex;
  flex-direction: column;
  gap: 16px;
  transition:
    transform var(--step-fast) steps(2),
    box-shadow var(--step-fast) steps(2);
}
.job:hover {
  --sh: calc(var(--px) * 3);
  transform: translate(calc(var(--px) * -1), calc(var(--px) * -1));
}
.job:active {
  --sh: 0px;
  transform: translate(calc(var(--px) * 2), calc(var(--px) * 2));
}
.job {
  position: relative;
}
.job-title {
  margin-bottom: 4px;
}
/* o cursor ▶ fica no respiro esquerdo da janela, sem empurrar o título */
.job-title .cur {
  position: absolute;
  left: 0;
  top: 22px;
}
.job-top p {
  margin: 0;
}
.podium {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.podium-row {
  display: grid;
  grid-template-columns: 2.4em minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  background: var(--surface-2);
  border: var(--px) solid var(--line-soft);
}
.podium-row.p1 {
  border-color: var(--frame-strong);
}
.podium-pos {
  font-size: 20px;
  font-weight: 600;
}
.p1 .podium-pos {
  color: var(--accent-strong);
}
.podium-who {
  min-width: 0;
  overflow: hidden;
}
.podium-score {
  font-size: 24px;
  font-weight: 600;
  line-height: 1;
}
.podium-empty {
  margin: 0;
  padding: 10px;
  border: var(--px) dashed var(--line-soft);
  text-align: center;
}
.by-engine {
  margin: -8px 0 0;
  color: var(--ink-3);
  text-align: right;
}
.job-meta {
  margin-top: auto;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}
.pill {
  padding: 2px 8px;
  background: var(--surface-2);
  border: var(--px) solid var(--line-soft);
  color: var(--ink);
}
.empty {
  max-width: 720px;
}
.dlg-actions {
  margin-top: 16px;
}
</style>
