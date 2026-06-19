<template>
  <div class="image-canvas-wrapper" ref="wrapperRef">
    <div class="canvas-container" ref="containerRef">
      <canvas
        ref="imageCanvas"
        class="image-canvas"
        @mousedown="handleMouseDown"
        @mousemove="handleMouseMove"
        @mouseup="handleMouseUp"
        @mouseleave="handleMouseUp"
      />
      <canvas
        ref="overlayCanvas"
        class="overlay-canvas"
      />
      <div
        v-for="(caption, idx) in overlayCaptions"
        :key="idx"
        class="subtitle-caption"
        :style="getCaptionStyle(caption)"
      >
        {{ caption.text }}
      </div>
      <div
        v-if="selection && isSelecting"
        class="selection-box"
        :style="selectionStyle"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed, nextTick } from 'vue'
import type { OcrTextBlock } from '@/types'

interface CaptionItem {
  x: number
  y: number
  width: number
  height: number
  text: string
}

const props = defineProps<{
  imagePath: string
  ocrBlocks: OcrTextBlock[]
}>()

const emit = defineEmits<{
  (e: 'region-selected', rect: { x: number; y: number; width: number; height: number }): void
}>()

const wrapperRef = ref<HTMLDivElement | null>(null)
const containerRef = ref<HTMLDivElement | null>(null)
const imageCanvas = ref<HTMLCanvasElement | null>(null)
const overlayCanvas = ref<HTMLCanvasElement | null>(null)

const imageLoaded = ref(false)
const naturalWidth = ref(0)
const naturalHeight = ref(0)
const scale = ref(1)
const offsetX = ref(0)
const offsetY = ref(0)

const isSelecting = ref(false)
const startPos = ref({ x: 0, y: 0 })
const currentPos = ref({ x: 0, y: 0 })
const overlayCaptions = ref<CaptionItem[]>([])

const selection = computed(() => {
  if (!isSelecting.value) return null
  const x = Math.min(startPos.value.x, currentPos.value.x)
  const y = Math.min(startPos.value.y, currentPos.value.y)
  const width = Math.abs(currentPos.value.x - startPos.value.x)
  const height = Math.abs(currentPos.value.y - startPos.value.y)
  return { x, y, width, height }
})

const selectionStyle = computed(() => {
  if (!selection.value) return {}
  return {
    left: `${selection.value.x}px`,
    top: `${selection.value.y}px`,
    width: `${selection.value.width}px`,
    height: `${selection.value.height}px`
  }
})

function getCaptionStyle(caption: CaptionItem) {
  return {
    left: `${caption.x}px`,
    top: `${caption.y}px`,
    width: `${caption.width}px`,
    fontSize: `${Math.max(12, caption.height * 0.6)}px`
  }
}

async function loadImage() {
  if (!props.imagePath || !imageCanvas.value) return
  
  const fs = await import('@tauri-apps/api/fs')
  const bytes = await fs.readBinaryFile(props.imagePath)
  const uint8 = new Uint8Array(bytes)
  const blob = new Blob([uint8])
  const url = URL.createObjectURL(blob)
  
  const img = new Image()
  img.onload = () => {
    naturalWidth.value = img.width
    naturalHeight.value = img.height
    resizeCanvas()
    drawImage(img)
    drawOcrBlocks()
    imageLoaded.value = true
    URL.revokeObjectURL(url)
  }
  img.src = url
}

function resizeCanvas() {
  if (!containerRef.value || !imageCanvas.value || !overlayCanvas.value) return
  
  const containerWidth = containerRef.value.clientWidth
  const containerHeight = containerRef.value.clientHeight
  
  const scaleX = containerWidth / naturalWidth.value
  const scaleY = containerHeight / naturalHeight.value
  scale.value = Math.min(scaleX, scaleY, 1)
  
  const displayWidth = naturalWidth.value * scale.value
  const displayHeight = naturalHeight.value * scale.value
  
  offsetX.value = (containerWidth - displayWidth) / 2
  offsetY.value = (containerHeight - displayHeight) / 2
  
  const dpr = window.devicePixelRatio || 1
  for (const canvas of [imageCanvas.value, overlayCanvas.value]) {
    canvas.width = containerWidth * dpr
    canvas.height = containerHeight * dpr
    canvas.style.width = `${containerWidth}px`
    canvas.style.height = `${containerHeight}px`
    const ctx = canvas.getContext('2d')!
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  }
}

