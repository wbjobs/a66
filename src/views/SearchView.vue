<template>
  <div class="search-view">
    <div class="search-header">
      <div class="search-box">
        <input
          v-model="keyword"
          type="text"
          placeholder="🔍 输入关键词进行全文检索..."
          @keyup.enter="doSearch"
        />
        <button @click="doSearch" :disabled="!keyword.trim()">
          搜索
        </button>
      </div>
      <div class="filter-row">
        <div class="filter-item">
          <label>日期范围:</label>
          <input v-model="dateFrom" type="date" />
          <span>至</span>
          <input v-model="dateTo" type="date" />
        </div>
        <div class="filter-item">
          <label>图片Hash:</label>
          <input
            v-model="imageHash"
            type="text"
            placeholder="可选，精确匹配SHA256"
            style="width: 300px"
          />
        </div>
      </div>
    </div>

    <div class="search-results">
      <div v-if="searching" class="loading-state">
        <div class="spinner"></div>
        <span>搜索中...</span>
      </div>

      <div
        v-else-if="
          !searchPerformed ||
          (!results.images.length && !results.blocks.length)
        "
        class="empty-state"
      >
        <p>{{ searchPerformed ? '未找到匹配结果' : '输入关键词开始搜索' }}</p>
        <p class="hint">支持按文字内容、日期范围、图片Hash检索</p>
      </div>

      <template v-else>
        <section v-if="results.images.length" class="result-section">
          <h3>
            匹配的图片 ({{ results.images.length }})
          </h3>
          <div class="image-grid">
            <div
              v-for="img in results.images"
              :key="img.id"
              class="image-card"
              @click="openImage(img.id)"
            >
              <div class="thumb">
                🖼️
              </div>
              <div class="card-info">
                <div class="file-name">{{ img.file_name }}</div>
                <div class="card-meta">
                  {{ img.width }}×{{ img.height }} |
                  {{ formatDate(img.created_at) }}
                </div>
                <div class="hash" :title="img.hash">
                  {{ img.hash.slice(0, 16) }}...
                </div>
              </div>
            </div>
          </div>
        </section>

        <section v-if="results.blocks.length" class="result-section">
          <h3>
            匹配的文字块 ({{ results.blocks.length }})
          </h3>
          <div class="block-list">
            <div
              v-for="block in results.blocks"
              :key="block.id"
              class="block-card"
            >
              <div class="block-text">
                {{ highlightMatch(block.text) }}
              </div>
              <div class="block-meta">
                <span>坐标: ({{ block.x }}, {{ block.y }}) {{ block.width }}×{{ block.height }}</span>
                <span>置信度: {{ block.confidence.toFixed(1) }}%</span>
                <span>{{ formatDate(block.created_at) }}</span>
              </div>
            </div>
          </div>
        </section>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, markRaw } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/store/app'
import type { SearchResult, ImageRecord } from '@/types'

const router = useRouter()
const appStore = useAppStore()

const keyword = ref('')
const dateFrom = ref('')
const dateTo = ref('')
const imageHash = ref('')
const searching = ref(false)
const searchPerformed = ref(false)

const results = reactive<SearchResult>({
  images: [],
  blocks: []
})

async function doSearch() {
  if (!keyword.value.trim() && !dateFrom.value && !dateTo.value && !imageHash.value) {
    return
  }
  searching.value = true
  try {
    const result = await appStore.search(
      keyword.value.trim(),
      dateFrom.value || undefined,
      dateTo.value || undefined,
      imageHash.value.trim() || undefined
    )
    results.images = result.images
    results.blocks = result.blocks
    searchPerformed.value = true
  } catch (e) {
    alert('搜索失败: ' + String(e))
  } finally {
    searching.value = false
  }
}

function highlightMatch(text: string): string {
  if (!keyword.value.trim()) return text
  const kw = keyword.value.trim()
  const regex = new RegExp(`(${escapeRegex(kw)})`, 'gi')
  return text.replace(regex, '<mark>$1</mark>')
}

function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function openImage(id: number) {
  appStore.loadImage(id).then(() => {
    router.push('/')
  })
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
</script>

<style scoped>
.search-view {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.search-header {
  padding: 20px 24px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.search-box {
  display: flex;
  gap: 10px;
}

.search-box input {
  flex: 1;
  font-size: 15px;
  padding: 12px 16px;
}

.filter-row {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}

.filter-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-item label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.filter-item input {
  padding: 6px 10px;
  font-size: 13px;
}

.search-results {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
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

.result-section {
  margin-bottom: 28px;
}

.result-section h3 {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 12px;
  font-weight: 600;
}

.image-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.image-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s ease;
}

.image-card:hover {
  border-color: var(--accent);
  transform: translateY(-2px);
  box-shadow: var(--shadow);
}

.thumb {
  height: 120px;
  background: var(--bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 40px;
}

.card-info {
  padding: 10px 12px;
}

.file-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 4px;
}

.card-meta {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.hash {
  font-size: 10px;
  color: var(--text-muted);
  font-family: monospace;
  opacity: 0.7;
}

.block-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.block-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px 16px;
}

.block-text {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.7;
  margin-bottom: 8px;
}

.block-text :deep(mark) {
  background: var(--warning);
  color: var(--bg-primary);
  padding: 1px 4px;
  border-radius: 3px;
}

.block-meta {
  display: flex;
  gap: 16px;
  font-size: 11px;
  color: var(--text-muted);
}
</style>
