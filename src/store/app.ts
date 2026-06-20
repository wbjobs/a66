import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ImageRecord, OcrTextBlock, InpaintOptions, BlockWithTranslation } from '@/types'
import {
  initDatabase,
  saveImage,
  runOcr,
  getOcrBlocks,
  queryBlocksInRegion,
  translateText,
  translateBlocksInRegion,
  searchFulltext,
  getImageHistory,
  getImageById,
  getImageByHash,
  initInpainter,
  exportInpaintedImage,
  listAvailableFonts
} from '@/api/tauri'

export const useAppStore = defineStore('app', () => {
  const initialized = ref(false)
  const currentImage = ref<ImageRecord | null>(null)
  const ocrBlocks = ref<OcrTextBlock[]>([])
  const historyImages = ref<ImageRecord[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const paragraphs = computed(() => {
    const map = new Map<string, OcrTextBlock[]>()
    for (const block of ocrBlocks.value) {
      if (!map.has(block.paragraph)) {
        map.set(block.paragraph, [])
      }
      map.get(block.paragraph)!.push(block)
    }
    return Array.from(map.entries()).map(([key, blocks]) => ({
      paragraph: key,
      blocks: blocks.sort((a, b) => a.paragraph_idx - b.paragraph_idx)
    }))
  })

  async function initialize() {
    try {
      await initDatabase()
      initialized.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  async function processImage(filePath: string, doOcr: boolean = true) {
    loading.value = true
    error.value = null
    try {
      const existing = await getImageByHash(
        await computeFileHash(filePath)
      )
      if (existing) {
        currentImage.value = existing
        ocrBlocks.value = await getOcrBlocks(existing.id)
        if (ocrBlocks.value.length === 0 && doOcr) {
          ocrBlocks.value = await runOcr(existing.id)
        }
      } else {
        currentImage.value = await saveImage(filePath)
        if (doOcr) {
          ocrBlocks.value = await runOcr(currentImage.value.id)
        }
      }
      return currentImage.value
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function computeFileHash(filePath: string): Promise<string> {
    const fs = (await import('@tauri-apps/api/fs')) as typeof import('@tauri-apps/api/fs')
    const data = await fs.readBinaryFile(filePath)
    const buffer = new Uint8Array(data)
    const hashBuffer = await crypto.subtle.digest('SHA-256', buffer)
    const hashArray = Array.from(new Uint8Array(hashBuffer))
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
  }

  async function loadImage(imageId: number) {
    loading.value = true
    error.value = null
    try {
      currentImage.value = await getImageById(imageId)
      ocrBlocks.value = await getOcrBlocks(imageId)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function refreshOcr() {
    if (!currentImage.value) return
    loading.value = true
    error.value = null
    try {
      ocrBlocks.value = await runOcr(currentImage.value.id)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function queryRegion(rect: any) {
    if (!currentImage.value) return []
    return queryBlocksInRegion(currentImage.value.id, rect)
  }

  async function translateRegion(rect: any) {
    if (!currentImage.value) return []
    return translateBlocksInRegion(currentImage.value.id, rect)
  }

  async function translateSingle(text: string) {
    return translateText(text)
  }

  async function search(keyword: string, dateFrom?: string, dateTo?: string, imageHash?: string) {
    return searchFulltext(keyword, dateFrom, dateTo, imageHash)
  }

  async function loadHistory(page: number = 1, pageSize: number = 20) {
    const result = await getImageHistory(page, pageSize)
    historyImages.value = result.images
    return result
  }

  async function initPainter(fontPath?: string) {
    return initInpainter(fontPath)
  }

  async function exportInpainted(
    translations: BlockWithTranslation[],
    outputPath: string,
    options?: InpaintOptions
  ) {
    if (!currentImage.value) {
      throw new Error('请先选择一张图片')
    }
    return exportInpaintedImage(
      currentImage.value.file_path,
      translations,
      outputPath,
      options
    )
  }

  async function listFonts() {
    return listAvailableFonts()
  }

  return {
    initialized,
    currentImage,
    ocrBlocks,
    historyImages,
    loading,
    error,
    paragraphs,
    initialize,
    processImage,
    loadImage,
    refreshOcr,
    queryRegion,
    translateRegion,
    translateSingle,
    search,
    loadHistory,
    initPainter,
    exportInpainted,
    listFonts
  }
})
