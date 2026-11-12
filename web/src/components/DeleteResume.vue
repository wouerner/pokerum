<script setup lang="ts">
import { computed, ref } from 'vue'
import { api, ApiError, getOwnerToken, removeOwnerToken } from '../api'
import DialogBox from './DialogBox.vue'

const props = defineProps<{ codename: string }>()
const emit = defineEmits<{ deleted: [] }>()

const open = ref(false)
const token = ref('')
const deleting = ref(false)
const error = ref<string | null>(null)

const savedToken = computed(() => (open.value ? getOwnerToken(props.codename) : null))
const ready = computed(() => token.value.trim().length > 0 && !deleting.value)

function toggle() {
  open.value = !open.value
  token.value = ''
  error.value = null
}

async function submit() {
  if (!ready.value) return
  if (!confirm(`Excluir definitivamente o currículo ${props.codename} e todas as candidaturas dele? Essa ação não pode ser desfeita.`)) return
  deleting.value = true
  error.value = null
  try {
    await api.deleteResume(props.codename, token.value.trim())
    removeOwnerToken(props.codename)
    emit('deleted')
  } catch (e) {
    if (e instanceof ApiError && e.status === 403) error.value = 'Código de exclusão inválido.'
    else error.value = e instanceof Error ? e.message : String(e)
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <div class="delr">
    <button v-if="!open" type="button" class="btn btn-danger" @click="toggle">Excluir meu currículo</button>

    <form v-else class="delr-form" @submit.prevent="submit">
      <div class="field">
        <label :for="`del-token-${codename}`">Código de exclusão</label>
        <input
          :id="`del-token-${codename}`"
          v-model="token"
          type="text"
          class="mono"
          autocomplete="off"
          spellcheck="false"
          placeholder="Cole aqui o código recebido ao enviar o currículo"
        />
        <span class="hint">
          É o código que apareceu uma única vez depois do envio.
          <button v-if="savedToken" type="button" class="linkish" @click="token = savedToken">
            Usar o código salvo neste navegador
          </button>
        </span>
      </div>
      <DialogBox v-if="error" speaker="ERRO" variant="error" compact role="alert">{{ error }}</DialogBox>
      <div class="delr-actions">
        <button type="button" class="btn btn-ghost" :disabled="deleting" @click="toggle">Cancelar</button>
        <button type="submit" class="btn btn-danger" :disabled="!ready">
          {{ deleting ? 'Excluindo…' : 'Excluir definitivamente' }}
        </button>
      </div>
    </form>
  </div>
</template>

<style scoped>
.delr-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
  width: min(100%, 520px);
}
.delr-form .field {
  margin: 0;
}
.delr-actions {
  display: flex;
  gap: 16px;
  justify-content: flex-end;
  flex-wrap: wrap;
}
.linkish {
  background: none;
  border: 0;
  padding: 0;
  font: inherit;
  font-weight: 700;
  color: var(--accent-2);
  text-decoration: underline;
  text-decoration-thickness: var(--px);
  text-underline-offset: 4px;
  cursor: pointer;
  min-height: 44px;
  display: flex;
  align-items: center;
  text-align: left;
}
</style>