function drawImage(img: HTMLImageElement) {
  if (!imageCanvas.value) return
  const ctx = imageCanvas.value.getContext('2d')!
  ctx.clearRect(0, 0, imageCanvas.value.width, imageCanvas.value.height)
  const displayWidth = naturalWidth.value * scale.value
  const displayHeight = naturalHeight.value * scale.value
  ctx.drawImage(img, offsetX.value, offsetY.value, displayWidth, displayHeight)
}

function drawOcrBlocks() {
  if (!overlayCanvas.value || !props.ocrBlocks.length) return
  const ctx = overlayCanvas.value.getContext('2d')!
  ctx.clearRect(0, 0, overlayCanvas.value.width, overlayCanvas.value.height)
  
  ctx.strokeStyle = 'rgba(122, 162, 247, 0.6)'
  ctx.lineWidth = 1
  ctx.fillStyle = 'rgba(122, 162, 247, 0.08)'
  
  for (const block of props.ocrBlocks) {
    const x = offsetX.value + block.x * scale.value
    const y = offsetY.value + block.y * scale.value
    const w = block.width * scale.value
    const h = block.height * scale.value
    
    ctx.fillRect(x, y, w, h)
    ctx.strokeRect(x, y, w, h)
  }
}

function screenToImageCoords(screenX: number, screenY: number) {
  const rect = overlayCanvas.value!.getBoundingClientRect()
  const x = (screenX - rect.left - offsetX.value) / scale.value
  const y = (screenY - rect.top - offsetY.value) / scale.value
  return { x: Math.round(x), y: Math.round(y) }
}

function handleMouseDown(e: MouseEvent) {
  if (!imageLoaded.value) return
  isSelecting.value = true
  startPos.value = { x: e.clientX, y: e.clientY }
  currentPos.value = { ...startPos.value }
  overlayCaptions.value = []
}

function handleMouseMove(e: MouseEvent) {
  if (!isSelecting.value) return
  currentPos.value = { x: e.clientX, y: e.clientY }
}

function handleMouseUp() {
  if (!isSelecting.value || !selection.value) {
    isSelecting.value = false
    return
  }
  
  const rect = overlayCanvas.value!.getBoundingClientRect()
  const selStart = {
    x: Math.min(startPos.value.x, currentPos.value.x) - rect.left,
    y: Math.min(startPos.value.y, currentPos.value.y) - rect.top
  }
  const selEnd = {
    x: Math.max(startPos.value.x, currentPos.value.x) - rect.left,
    y: Math.max(startPos.value.y, currentPos.value.y) - rect.top
  }
  
  const imgSel = {
    x: Math.max(0, Math.round((selStart.x - offsetX.value) / scale.value)),
    y: Math.max(0, Math.round((selStart.y - offsetY.value) / scale.value)),
    width: Math.min(
      naturalWidth.value,
      Math.round((selEnd.x - selStart.x) / scale.value)
    ),
    height: Math.min(
      naturalHeight.value,
      Math.round((selEnd.y - selStart.y) / scale.value)
    )
  }
  
  if (imgSel.width > 5 && imgSel.height > 5) {
    emit('region-selected', imgSel)
  }
  
  isSelecting.value = false
}

function updateCaptions(
  translations: Array<{ block: OcrTextBlock; translated_text: string }>
) {
  overlayCaptions.value = translations.map(({ block, translated_text }) => ({
    x: offsetX.value + block.x * scale.value,
    y: offsetY.value + block.y * scale.value,
    width: block.width * scale.value,
    height: block.height * scale.value,
    text: translated_text
  }))
}

function handleResize() {
  if (imageLoaded.value) {
    loadImage()
  }
}

defineExpose({
  updateCaptions
})

watch(() => props.imagePath, loadImage)
watch(() => props.ocrBlocks, drawOcrBlocks, { deep: true })

onMounted(() => {
  window.addEventListener('resize', handleResize)
  nextTick(loadImage)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
})
</script>

<style scoped>
.image-canvas-wrapper {
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: var(--bg-primary);
}

.canvas-container {
  position: relative;
  width: 100%;
  height: 100%;
}

.image-canvas,
.overlay-canvas {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
}

.overlay-canvas {
  pointer-events: auto;
  cursor: crosshair;
  z-index: 2;
}

.selection-box {
  position: absolute;
  border: 2px solid var(--accent);
  background: rgba(122, 162, 247, 0.15);
  pointer-events: none;
  z-index: 3;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.3);
}

.subtitle-caption {
  position: absolute;
  background: rgba(0, 0, 0, 0.8);
  color: var(--text-primary);
  padding: 2px 6px;
  border-radius: 3px;
  font-weight: 500;
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  z-index: 4;
  pointer-events: none;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}
</style>
