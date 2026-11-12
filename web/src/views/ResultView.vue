<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import {
  api,
  DEFAULT_ENGINE,
  isEvaluationPending,
  takeFreshApply,
  type ApplicationResult,
  type ApplyResponse,
  type Evaluation,
} from '../api'
import { fmtScore, scoreTextColor, scoreTier } from '../util'
import EngineBadge from '../components/EngineBadge.vue'
import ScoreGauge from '../components/ScoreGauge.vue'
import BreakdownList from '../components/BreakdownList.vue'
import KeywordChips from '../components/KeywordChips.vue'
import CodenameBadge from '../components/CodenameBadge.vue'
import CopyField from '../components/CopyField.vue'
import LoadState from '../components/LoadState.vue'
import DeleteResume from '../components/DeleteResume.vue'
import DialogBox from '../components/DialogBox.vue'
import PixelArt from '../components/PixelArt.vue'
import { BRAND_MARK } from '../pixelart'

const props = defineProps<{ slug: string; codename: string }>()
const router = useRouter()

const result = ref<ApplicationResult | null>(null)
const fresh = ref<ApplyResponse | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

const anonText = ref<string | null>(null)
const anonLoading = ref(false)
const anonError = ref<string | null>(null)
const showText = ref(false)

// ---- polling enquanto houver avaliações pending/running ----
const POLL_MS = 3000
const POLL_MAX_MS = 5 * 60 * 1000
const polling = ref(false)
const pollTimedOut = ref(false)
let pollTimer: ReturnType<typeof setTimeout> | null = null
let pollGen = 0
let pollStartedAt = 0

function stopPolling() {
  pollGen++
  if (pollTimer) clearTimeout(pollTimer)
  pollTimer = null
  polling.value = false
}

function hasPending(r: ApplicationResult | null): boolean {
  return !!r && (r.evaluations ?? []).some(isEvaluationPending)
}

function maybePoll() {
  if (!hasPending(result.value)) {
    stopPolling()
    return
  }
  if (polling.value) return
  const gen = ++pollGen
  polling.value = true
  pollTimedOut.value = false
  pollStartedAt = Date.now()
  const tick = async () => {
    if (gen !== pollGen) return
    if (Date.now() - pollStartedAt > POLL_MAX_MS) {
      pollTimedOut.value = true
      stopPolling()
      return
    }
    try {
      const r = await api.getApplication(props.slug, props.codename)
      if (gen !== pollGen) return
      result.value = r
    } catch {
      /* falha transitória: tenta de novo no próximo ciclo */
    }
    if (gen !== pollGen) return
    if (hasPending(result.value)) pollTimer = setTimeout(tick, POLL_MS)
    else stopPolling()
  }
  pollTimer = setTimeout(tick, POLL_MS)
}

