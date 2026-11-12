<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { api, saveOwnerToken, stashFreshApply } from '../api'
import { useAsync } from '../useAsync'
import DialogBox from '../components/DialogBox.vue'
import PixelArt from '../components/PixelArt.vue'
import { FLOPPY } from '../pixelart'

const props = defineProps<{ slug: string }>()
const router = useRouter()

const job = useAsync(() => api.getJob(props.slug))
onMounted(job.run)

const MAX = 5 * 1024 * 1024
const ACCEPT = ['.pdf', '.txt', '.md']

const mode = ref<'file' | 'text'>('file')
const file = ref<File | null>(null)
const text = ref('')
const name = ref('')
const dragging = ref(false)
const submitting = ref(false)
const error = ref<string | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)

function pick(f: File | undefined | null) {
  error.value = null
  if (!f) return
  const ext = f.name.slice(f.name.lastIndexOf('.')).toLowerCase()
  if (!ACCEPT.includes(ext)) {
    error.value = 'Formato não suportado. Envie um arquivo PDF, TXT ou MD.'
    return
  }
  if (f.size > MAX) {
    error.value = 'Arquivo maior que 5 MB.'
    return
  }
  file.value = f
}

function onDrop(e: DragEvent) {
  dragging.value = false
  pick(e.dataTransfer?.files?.[0])
}
function onChange(e: Event) {
  pick((e.target as HTMLInputElement).files?.[0])
}
function clearFile() {
  file.value = null
  if (fileInput.value) fileInput.value.value = ''
}

const ready = computed(() => (mode.value === 'file' ? !!file.value : text.value.replace(/\s/g, '').length >= 200))

function fmtSize(n: number) {
  return n < 1024 * 1024 ? `${Math.round(n / 1024)} KB` : `${(n / 1024 / 1024).toFixed(1)} MB`
}

