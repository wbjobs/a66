import { invoke } from '@tauri-apps/api/tauri'
import type {
  ImageRecord,
  OcrTextBlock,
  TranslateResult,
  Rect,
  OcrProcessOptions,
  TranslateOptions,
  SearchResult
} from '@/types'

export async function initDatabase(): Promise<void> {
  return invoke('init_database')
}

export async function saveImage(filePath: string): Promise<ImageRecord> {
  return invoke('save_image', { filePath })
}

export async function runOcr(
  imageId: number,
  options?: OcrProcessOptions
): Promise<OcrTextBlock[]> {
  return invoke('run_ocr', { imageId, options })
}

export async function getOcrBlocks(imageId: number): Promise<OcrTextBlock[]> {
  return invoke('get_ocr_blocks', { imageId })
}

export async function queryBlocksInRegion(
  imageId: number,
  rect: Rect,
  threshold: number = 0.5
): Promise<OcrTextBlock[]> {
  return invoke('query_blocks_in_region', { imageId, rect, threshold })
}

export async function translateText(
  sourceText: string,
  options?: TranslateOptions
): Promise<TranslateResult> {
  return invoke('translate_text', { sourceText, options })
}

export async function translateBlocksInRegion(
  imageId: number,
  rect: Rect,
  options?: TranslateOptions
): Promise<Array<{ block: OcrTextBlock; translation: TranslateResult }>> {
  return invoke('translate_blocks_in_region', { imageId, rect, options })
}

export async function saveTranslation(
  ocrBlockId: number,
  sourceText: string,
  translatedText: string,
  sourceLang: string,
  targetLang: string
): Promise<TranslateResult> {
  return invoke('save_translation', {
    ocrBlockId,
    sourceText,
    translatedText,
    sourceLang,
    targetLang
  })
}

export async function searchFulltext(
  keyword: string,
  dateFrom?: string,
  dateTo?: string,
  imageHash?: string
): Promise<SearchResult> {
  return invoke('search_fulltext', { keyword, dateFrom, dateTo, imageHash })
}

export async function getImageHistory(
  page: number = 1,
  pageSize: number = 20
): Promise<{ images: ImageRecord[]; total: number }> {
  return invoke('get_image_history', { page, pageSize })
}

export async function getImageById(imageId: number): Promise<ImageRecord> {
  return invoke('get_image_by_id', { imageId })
}

export async function getImageByHash(hash: string): Promise<ImageRecord | null> {
  return invoke('get_image_by_hash', { hash })
}

export async function getTranslations(ocrBlockId: number): Promise<TranslateResult[]> {
  return invoke('get_translations', { ocrBlockId })
}
