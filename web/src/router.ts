import { createRouter, createWebHistory } from 'vue-router'

export const router = createRouter({
  history: createWebHistory(),
  scrollBehavior: () => ({ top: 0 }),
  routes: [
    { path: '/', name: 'jobs', component: () => import('./views/JobsView.vue') },
    { path: '/vagas/nova', name: 'new-job', component: () => import('./views/NewJobView.vue') },
    { path: '/vagas/:slug', name: 'job', component: () => import('./views/JobView.vue'), props: true },
    { path: '/vagas/:slug/aplicar', name: 'apply', component: () => import('./views/ApplyView.vue'), props: true },
    {
      path: '/vagas/:slug/candidatos/:codename',
      name: 'result',
      component: () => import('./views/ResultView.vue'),
      props: true,
    },
    { path: '/curriculos/:codename', name: 'resume', component: () => import('./views/ResumeView.vue'), props: true },
    { path: '/:pathMatch(.*)*', name: 'not-found', component: () => import('./views/NotFoundView.vue') },
  ],
})
