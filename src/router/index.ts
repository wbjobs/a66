import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'
import HomeView from '@/views/HomeView.vue'
import SearchView from '@/views/SearchView.vue'
import HistoryView from '@/views/HistoryView.vue'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: HomeView,
    meta: { title: 'OCR 翻译' }
  },
  {
    path: '/search',
    name: 'search',
    component: SearchView,
    meta: { title: '全文检索' }
  },
  {
    path: '/history',
    name: 'history',
    component: HistoryView,
    meta: { title: '历史记录' }
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
