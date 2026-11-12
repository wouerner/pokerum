<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { api, DEFAULT_ENGINE, type RankingEntry } from '../api'
import { useAsync } from '../useAsync'
import { fmtDate } from '../util'
import LoadState from '../components/LoadState.vue'
import KeywordChips from '../components/KeywordChips.vue'
import RankingTable from '../components/RankingTable.vue'
import EngineBadge from '../components/EngineBadge.vue'
import DialogBox from '../components/DialogBox.vue'

const props = defineProps<{ slug: string }>()
const route = useRoute()
const router = useRouter()

const job = useAsync(() => api.getJob(props.slug))
const expanded = ref(false)

/** Motor escolhido via `?motor=ID` (compartilhável). */
const motorQuery = computed(() => {
  const m = route.query.motor
  const v = Array.isArray(m) ? m[0] : m
  return typeof v === 'string' && v ? v : null
})
const engines = computed(() => job.data.value?.engines ?? [])
const defaultEngine = computed(() => job.data.value?.ranking_engine ?? DEFAULT_ENGINE)
const currentEngineId = computed(() => motorQuery.value ?? defaultEngine.value)
const currentEngine = computed(() => engines.value.find((e) => e.id === currentEngineId.value) ?? null)

// Ranking: carregado manualmente para descartar respostas obsoletas ao trocar de motor.
const ranking = ref<RankingEntry[] | null>(null)
const rankingLoading = ref(false)
const rankingError = ref<string | null>(null)
let rankingSeq = 0
async function loadRanking() {
  const seq = ++rankingSeq
  rankingLoading.value = true
  rankingError.value = null
  try {
    const r = await api.getRanking(props.slug, motorQuery.value)
    if (seq === rankingSeq) ranking.value = r
  } catch (e) {
    if (seq === rankingSeq) rankingError.value = e instanceof Error ? e.message : String(e)
  } finally {
    if (seq === rankingSeq) rankingLoading.value = false
  }
}

function selectEngine(id: string) {
  if (id === currentEngineId.value) return
  const query = { ...route.query }
  if (id === defaultEngine.value) delete query.motor
  else query.motor = id
  router.replace({ query })
}

function load() {
  job.run()
  loadRanking()
}
onMounted(load)
watch(() => props.slug, load)
watch(motorQuery, (n, o) => {
  if (n !== o) loadRanking()
})
</script>

<template>
  <div class="container page">
    <RouterLink to="/" class="back">← Todas as vagas</RouterLink>
    <LoadState :loading="job.loading.value" :error="job.error.value" @retry="load">
      <template v-if="job.data.value">
        <div class="head">
          <div>
            <p class="eyebrow">{{ job.data.value.company || 'Empresa não informada' }} · publicada em {{ fmtDate(job.data.value.created_at) }}</p>
            <h1>{{ job.data.value.title }}</h1>
          </div>
          <RouterLink :to="`/vagas/${slug}/aplicar`" class="btn btn-primary btn-lg">Testar meu currículo →</RouterLink>
        </div>

        <div class="layout">
          <section class="card ranking">
            <div class="sec-head">
              <h2>Ranking público</h2>
              <span v-if="ranking && !rankingLoading" class="label muted">{{ ranking.length }} candidatos</span>
            </div>
            <div v-if="engines.length > 1" class="engine-pick">
              <p class="eyebrow">Algoritmo do ranking</p>
              <div class="seg px-frame" role="tablist" aria-label="Algoritmo do ranking">
                <button
                  v-for="e in engines"
                  :key="e.id"
                  type="button"
                  role="tab"
                  class="seg-btn menu-item"
                  :class="{ active: e.id === currentEngineId }"
                  :aria-selected="e.id === currentEngineId"
                  :title="e.description"
                  @click="selectEngine(e.id)"
                >
                  <span class="cur pi pi-cursor" aria-hidden="true"></span>
                  <EngineBadge :name="e.name" :kind="e.kind" :is-default="e.id === defaultEngine" />
                </button>
              </div>
              <p v-if="currentEngine?.description" class="muted engine-desc">{{ currentEngine.description }}</p>
              <p v-else-if="!currentEngine" class="muted engine-desc">
                Motor <b>{{ currentEngineId }}</b> não está ativo nesta vaga.
              </p>
            </div>
            <p v-else-if="engines.length === 1" class="muted engine-line">
              Ranking por: <EngineBadge :name="engines[0].name" :kind="engines[0].kind" />
            </p>
            <LoadState :loading="rankingLoading" :error="rankingError" @retry="loadRanking">
              <RankingTable v-if="ranking && ranking.length" :entries="ranking" :slug="slug" />
              <DialogBox v-else speaker="POKERUM" more>
                <p>Ninguém testou o currículo nesta vaga ainda. Que tal ser o primeiro?</p>
                <div class="dlg-actions">
                  <RouterLink :to="`/vagas/${slug}/aplicar`" class="btn btn-primary">Testar meu currículo</RouterLink>
                </div>
              </DialogBox>
            </LoadState>
          </section>

          <aside class="side">
            <section class="card">
              <h3 class="side-title">Palavras-chave extraídas</h3>
              <p class="muted">O que um ATS provavelmente procura no seu currículo.</p>
              <KeywordChips :keywords="job.data.value.keywords" empty="Nenhuma palavra-chave identificada." />
            </section>
            <section class="card">
              <h3 class="side-title">Descrição</h3>
              <div class="desc" :class="{ clamp: !expanded }">{{ job.data.value.description }}</div>
              <button type="button" class="btn btn-ghost small" @click="expanded = !expanded">
                {{ expanded ? 'Mostrar menos' : 'Ler descrição completa' }}
              </button>
            </section>
          </aside>
        </div>
      </template>
    </LoadState>
  </div>
