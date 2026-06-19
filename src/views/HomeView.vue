<template>
  <div class="home-view">
    <div v-if="!appStore.currentImage" class="upload-section">
      <ImageUploader @file-selected="handleFileSelected" />
    </div>
    
    <div v-else class="workspace">
      <div class="toolbar">
        <div class="toolbar-left">
          <span class="file-info">
          🖼️ {{ appStore.currentImage.file_name }}
        </span>
        <span class="meta">
          {{ appStore.currentImage.width }}×{{ appStore.currentImage.height }}
          |
          {{ formatSize(appStore.currentImage.file_size) }}
        </span>
      </div>
      <div class="toolbar-right">
        <button class="secondary" @click="clearCurrent">
          🔙 更换图片
        </button>
        <button @click="runOcr" :disabled="appStore.loading">
          {{ appStore.ocrBlocks.length ? '🔄 重新识别' : '🔍 OCR识别' }}
        </button>
      </div>
    </div>

    <div class="main-content">
      <div class="left-panel">
        <ImageCanvas
          ref="imageCanvasRef"
          :image-path="appStore.currentImage.file_path"
          :ocr-blocks="appStore.ocrBlocks"
          @region-selected="handleRegionSelected"
        />
        <div class="canvas-hint" v-if="!appStore.ocrBlocks.length">
          💡 在图片上拖拽框选区域后自动翻译，或先点击"OCR识别"查看文字块
        </div>
      </div>

      <div class="right-panel">
        <div class="panel-tabs">
          <button
            :class="{ active: activeTab === 'text' }"
            @click="activeTab = 'text'"
          >
            📝 OCR文字
          </button>
          <button
            :class="{ active: activeTab === 'translate' }"
            @click="activeTab = 'translate'"
          >
            🌐 翻译结果
          </button>
        </div>

        <div v-if="activeTab === 'text'" class="tab-content">
          <div v-if="appStore.loading" class="loading-state">
          <div class="spinner"></div>
          <span>识别中...</span>
        </div>
          <div v-else-if="!appStore.ocrBlocks.length" class="empty-state">
          <p>暂无OCR结果</p>
          <p class="hint">点击"OCR识别"开始文字识别</p>
        </div>
        <div v-else class="paragraphs">
          <div
            v-for="para in appStore.paragraphs"
            :key="para.paragraph"
            class="paragraph-item"
          >
            <div class="paragraph-header">
              <span class="paragraph-title">段落 {{ para.blocks[0].paragraph_idx + 1 }}</span>
            </div>
            <p class="paragraph-text">
              {{ para.blocks.map(b => b.text).join(' ') }}
            </p>
            <div class="block-list">
              <div
                v-for="block in para.blocks"
                :key="block.id"
                class="block-item"
                :title="`坐标: (${block.x}, ${block.y}) ${block.width}×${block.height} 置信度: ${(block.confidence.toFixed(1)}%"
              >
                <span class="block-text">{{ block.text }}</span>
                <button
                  class="translate-btn"
                  @click="translateBlock(block)"
                >
                  译
                </button>
              </div>
            </div>
          </div>
        </div>
        </div>

        <div v-if="activeTab === 'translate'" class="tab-content">
          <div v-if="translating" class="loading-state">
          <div class="spinner"></div>
          <span>翻译中...</span>
        </div>
          <div v-else-if="!translationResults.length" class="empty-state">
          <p>暂无翻译结果</p>
          <p class="hint">在图片上框选区域，或点击文字块旁的"译"按钮</p>
        </div>
        <div v-else class="translations">
          <div
            v-for="item in translationResults"
            :key="item.block.id"
            class="translation-item"
          >
            <div class="translation-source">
              <span class="label">原文:</span>
              <p>{{ item.block.text }}</p>
            </div>
            <div class="translation-arrow">⬇️</div>
            <div class="translation-target">
              <span class="label">译文:</span>
              <p class="translated">{{ item.translated_text }}</p>
            </div>
          </div>
        </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAppStore } from '@/store/app'
import ImageUploader from '@/components/ImageUploader.vue'
import ImageCanvas from '@/components/ImageCanvas.vue'
import type { OcrTextBlock } from '@/types'

const appStore = useAppStore()

const imageCanvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null)
const activeTab = ref<'text' | 'translate'>('text')
const translating = ref(false)
const translationResults = ref<
  Array<{ block: OcrTextBlock; translated_text: string }>
