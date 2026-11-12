<script setup lang="ts">
import DialogBox from './DialogBox.vue'

defineProps<{ loading: boolean; error: string | null; label?: string }>()
defineEmits<{ retry: [] }>()
</script>

<template>
  <div v-if="loading" class="ls" aria-live="polite">
    <DialogBox speaker="SISTEMA" more compact class="ls-box">
      <span class="ls-row">
        <span class="spinner" aria-hidden="true"></span>
        {{ label ?? 'Carregando…' }}
      </span>
    </DialogBox>
  </div>
  <div v-else-if="error" class="ls-err">
    <DialogBox speaker="ERRO" variant="error" role="alert">
      <p>Algo deu errado: {{ error }}</p>
      <div class="dlg-actions">
        <button class="btn small" type="button" @click="$emit('retry')">Tentar novamente</button>
      </div>
    </DialogBox>
  </div>
  <slot v-else />
</template>

<style scoped>
.ls {
  display: flex;
  justify-content: center;
  padding: 40px 0;
}
.ls-box {
  min-width: min(100%, 280px);
}
.ls-row {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  padding-right: 16px;
}
.ls-err {
  padding: 16px 0;
}
.dlg-actions {
  margin-top: 12px;
}
</style>
