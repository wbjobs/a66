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
        <button 
          class="export-btn" 
          @click="showExportDialog = true" 
          :disabled="!translationResults.length || exporting"
        >
          {{ exporting ? '⏳ 导出中...' : '📥 导出修复图' }}
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
                :title="formatBlockInfo(block)"
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

    <div v-if="showExportDialog" class="export-dialog-overlay" @click.self="showExportDialog = false">
      <div class="export-dialog">
        <div class="dialog-header">
          <h3>📥 导出修复后图片</h3>
          <button class="close-btn" @click="showExportDialog = false">✕</button>
        </div>
        
        <div class="dialog-body">
          <div class="form-group">
            <label>选择字体</label>
            <select v-model="exportOptions.text_style.font_path" class="form-control">
              <option value="">自动检测</option>
              <option v-for="font in availableFonts" :key="font" :value="font.split('|')[1]">
                {{ font.split('|')[0] }}
              </option>
            </select>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label>字体大小</label>
              <input 
                type="number" 
                v-model.number="exportOptions.text_style.font_size" 
                min="8" 
                max="200"
                class="form-control"
              />
            </div>
            <div class="form-group">
              <label>修复半径</label>
              <input 
                type="number" 
                v-model.number="exportOptions.inpaint_radius" 
                min="1" 
                max="20"
                class="form-control"
              />
            </div>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label>文字颜色</label>
              <input 
                type="color" 
                :value="rgbToHex(exportOptions.text_style.color)" 
                @input="exportOptions.text_style.color = hexToRgb($event.target.value)"
                class="form-control color-picker"
              />
            </div>
            <div class="form-group">
              <label>描边颜色</label>
              <div class="color-row">
                <input 
                  type="checkbox" 
                  v-model="useStroke" 
                  id="useStroke"
                />
                <label for="useStroke" class="checkbox-label">启用</label>
                <input 
                  type="color" 
                  :value="useStroke && exportOptions.text_style.stroke_color ? rgbToHex(exportOptions.text_style.stroke_color) : '#ffffff'" 
                  @input="exportOptions.text_style.stroke_color = hexToRgb($event.target.value)"
                  :disabled="!useStroke"
                  class="form-control color-picker"
                />
              </div>
            </div>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label>导出格式</label>
              <select v-model="exportOptions.export_format" class="form-control">
                <option value="png">PNG (无损)</option>
                <option value="jpeg">JPEG (有损)</option>
              </select>
            </div>
            <div class="form-group">
              <label>JPEG 质量</label>
              <input 
                type="range" 
                v-model.number="exportOptions.quality" 
                min="1" 
                max="100"
                :disabled="exportOptions.export_format !== 'jpeg'"
                class="form-control range"
              />
              <span class="range-value">{{ exportOptions.quality }}%</span>
            </div>
          </div>

          <div class="form-group">
            <label>输出路径</label>
            <div class="path-row">
              <input 
                type="text" 
                v-model="outputPath" 
                placeholder="点击选择保存位置..."
                readonly
                class="form-control path-input"
              />
              <button class="browse-btn" @click="selectOutputPath">📁</button>
            </div>
          </div>

          <div class="preview-info">
            <p>将处理 <strong>{{ translationResults.length }}</strong> 个文字块</p>
            <p class="hint">原图文字将被背景修复后替换为翻译文字</p>
          </div>
        </div>

        <div class="dialog-footer">
          <button class="secondary" @click="showExportDialog = false">取消</button>
          <button 
            class="primary" 
            @click="doExport" 
            :disabled="exporting || !outputPath"
          >
            {{ exporting ? '⏳ 处理中...' : '✅ 开始导出' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useAppStore } from '@/store/app'
import ImageUploader from '@/components/ImageUploader.vue'
import ImageCanvas from '@/components/ImageCanvas.vue'
import type { OcrTextBlock, InpaintOptions, BlockWithTranslation, TranslateResult } from '@/types'
import { defaultInpaintOptions } from '@/types'
import { save } from '@tauri-apps/api/dialog'

const appStore = useAppStore()

const imageCanvasRef = ref<InstanceType<typeof ImageCanvas> | null>(null)
const activeTab = ref<'text' | 'translate'>('text')
const translating = ref(false)
const exporting = ref(false)
const showExportDialog = ref(false)
const outputPath = ref('')
const availableFonts = ref<string[]>([])
const useStroke = ref(true)
const translationResults = ref<
  Array<{ block: OcrTextBlock; translated_text: string }>
>([])

const exportOptions = ref<InpaintOptions>(defaultInpaintOptions())

watch(useStroke, (val) => {
  if (val) {
    exportOptions.value.text_style.stroke_color = exportOptions.value.text_style.stroke_color || [255, 255, 255]
    exportOptions.value.text_style.stroke_width = exportOptions.value.text_style.stroke_width || 1
  } else {
    exportOptions.value.text_style.stroke_color = undefined
  }
})

onMounted(async () => {
  try {
    availableFonts.value = await appStore.listFonts()
  } catch (e) {
    console.warn('加载字体列表失败:', e)
  }
})

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

function formatBlockInfo(block: OcrTextBlock) {
  const img = appStore.currentImage
  if (!img) {
    return `坐标: (${(block.x * 100).toFixed(2)}%, ${(block.y * 100).toFixed(2)}%) 置信度: ${block.confidence.toFixed(1)}%`
  }
  const px = Math.round(block.x * img.width)
  const py = Math.round(block.y * img.height)
  const pw = Math.round(block.width * img.width)
  const ph = Math.round(block.height * img.height)
  return `坐标: (${px}, ${py}) ${pw}×${ph} | 归一化: (${block.x.toFixed(4)}, ${block.y.toFixed(4)}) | 置信度: ${block.confidence.toFixed(1)}%`
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

function rgbToHex(rgb: [number, number, number]): string {
  return '#' + rgb.map(c => c.toString(16).padStart(2, '0')).join('')
}

function hexToRgb(hex: string): [number, number, number] {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result
    ? [
        parseInt(result[1], 16),
        parseInt(result[2], 16),
        parseInt(result[3], 16)
      ]
    : [0, 0, 0]
}

async function selectOutputPath() {
  try {
    const ext = exportOptions.value.export_format
    const defaultName = `translated_${Date.now()}.${ext}`
    const path = await save({
      defaultPath: defaultName,
      filters: [
        {
          name: ext === 'png' ? 'PNG 图片' : 'JPEG 图片',
          extensions: [ext]
        }
      ]
    })
    if (path) {
      outputPath.value = path
    }
  } catch (e) {
    console.warn('选择路径取消:', e)
  }
}

async function doExport() {
  if (!outputPath.value || !translationResults.value.length) return

  exporting.value = true
  try {
    await appStore.initPainter(exportOptions.value.text_style.font_path || undefined)

    const translations: BlockWithTranslation[] = translationResults.value.map(item => {
      const tr: TranslateResult = {
        id: 0,
        ocr_block_id: item.block.id,
        source_text: item.block.text,
        translated_text: item.translated_text,
        source_lang: 'en',
        target_lang: 'zh',
        created_at: new Date().toISOString()
      }
      return {
        block: item.block,
        translation: tr
      }
    })

    const resultPath = await appStore.exportInpainted(
      translations,
      outputPath.value,
      exportOptions.value
    )

    alert(`✅ 导出成功！\n\n保存位置: ${resultPath}`)
    showExportDialog.value = false
  } catch (e) {
    alert('❌ 导出失败: ' + String(e))
  } finally {
    exporting.value = false
  }
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

.export-btn {
  background: linear-gradient(135deg, var(--success), var(--accent));
  color: white;
  border: none;
}

.export-btn:hover:not(:disabled) {
  filter: brightness(1.1);
  transform: translateY(-1px);
}

.export-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.export-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.export-dialog {
  background: var(--bg-secondary);
  border-radius: 12px;
  width: 500px;
  max-width: 90vw;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  border: 1px solid var(--border);
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.dialog-header h3 {
  margin: 0;
  font-size: 16px;
  color: var(--text-primary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.dialog-body {
  padding: 20px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.form-control {
  width: 100%;
  padding: 8px 12px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
  box-sizing: border-box;
}

.form-control:focus {
  outline: none;
  border-color: var(--accent);
}

.form-control:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-row .form-group {
  flex: 1;
}

.color-picker {
  height: 40px;
  padding: 2px;
  cursor: pointer;
}

.color-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.checkbox-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 0;
}

.path-row {
  display: flex;
  gap: 8px;
}

.path-input {
  flex: 1;
  cursor: pointer;
}

.browse-btn {
  padding: 8px 16px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
}

.browse-btn:hover {
  background: var(--bg-primary);
}

.range {
  flex: 1;
}

.range-value {
  display: inline-block;
  min-width: 40px;
  text-align: right;
  font-size: 12px;
  color: var(--text-muted);
}

.preview-info {
  margin-top: 20px;
  padding: 16px;
  background: var(--bg-tertiary);
  border-radius: 8px;
  border: 1px solid var(--border);
}

.preview-info p {
  margin: 0 0 8px 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.preview-info p:last-child {
  margin-bottom: 0;
}

.preview-info .hint {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
