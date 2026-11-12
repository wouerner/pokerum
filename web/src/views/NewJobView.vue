<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { api, DEFAULT_ENGINE, getEnginesCached, isEngineActive, type Engine } from '../api'
import { engineKindLabel } from '../util'
import DialogBox from '../components/DialogBox.vue'

const router = useRouter()
const title = ref('')
const company = ref('')
const description = ref('')
const submitting = ref(false)
const error = ref<string | null>(null)

// Motores ativos — o select só aparece quando há mais de um.
const engines = ref<Engine[]>([])
const rankingEngine = ref(DEFAULT_ENGINE)
onMounted(async () => {
  try {
    const all = await getEnginesCached()
    engines.value = (all ?? []).filter(isEngineActive)
    if (!engines.value.some((e) => e.id === rankingEngine.value) && engines.value.length) {
      rankingEngine.value = engines.value[0].id
    }
  } catch {
    engines.value = [] // backend antigo: segue sem o campo
  }
})
const showEngineSelect = computed(() => engines.value.length > 1)
const selectedEngine = computed(() => engines.value.find((e) => e.id === rankingEngine.value) ?? null)

const valid = computed(() => title.value.trim().length >= 2 && description.value.trim().length >= 30)

async function submit() {
  if (!valid.value || submitting.value) return
  submitting.value = true
  error.value = null
  try {
    const job = await api.createJob({
      title: title.value.trim(),
      company: company.value.trim() || undefined,
      description: description.value.trim(),
      ranking_engine: showEngineSelect.value ? rankingEngine.value : undefined,
    })
    router.push(`/vagas/${job.slug}`)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="container page narrow">
    <RouterLink to="/" class="back">← Voltar para vagas</RouterLink>
    <h1>Publicar vaga</h1>
    <p class="muted intro">
      Cole a descrição completa da vaga. Quanto mais detalhes (requisitos, tecnologias, senioridade, idiomas),
      mais fiel será a simulação do ATS.
    </p>

    <form class="card stack" @submit.prevent="submit">
      <div class="field">
        <label for="title">Título da vaga</label>
        <input id="title" v-model="title" type="text" placeholder="Ex.: Desenvolvedor(a) Back-end Sênior" required maxlength="200" />
      </div>
      <div class="field">
        <label for="company">Empresa <span class="muted">(opcional)</span></label>
        <input id="company" v-model="company" type="text" placeholder="Ex.: Acme Ltda." maxlength="200" />
      </div>
      <div class="field">
        <label for="desc">Descrição</label>
        <textarea id="desc" v-model="description" rows="16" placeholder="Responsabilidades, requisitos, diferenciais…" required></textarea>
        <span class="hint">{{ description.trim().length }} caracteres · mínimo 30</span>
      </div>
      <div v-if="showEngineSelect" class="field">
        <label for="engine">Algoritmo do ranking</label>
        <div class="select-wrap">
          <select id="engine" v-model="rankingEngine">
            <option v-for="e in engines" :key="e.id" :value="e.id">
              {{ e.name }} — {{ engineKindLabel(e.kind) }}{{ e.id === DEFAULT_ENGINE ? ' (padrão)' : '' }}
            </option>
          </select>
        </div>
        <span class="hint">
          {{ selectedEngine?.description }}
          Todos os algoritmos avaliam cada currículo; este define a ordem do ranking principal.
        </span>
      </div>
      <DialogBox v-if="error" speaker="ERRO" variant="error" role="alert">{{ error }}</DialogBox>
      <div class="actions">
        <button type="submit" class="btn btn-primary btn-lg" :disabled="!valid || submitting">
          <span v-if="submitting" class="spinner" aria-hidden="true"></span>
          {{ submitting ? 'Publicando…' : 'Publicar vaga' }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.narrow {
  max-width: 760px;
}
.intro {
  margin: 12px 0 28px;
  font-size: 18px;
}
.actions {
  display: flex;
  justify-content: flex-end;
}
.actions .spinner {
  width: 16px;
  height: 16px;
  border-width: 2px;
}
</style>
