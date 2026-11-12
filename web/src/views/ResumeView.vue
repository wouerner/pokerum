<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { api } from '../api'
import { useAsync } from '../useAsync'
import LoadState from '../components/LoadState.vue'
import CodenameBadge from '../components/CodenameBadge.vue'
import ScoreBar from '../components/ScoreBar.vue'
import DeleteResume from '../components/DeleteResume.vue'
import DialogBox from '../components/DialogBox.vue'

const props = defineProps<{ codename: string }>()
const router = useRouter()
const { data, loading, error, run } = useAsync(() => api.getResume(props.codename))
onMounted(run)
watch(() => props.codename, run)
</script>

<template>
  <div class="container page narrow">
    <RouterLink to="/" class="back">← Vagas</RouterLink>
    <LoadState :loading="loading" :error="error" @retry="run">
      <template v-if="data">
        <section class="card profile">
          <p class="eyebrow">Perfil anônimo</p>
          <CodenameBadge :codename="data.codename" :alias="data.alias" size="lg" />
        </section>

        <section class="card">
          <h2>Candidaturas</h2>
          <ul v-if="data.applications.length" class="apps">
            <li v-for="a in data.applications" :key="a.job_slug">
              <RouterLink :to="`/vagas/${a.job_slug}/candidatos/${encodeURIComponent(data.codename)}`" class="app menu-item">
                <div class="app-head">
                  <strong class="app-title"><span class="cur pi pi-cursor" aria-hidden="true"></span>{{ a.job_title }}</strong>
                  <span class="muted"><b class="num app-pos">{{ a.position }}º</b> de {{ a.total }}</span>
                </div>
                <ScoreBar :score="a.score" />
              </RouterLink>
            </li>
          </ul>
          <DialogBox v-else speaker="POKERUM" compact class="apps-empty">Nenhuma candidatura por enquanto.</DialogBox>
        </section>

        <details class="card anon">
          <summary><span class="pi pi-cursor tri" aria-hidden="true"></span> Ver texto anonimizado</summary>
          <pre>{{ data.anonymized_text }}</pre>
        </details>

        <section class="card del-card">
          <h2>Excluir currículo</h2>
          <p class="muted">Remove o currículo e todas as candidaturas dele dos rankings. Exige o código de exclusão.</p>
          <DeleteResume :codename="data.codename" @deleted="router.push('/')" />
        </section>
      </template>
    </LoadState>
  </div>
</template>

<style scoped>
.del-card h2 {
  margin-top: 0;
}
.del-card p {
  margin: 8px 0 16px;
}
.narrow {
  max-width: 760px;
}
.card + .card {
  margin-top: 32px;
}
.profile .eyebrow {
  margin: 0 0 12px;
}
.apps {
  list-style: none;
  margin: 16px 0 0;
  padding: 0;
  display: grid;
  gap: 8px;
}
.apps-empty {
  margin-top: 16px;
}
.app {
  display: block;
  text-decoration: none;
  padding: 12px 14px 12px 8px;
  border: var(--px) solid var(--line-soft);
}
.app:hover {
  border-color: var(--frame-strong);
}
.app-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 4px 12px;
  margin-bottom: 8px;
}
.app-title {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
}
.app-title .cur {
  align-self: center;
}
.app-pos {
  font-size: 20px;
  color: var(--ink);
}
.anon summary {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 44px;
  cursor: pointer;
  font-weight: 700;
  list-style: none;
}
.anon summary::-webkit-details-marker {
  display: none;
}
.anon[open] .tri {
  transform: rotate(90deg);
}
.anon pre {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-family: var(--font);
  font-size: 16px;
  line-height: 1.6;
  color: var(--ink-2);
  margin: 14px 0 0;
  max-height: 480px;
  overflow: auto;
}
</style>