</template>

<style scoped>
.engine-pick {
  margin-bottom: 20px;
}
.engine-pick .eyebrow {
  margin: 0 0 12px;
}
.seg {
  --fc: var(--frame-strong);
  display: flex;
  flex-wrap: wrap;
  background: var(--surface-2);
  margin: 4px;
}
.seg-btn {
  flex: 1 1 auto;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  min-height: 52px;
  font: inherit;
  color: var(--ink-2);
  background: transparent;
  border: none;
  border-right: var(--px) solid var(--line-soft);
  padding: 8px 14px 8px 8px;
  cursor: pointer;
  text-align: left;
}
.seg-btn:last-child {
  border-right: none;
}
.seg-btn:hover {
  color: var(--ink);
}
.seg-btn:focus-visible {
  outline-offset: calc(var(--px) * -4);
}
.seg-btn.active {
  color: var(--ink);
  background: var(--surface);
  box-shadow: inset 0 calc(var(--px) * -2) 0 var(--accent);
}
.engine-desc {
  margin: 12px 0 0;
}
.engine-line {
  margin: -6px 0 16px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: flex-end;
  gap: 20px;
  margin-bottom: 32px;
}
.head h1 {
  margin-top: 8px;
}
.layout {
  display: grid;
  grid-template-columns: minmax(0, 1.6fr) minmax(0, 1fr);
  gap: 32px;
  align-items: start;
}
.sec-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
}
.side {
  display: grid;
  gap: 32px;
}
.side-title {
  margin-bottom: 4px;
}
.side p {
  margin: 0 0 12px;
}
.desc {
  white-space: pre-wrap;
  color: var(--ink-2);
  margin: 12px 0;
  overflow-wrap: anywhere;
}
.desc.clamp {
  max-height: 220px;
  overflow: hidden;
  /* "dithering" em degraus em vez de degradê suave */
  -webkit-mask-image: linear-gradient(#000 0 70%, rgb(0 0 0 / 0.66) 70% 80%, rgb(0 0 0 / 0.33) 80% 90%, transparent 90%);
  mask-image: linear-gradient(#000 0 70%, rgb(0 0 0 / 0.66) 70% 80%, rgb(0 0 0 / 0.33) 80% 90%, transparent 90%);
}
.dlg-actions {
  margin-top: 16px;
}
@media (max-width: 820px) {
  .layout {
    grid-template-columns: 1fr;
  }
  .side {
    order: -1;
  }
  .head .btn {
    width: 100%;
    white-space: normal;
  }
}
</style>
