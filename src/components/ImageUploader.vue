<template>
  <div class="image-uploader">
    <div
      class="drop-zone"
      :class="{ 'is-dragging': isDragging }"
      @dragover.prevent="handleDragOver"
      @dragleave.prevent="handleDragLeave"
      @drop.prevent="handleDrop"
      @click="handleFileSelect"
    >
      <div class="upload-icon">📁</div>
      <div class="upload-text">
        <p class="upload-title">拖入图片 或 点击选择</p>
        <p class="upload-hint">支持 JPEG、PNG 格式</p>
      </div>
      <input
        ref="fileInput"
        type="file"
        accept="image/jpeg,image/png,image/jpg"
        style="display: none"
        @change="handleFileChange"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { open } from '@tauri-apps/api/dialog'

const emit = defineEmits<{
  (e: 'file-selected', filePath: string): void
}>()

const fileInput = ref<HTMLInputElement | null>(null)
const isDragging = ref(false)

function handleDragOver() {
  isDragging.value = true
}

function handleDragLeave() {
  isDragging.value = false
}

async function handleDrop(e: DragEvent) {
  isDragging.value = false
  const files = e.dataTransfer?.files
  if (files && files.length > 0) {
    const file = files[0]
    if (validateFile(file)) {
      await processFile(file)
    }
  }
}

async function handleFileSelect() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: '图片文件',
          extensions: ['jpeg', 'jpg', 'png']
        }
      ]
    })
    if (selected && typeof selected === 'string') {
      emit('file-selected', selected)
    }
  } catch (e) {
    console.error('文件选择失败:', e)
  }
}

function handleFileChange(e: Event) {
  const target = e.target as HTMLInputElement
  const files = target.files
  if (files && files.length > 0) {
    const file = files[0]
    if (validateFile(file)) {
      processFile(file)
    }
  }
}

function validateFile(file: File): boolean {
  const validTypes = ['image/jpeg', 'image/png', 'image/jpg']
  if (!validTypes.includes(file.type)) {
    alert('请选择 JPEG 或 PNG 格式的图片')
    return false
  }
  return true
}

async function processFile(file: File) {
  try {
    const arrayBuffer = await file.arrayBuffer()
    const fs = await import('@tauri-apps/api/fs')
    const path = await import('@tauri-apps/api/path')
    
    const tempDir = await path.tempDir()
    const fileName = file.name || `image_${Date.now()}.png`
    const tempPath = await path.join(tempDir, fileName)
    
    const bytes = new Uint8Array(arrayBuffer)
    await fs.writeBinaryFile(tempPath, Array.from(bytes))
    
    emit('file-selected', tempPath)
  } catch (e) {
    console.error('文件处理失败:', e)
  }
}
</script>

<style scoped>
.image-uploader {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}

.drop-zone {
  width: 100%;
  max-width: 500px;
  height: 300px;
  border: 2px dashed var(--border);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 20px;
  cursor: pointer;
  transition: all 0.3s ease;
  background: var(--bg-secondary);
}

.drop-zone:hover,
.drop-zone.is-dragging {
  border-color: var(--accent);
  background: var(--bg-tertiary);
  transform: scale(1.02);
}

.upload-icon {
  font-size: 64px;
  opacity: 0.8;
}

.upload-text {
  text-align: center;
}

.upload-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.upload-hint {
  font-size: 13px;
  color: var(--text-muted);
}
</style>