async function submit() {
  if (!ready.value || submitting.value) return
  submitting.value = true
  error.value = null
  try {
    const res = await api.apply(props.slug, {
      file: mode.value === 'file' ? file.value : null,
      text: mode.value === 'text' ? text.value : undefined,
      name: name.value,
    })
    saveOwnerToken(res.codename, res.owner_token)
    stashFreshApply(props.slug, res)
    router.push(`/vagas/${props.slug}/candidatos/${encodeURIComponent(res.codename)}`)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="container page narrow">
    <RouterLink :to="`/vagas/${slug}`" class="back">← Voltar para a vaga</RouterLink>
    <p class="eyebrow">Testar currículo</p>
    <h1>{{ job.data.value?.title ?? 'Carregando vaga…' }}</h1>
    <p v-if="job.data.value?.company" class="muted company">{{ job.data.value.company }}</p>
    <DialogBox v-if="job.error.value" speaker="ERRO" variant="error" role="alert">{{ job.error.value }}</DialogBox>

    <form class="card stack form" @submit.prevent="submit">
      <div class="tabs" role="tablist">
        <button
          type="button"
          role="tab"
          class="menu-item"
          :aria-selected="mode === 'file'"
          :class="{ on: mode === 'file' }"
          @click="mode = 'file'"
        >
          <span class="cur pi pi-cursor" aria-hidden="true"></span>
          Enviar arquivo
        </button>
        <button
          type="button"
          role="tab"
          class="menu-item"
          :aria-selected="mode === 'text'"
          :class="{ on: mode === 'text' }"
          @click="mode = 'text'"
        >
          <span class="cur pi pi-cursor" aria-hidden="true"></span>
          Colar texto
        </button>
      </div>

      <div v-if="mode === 'file'" class="file-pane">
        <label
          class="drop"
          :class="{ dragging, has: !!file }"
          @dragover.prevent="dragging = true"
          @dragleave.prevent="dragging = false"
          @drop.prevent="onDrop"
        >
          <input ref="fileInput" type="file" class="sr-only" accept=".pdf,.txt,.md,application/pdf,text/plain,text/markdown" @change="onChange" />
          <PixelArt :rows="FLOPPY.rows" :palette="FLOPPY.palette" :scale="3" />
          <template v-if="file">
            <strong class="drop-title"><span class="pi pi-check ok" aria-hidden="true"></span> {{ file.name }}</strong>
            <span class="muted">{{ fmtSize(file.size) }}</span>
          </template>
          <template v-else>
            <strong class="drop-title">Arraste seu currículo aqui</strong>
            <span class="muted">ou clique para escolher · PDF, TXT ou MD · até 5 MB</span>
          </template>
        </label>
        <button v-if="file" type="button" class="btn btn-ghost small" @click="clearFile">Remover arquivo</button>
      </div>

      <div v-else class="field">
        <label for="cvtext">Texto do currículo</label>
        <textarea id="cvtext" v-model="text" rows="14" placeholder="Cole aqui o conteúdo do seu currículo…"></textarea>
        <span class="hint">{{ text.replace(/\s/g, '').length }} caracteres (sem espaços) · mínimo 200</span>
      </div>

      <div class="field">
        <label for="name">Seu nome <span class="muted">(opcional)</span></label>
        <input id="name" v-model="name" type="text" autocomplete="name" placeholder="Ex.: Maria da Silva" maxlength="120" />
        <span class="hint">Usado só para remover do texto, não é salvo.</span>
      </div>

      <DialogBox speaker="Privacidade" variant="info" more class="privacy">
        <p class="privacy-title">Como protegemos seus dados:</p>
        <ul>
          <li>Removemos do texto: <b>nome, e-mail, telefone, links, documentos (CPF/RG/CNPJ), endereço</b> e data de nascimento.</li>
          <li>Seu nome real é substituído por um <b>alias aleatório</b> e você recebe um <b>codinome de Pokémon</b> único.</li>
          <li>O <b>texto original não é salvo</b> — apenas a versão anonimizada, que aparece no ranking público.</li>
          <li>Você recebe um <b>código de exclusão</b> secreto para excluir seu currículo quando quiser.</li>
        </ul>
      </DialogBox>

      <DialogBox v-if="error" speaker="ERRO" variant="error" role="alert">{{ error }}</DialogBox>

      <button type="submit" class="btn btn-primary btn-lg submit" :disabled="!ready || submitting">
        <span v-if="submitting" class="spinner" aria-hidden="true"></span>
        {{ submitting ? 'Analisando seu currículo…' : 'Anonimizar e pontuar' }}
      </button>
    </form>
  </div>
</template>

<style scoped>
.narrow {
  max-width: 760px;
}
h1 {
  margin: 8px 0 4px;
}
.company {
  margin: 0;
}
.form {
  margin-top: 28px;
}
.tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 16px;
  border-bottom: var(--px) solid var(--line-soft);
}
.tabs button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-height: 44px;
  font-family: var(--font-label);
  font-synthesis: none;
  font-size: 16px;
  text-transform: uppercase;
  border: none;
  background: transparent;
  color: var(--ink-2);
  padding: 0 8px 0 0;
  cursor: pointer;
  margin-bottom: calc(var(--px) * -1);
  border-bottom: calc(var(--px) * 2) solid transparent;
}
.tabs button:hover {
  color: var(--ink);
}
.tabs button.on {
  color: var(--ink);
  border-bottom-color: var(--accent);
}
.file-pane {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
}
.drop {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  min-height: 200px;
  padding: 24px;
  text-align: center;
  border: calc(var(--px) * 2) dashed var(--ink-3);
  background: var(--surface-2);
  cursor: pointer;
  overflow-wrap: anywhere;
}
.drop:hover,
.drop.dragging {
  border-color: var(--ink);
  background: var(--surface);
}
.drop:focus-within {
  outline: calc(var(--px) * 2) dashed var(--ink);
  outline-offset: calc(var(--px) * 2);
}
.drop.has {
  border-style: solid;
  border-color: var(--score-high-text);
}
.drop-title {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
}
.ok {
  color: var(--score-high-text);
}
.privacy-title {
  font-weight: 700;
}
.privacy ul {
  margin: 8px 0 0;
  padding-left: 20px;
  font-size: 16px;
  line-height: 1.6;
}
.privacy li + li {
  margin-top: 6px;
}
.submit {
  width: 100%;
  white-space: normal;
}
.submit .spinner {
  width: 16px;
  height: 16px;
  border-width: 2px;
}
</style>