>([])

async function handleFileSelected(filePath: string) {
  try {
    await appStore.processImage(filePath, false)
    translationResults.value = []
  } catch (e) {
    alert('图片加载失败: ' + String(e))
  }
}

async function runOcr() {
  try {
    await appStore.refreshOcr()
  } catch (e) {
    alert('OCR识别失败: ' + String(e))
  }
}

async function handleRegionSelected(rect: any) {
  translating.value = true
  activeTab.value = 'translate'
  try {
    const results = await appStore.translateRegion(rect)
    translationResults.value = results.map((r: any) => ({
      block: r.block,
      translated_text: r.translation.translated_text
    }))
    imageCanvasRef.value?.updateCaptions(translationResults.value)
  } catch (e) {
    alert('翻译失败: ' + String(e))
  } finally {
    translating.value = false
  }
}

async function translateBlock(block: OcrTextBlock) {
  translating.value = true
  activeTab.value = 'translate'
  try {
    const result = await appStore.translateSingle(block.text)
    const item = { block, translated_text: result.translated_text }
    translationResults.value.push(item)
    imageCanvasRef.value?.updateCaptions([
      ...translationResults.value
    ])
  } catch (e) {
    alert('翻译失败: ' + String(e))
  } finally {
    translating.value = false
  }
}

function clearCurrent() {
  appStore.currentImage = null
  appStore.ocrBlocks = []
  translationResults.value = []
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1024 / 1024).toFixed(1) + ' MB'
}
</script>

<style scoped>
.home-view {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.upload-section {
  flex: 1;
  overflow: hidden;
}

.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.file-info {
  font-weight: 600;
  color: var(--text-primary);
}

.meta {
  font-size: 12px;
  color: var(--text-muted);
}

.toolbar-right {
  display: flex;
  gap: 10px;
}

.main-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.left-panel {
  flex: 1;
  position: relative;
  overflow: hidden;
  background: var(--bg-primary);
  border-right: 1px solid var(--border);
}

.canvas-hint {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--bg-tertiary);
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 12px;
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.right-panel {
  width: 420px;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
}

.panel-tabs {
  display: flex;
  border-bottom: 1px solid var(--border);
}

.panel-tabs button {
  flex: 1;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 0;
  padding: 14px 16px;
  border-bottom: 2px solid transparent;
  font-weight: 500;
}

.panel-tabs button:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  transform: none;
}

.panel-tabs button.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.tab-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 200px;
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
  height: 200px;
  color: var(--text-muted);
  text-align: center;
}

.empty-state .hint {
  font-size: 12px;
}

.paragraphs {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.paragraph-item {
  background: var(--bg-tertiary);
  border-radius: var(--radius);
  padding: 12px;
  border: 1px solid var(--border);
}

.paragraph-header {
  margin-bottom: 8px;
}

.paragraph-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
}

.paragraph-text {
  color: var(--text-primary);
  line-height: 1.7;
  margin-bottom: 12px;
  font-size: 13px;
}

.block-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.block-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 6px 10px;
  background: var(--bg-secondary);
  border-radius: 6px;
  border: 1px solid var(--border);
}

.block-text {
  flex: 1;
  font-size: 13px;
  color: var(--text-secondary);
}

.translate-btn {
  padding: 4px 10px;
  font-size: 11px;
  background: var(--bg-primary);
  color: var(--accent);
  border: 1px solid var(--border);
}

.translate-btn:hover {
  background: var(--accent);
  color: var(--bg-primary);
}

.translations {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.translation-item {
  background: var(--bg-tertiary);
  border-radius: var(--radius);
  padding: 12px;
  border: 1px solid var(--border);
}

.translation-source,
.translation-target {
  padding: 8px;
  border-radius: 6px;
}

.translation-source {
  background: var(--bg-secondary);
  margin-bottom: 4px;
}

.translation-target {
  background: rgba(158, 206, 106, 0.1);
  border: 1px solid rgba(158, 206, 106, 0.3);
  margin-top: 4px;
}

.label {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 4px;
  display: block;
}

.translation-source p {
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.6;
}

.translated {
  color: var(--success);
  font-size: 14px;
  line-height: 1.7;
  font-weight: 500;
}

.translation-arrow {
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
  padding: 2px 0;
}
</style>
