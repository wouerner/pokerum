import { ref } from 'vue'

/**
 * Consentimento do Google Analytics (LGPD, Consent Mode). A tag só existe quando o servidor
 * tem POKERUM_GA_ID (backend/api/src/analytics.rs), que também define `window.pokerumAnalytics`
 * e o consentimento padrão negado. Aqui só guardamos a escolha e avisamos o gtag.
 */
type Choice = 'granted' | 'denied'

const KEY = 'pokerum-ga-consent' // igual a analytics::CONSENT_KEY no backend

declare global {
  interface Window {
    pokerumAnalytics?: { id: string }
    gtag?: (...args: unknown[]) => void
  }
}

export const analyticsEnabled = typeof window !== 'undefined' && !!window.pokerumAnalytics

function stored(): Choice | null {
  try {
    const v = localStorage.getItem(KEY)
    return v === 'granted' || v === 'denied' ? v : null
  } catch {
    return null
  }
}

/** Mostra o aviso: só com a tag presente e sem escolha salva. */
export const consentOpen = ref(analyticsEnabled && stored() === null)

export function choose(v: Choice) {
  try {
    localStorage.setItem(KEY, v)
  } catch {
    /* navegação privada: vale só nesta visita */
  }
  window.gtag?.('consent', 'update', { analytics_storage: v })
  consentOpen.value = false
}

export function reopenConsent() {
  consentOpen.value = true
}