async function load() {
  stopPolling()
  pollTimedOut.value = false
  error.value = null
  anonText.value = null
  showText.value = false
  openEval.value = {}
  const f = takeFreshApply(props.slug, props.codename)
  if (f) {
    fresh.value = f
    result.value = {
      codename: f.codename,
      alias: f.alias,
      score: f.score,
      position: f.position,
      total: f.total,
      report: f.report,
      primary_engine: f.primary_engine,
      evaluations: f.evaluations,
    }
    anonText.value = f.anonymized_text
    maybePoll()
    return
  }
  fresh.value = null
  loading.value = true
  try {
    result.value = await api.getApplication(props.slug, props.codename)
    maybePoll()
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}
onMounted(load)
watch(() => [props.slug, props.codename], load)
onBeforeUnmount(stopPolling)

function retryPolling() {
  pollTimedOut.value = false
  maybePoll()
}

// ---- avaliações por motor ----
const evaluations = computed<Evaluation[]>(() => result.value?.evaluations ?? [])
const anyPending = computed(() => evaluations.value.some(isEvaluationPending))
const showEvaluations = computed(
  () => evaluations.value.length > 1 || evaluations.value.some((e) => e.status !== 'done'),
)
const openEval = ref<Record<string, boolean>>({})
function toggleEval(id: string) {
  openEval.value = { ...openEval.value, [id]: !openEval.value[id] }
}

function engineName(id: string): string {
  return evaluations.value.find((e) => e.engine_id === id)?.engine_name ?? id
}
function engineKind(id: string): string | null {
  return evaluations.value.find((e) => e.engine_id === id)?.engine_kind ?? null
}

/** Motor que produziu a nota principal (primary_engine, ou heuristic-v1 enquanto ele não termina). */
const shownEngine = computed(() => {
  const r = result.value
  if (!r) return null
  const primary = r.primary_engine
  if (!primary) return r.report.engine_version || null
  const ev = evaluations.value.find((e) => e.engine_id === primary)
  if (ev && ev.status !== 'done') return DEFAULT_ENGINE
  return primary
})
const primaryNotReady = computed(() => {
  const r = result.value
  if (!r?.primary_engine) return null
  const ev = evaluations.value.find((e) => e.engine_id === r.primary_engine)
  return ev && ev.status !== 'done' ? ev : null
})

async function toggleText() {
  showText.value = !showText.value
  if (showText.value && anonText.value === null && !anonLoading.value) {
    anonLoading.value = true
    anonError.value = null
    try {
      anonText.value = (await api.getResume(props.codename)).anonymized_text
    } catch (e) {
      anonError.value = e instanceof Error ? e.message : String(e)
    } finally {
      anonLoading.value = false
    }
  }
}

const verdict = computed(() => {
  if (!result.value) return ''
  const t = scoreTier(result.value.score)
  if (t === 'high') return 'Seu currículo tem boas chances de passar pela triagem automática desta vaga.'
  if (t === 'mid') return 'Você está na disputa, mas alguns ajustes podem te colocar no topo.'
  return 'Há grandes chances de um ATS filtrar seu currículo antes de um humano ler. Veja as sugestões abaixo.'
})
</script>

<template>
  <div class="container page">
    <RouterLink :to="`/vagas/${slug}`" class="back">← Voltar para o ranking da vaga</RouterLink>

    <LoadState :loading="loading" :error="error" @retry="load">
      <template v-if="result">
        <DialogBox v-if="fresh" speaker="Atenção" variant="warn" class="token" aria-live="polite">
          <h2>Seu código de exclusão</h2>
          <p>
            Guarde isto agora: ele aparece <b>só uma vez</b>. É pedido sempre que você quiser excluir seu currículo do
            Pokerum. Também foi salvo neste navegador, mas se você limpar os dados ou trocar de dispositivo, só o código
            copiado vai funcionar.
          </p>
          <CopyField :value="fresh.owner_token" />
        </DialogBox>

        <section class="hero card">
          <div class="hero-id">
            <p class="eyebrow">Codinome</p>
            <CodenameBadge :codename="result.codename" :alias="result.alias" size="lg" />
            <p class="rank">
              Você ficaria em <b class="rank-pos">{{ result.position }}º</b> de <b>{{ result.total }}</b>
              {{ result.total === 1 ? 'candidato' : 'candidatos' }}
            </p>
            <DialogBox speaker="Pokerum" more class="verdict">
              <template #portrait>
                <PixelArt :rows="BRAND_MARK.rows" :palette="BRAND_MARK.palette" :scale="3" />
              </template>
              {{ verdict }}
            </DialogBox>
            <p v-if="shownEngine" class="muted scored-by">
              Nota pelo algoritmo
              <EngineBadge :name="engineName(shownEngine)" :kind="engineKind(shownEngine)" />
            </p>
            <p v-if="primaryNotReady" class="muted scored-by">
              O algoritmo padrão desta vaga ({{ primaryNotReady.engine_name }})
              {{ isEvaluationPending(primaryNotReady) ? 'ainda está avaliando' : 'não conseguiu avaliar' }} — mostrando a nota
              heurística por enquanto.
            </p>
            <RouterLink :to="`/curriculos/${encodeURIComponent(result.codename)}`" class="link">
              Ver perfil do currículo e outras vagas →
            </RouterLink>
          </div>
          <ScoreGauge :score="result.score" :size="240" class="hero-gauge" />
        </section>

        <div class="grid">
          <section class="card">
            <h2>Pontuação por critério</h2>
            <p class="muted sub">Como um motor ATS ponderaria cada aspecto do seu currículo.</p>
            <BreakdownList :items="result.report.breakdown" />
          </section>

          <div class="col">
            <section class="card">
              <h2>Palavras-chave</h2>
              <h3 class="kw-h">Encontradas no seu currículo</h3>
              <KeywordChips :keywords="result.report.matched_keywords" variant="match" empty="Nenhuma palavra-chave da vaga encontrada." />
              <h3 class="kw-h">Faltando</h3>
              <KeywordChips :keywords="result.report.missing_keywords" variant="miss" empty="Nada importante faltando." />
            </section>

            <section class="card">
              <h2>Como melhorar</h2>
              <ol v-if="result.report.suggestions.length" class="tips">
                <li v-for="(s, i) in result.report.suggestions" :key="i">{{ s }}</li>
              </ol>
              <p v-else class="muted">Sem sugestões — seu currículo já está muito bem alinhado.</p>
            </section>
          </div>
        </div>

        <section v-if="showEvaluations" class="evals">
          <div class="evals-head">
            <h2>Avaliação por algoritmo</h2>
            <span v-if="polling" class="muted live">
              <span class="spinner spinner-xs" aria-hidden="true"></span> atualizando automaticamente
            </span>
            <button v-else-if="pollTimedOut && anyPending" type="button" class="btn btn-ghost small" @click="retryPolling">
              Verificar novamente
            </button>
          </div>
          <p class="muted sub">Cada algoritmo simula um tipo diferente de triagem automática.</p>
          <div class="evals-grid">
            <article
              v-for="ev in evaluations"
              :key="ev.engine_id"
              class="card ev"
              :class="`ev-${ev.status}`"
              :aria-busy="isEvaluationPending(ev)"
            >
              <div class="ev-head">
                <EngineBadge
                  :name="ev.engine_name || ev.engine_id"
                  :kind="ev.engine_kind"
                  :is-default="ev.engine_id === result.primary_engine"
                />
              </div>

              <DialogBox v-if="isEvaluationPending(ev)" speaker="Sistema" more compact aria-live="polite">
                <span class="ev-wait">
                  <span class="spinner" aria-hidden="true"></span>
                  Avaliando…
                </span>
              </DialogBox>

              <DialogBox v-else-if="ev.status === 'failed'" speaker="Erro" variant="error" compact role="status">
                <p><b>Não foi possível avaliar.</b></p>
                <p>{{ ev.error || 'Erro desconhecido.' }}</p>
              </DialogBox>

              <template v-else-if="ev.status === 'done' && ev.score !== null">
                <div class="ev-score">
                  <span class="ev-num num" :style="{ color: scoreTextColor(ev.score) }">{{ fmtScore(ev.score) }}</span>
                  <span class="muted label">/100</span>
                  <span v-if="ev.position !== null && ev.total !== null" class="ev-pos">
                    <b class="num">{{ ev.position }}º</b> de {{ ev.total }}
                  </span>
                </div>
                <div class="ev-actions">
                  <button
                    v-if="ev.report"
                    type="button"
                    class="btn btn-ghost small"
                    :aria-expanded="!!openEval[ev.engine_id]"
                    @click="toggleEval(ev.engine_id)"
                  >
                    <span class="pi pi-cursor tri" :class="{ open: openEval[ev.engine_id] }" aria-hidden="true"></span>
                    {{ openEval[ev.engine_id] ? 'Ocultar detalhes' : 'Ver detalhes' }}
                  </button>
                  <RouterLink :to="{ path: `/vagas/${slug}`, query: { motor: ev.engine_id } }" class="link">
                    ver ranking deste algoritmo →
                  </RouterLink>
                </div>
                <div v-if="ev.report && openEval[ev.engine_id]" class="ev-body">
                  <BreakdownList :items="ev.report.breakdown ?? []" />
                  <h3 class="kw-h">Encontradas</h3>
                  <KeywordChips :keywords="ev.report.matched_keywords ?? []" variant="match" empty="Nenhuma." />
                  <h3 class="kw-h">Faltando</h3>
                  <KeywordChips :keywords="ev.report.missing_keywords ?? []" variant="miss" empty="Nada importante faltando." />
                  <template v-if="ev.report.suggestions?.length">
                    <h3 class="kw-h">Sugestões</h3>
                    <ol class="tips">
                      <li v-for="(t, i) in ev.report.suggestions" :key="i">{{ t }}</li>
                    </ol>
                  </template>
                </div>
              </template>

              <p v-else class="muted">Sem resultado.</p>
            </article>
          </div>
        </section>

        <section class="card anon">
          <button type="button" class="anon-toggle" :aria-expanded="showText" @click="toggleText">
            <span class="anon-label"><span class="pi pi-cursor tri" :class="{ open: showText }" aria-hidden="true"></span> Ver texto anonimizado</span>
            <span class="muted anon-note">é isto que fica público</span>
          </button>
          <div v-if="showText" class="anon-body">
            <LoadState :loading="anonLoading" :error="anonError" @retry="toggleText">
              <pre class="anon-text">{{ anonText }}</pre>
            </LoadState>
          </div>
        </section>

        <footer class="meta">
          <span class="label muted">motor: {{ result.report.engine_version }}</span>
          <DeleteResume :codename="codename" @deleted="router.push(`/vagas/${slug}`)" />
        </footer>
      </template>
    </LoadState>
  </div>
</template>

<style scoped>
.scored-by {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin: 0 0 12px;
}
.evals {
  margin-top: 40px;
}
.evals-head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 8px 16px;
}
.live {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.spinner-xs {
  --q: 6px;
  width: 14px;
  height: 14px;
}
.evals-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr));
  gap: 24px;
  align-items: start;
}
.ev {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-width: 0;
}
.ev-pending,
.ev-running {
  --fc: var(--line-soft);
  --sh: 0px;
}
.ev-wait {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  padding-right: 16px;
}
.ev-score {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 4px 8px;
}
.ev-num {
  font-size: 48px;
  font-weight: 600;
  line-height: 1;
}
.ev-pos {
  margin-left: auto;
}
.ev-pos b {
  font-size: 24px;
  color: var(--accent-strong);
}
.ev-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}
.ev-body {
  border-top: var(--px) dashed var(--line-soft);
  padding-top: 16px;
}
.ev-body .kw-h:first-of-type {
  margin-top: 20px;
}
.tri.open {
  transform: rotate(90deg);
}
.token {
  margin-bottom: 32px;
}
.token h2 {
  margin: 0 0 8px;
}
.token p {
  margin: 0 0 16px;
  font-size: 16px;
  line-height: 1.6;
}
.hero {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 32px;
  margin-bottom: 32px;
}
.hero-id {
  flex: 1 1 340px;
  min-width: 0;
}
.hero-id .eyebrow {
  margin: 0 0 12px;
}
.rank {
  font-size: 18px;
  margin: 20px 0 16px;
}
.rank b {
  font-family: var(--font-num);
  font-weight: 600;
  font-synthesis: none;
  font-size: 24px;
}
.rank .rank-pos {
  font-size: 40px;
  color: var(--accent-strong);
}
.verdict {
  margin: 0 0 20px;
}
.hero-gauge {
  margin: 0 auto;
}
.grid {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr);
  gap: 32px;
  align-items: start;
}
.col {
  display: grid;
  gap: 32px;
}
.sub {
  margin: 4px 0 20px;
}
.kw-h {
  margin: 20px 0 8px;
  color: var(--ink-2);
}
.tips {
  margin: 12px 0 0;
  padding-left: 28px;
}
.tips li {
  margin-bottom: 10px;
}
.tips li::marker {
  font-family: var(--font-num);
  font-weight: 600;
  color: var(--accent-strong);
}
.anon {
  margin-top: 40px;
  padding: 0;
}
.anon-toggle {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  min-height: 56px;
  padding: 12px 20px;
  border: none;
  background: transparent;
  color: var(--ink);
  font: inherit;
  font-weight: 700;
  cursor: pointer;
  text-align: left;
}
.anon-label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.anon-note {
  font-weight: 400;
}
.anon-body {
  border-top: var(--px) dashed var(--line-soft);
  padding: 16px 20px;
}
.anon-text {
  margin: 0;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-family: var(--font);
  font-size: 16px;
  line-height: 1.6;
  max-height: 480px;
  overflow: auto;
  color: var(--ink-2);
}
.meta {
  margin-top: 32px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
}
@media (max-width: 820px) {
  .grid {
    grid-template-columns: 1fr;
  }
}
</style>
