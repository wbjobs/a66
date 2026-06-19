<template>
  <div class="history-view">
    <div class="view-header">
      <h2>📚 历史记录</h2>
      <div class="header-actions">
        <span class="total-count">共 {{ total }} 张图片</span>
        <button class="secondary" @click="refresh">🔄 刷新</button>
      </div>
    </div>

    <div class="history-content">
      <div v-if="loading" class="loading-state">
        <div class="spinner"></div>
        <span>加载中...</span>
      </div>

      <div v-else-if="!appStore.historyImages.length" class="empty-state">
        <p>暂无历史记录</p>
        <p class="hint">处理的图片会自动保存到这里</p>
      </div>

      <div v-else class="history-list">
        <div
          v-for="img in appStore.historyImages"
          :key="img.id"
          class="history-item"
          @click="openImage(img.id)"
        >
          <div class="thumb">🖼️</div>
          <div class="item-info">
            <div class="item-title">{{ img.file_name }}</div>
            <div class="item-meta">
              <span>{{ img.width }}×{{ img.height }}</span>
              <span>·</span>
              <span>{{ formatSize(img.file_size) }}</span>
              <span>·</span>
              <span>{{ formatDate(img.created_at) }}</span>
            </div>
            <div class="item-hash" :title="img.hash">
              SHA256: {{ img.hash.slice(0, 32) }}...
            </div>
          </div>
          <div class="item-actions" @click.stop>
            <button @click="openImage(img.id)">打开</button>
          </div>
        </div>
      </div>

      <div v-if="total > pageSize" class="pagination">
        <button
          class="secondary"
          :disabled="page === 1"
          @click="prevPage"
        >
          上一页
        </button>
        <span class="page-info">
          第 {{ page }} / {{ totalPages }} 页
        </span>
        <button
          class="secondary"
          :disabled="page >= totalPages"
          @click="nextPage"
        >
          下一页
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/store/app'

const router = useRouter()
const appStore = useAppStore()

const loading = ref(false)
const total = ref(0)
const page = ref(1)
const pageSize = 20

const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)))

async function loadData() {
  loading.value = true
  try {
    const result = await appStore.loadHistory(page.value, pageSize)
    total.value = result.total
  } catch (e) {
    console.error('加载历史记录失败:', e)
  } finally {
    loading.value = false
  }
}

function refresh() {
  page.value = 1
  loadData()
}

function prevPage() {
  if (page.value > 1) {
    page.value--
    loadData()
  }
}

function nextPage() {
  if (page.value < totalPages.value) {
    page.value++
    loadData()
  }
}

async function openImage(id: number) {
  await appStore.loadImage(id)
  router.push('/')
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1024 / 1024).toFixed(1) + ' MB'
}

function formatDate(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

onMounted(loadData)
</script>

<style scoped>
.history-view {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.view-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.total-count {
  font-size: 13px;
  color: var(--text-muted);
}

.history-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 80px 0;
  color: var(--text-secondary);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 80px 0;
  color: var(--text-muted);
}

.empty-state .hint {
  font-size: 13px;
}

.history-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  cursor: pointer;
  transition: all 0.2s ease;
}

.history-item:hover {
  border-color: var(--accent);
  background: var(--bg-tertiary);
  transform: translateX(4px);
}

.thumb {
  width: 64px;
  height: 64px;
  background: var(--bg-tertiary);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  flex-shrink: 0;
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.item-hash {
  font-size: 11px;
  color: var(--text-muted);
  font-family: monospace;
  opacity: 0.6;
}

.item-actions {
  flex-shrink: 0;
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 20px 0;
  border-top: 1px solid var(--border);
  margin-top: 16px;
}

.page-info {
  font-size: 13px;
  color: var(--text-muted);
}
</style>
