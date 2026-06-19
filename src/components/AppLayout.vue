<template>
  <div class="app-layout">
    <header class="app-header">
      <div class="header-title">
        <span class="logo-icon">📝</span>
        <span>OCR 翻译助手</span>
      </div>
      <nav class="header-nav">
        <router-link
          v-for="route in routes"
          :key="route.path"
          :to="route.path"
          class="nav-link"
        >
          {{ route.label }}
        </router-link>
      </nav>
    </header>
    <main class="app-main">
      <slot />
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import { computed } from 'vue'

const route = useRoute()

const routes = [
  { path: '/', label: 'OCR 翻译' },
  { path: '/search', label: '全文检索' },
  { path: '/history', label: '历史记录' }
]
</script>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  box-shadow: var(--shadow);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 18px;
  font-weight: 600;
  color: var(--accent);
}

.logo-icon {
  font-size: 24px;
}

.header-nav {
  display: flex;
  gap: 8px;
}

.nav-link {
  padding: 8px 16px;
  border-radius: var(--radius);
  color: var(--text-secondary);
  text-decoration: none;
  transition: all 0.2s ease;
  font-weight: 500;
}

.nav-link:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.nav-link.router-link-active {
  background: var(--accent);
  color: var(--bg-primary);
}

.app-main {
  flex: 1;
  overflow: hidden;
}
</style>
