export interface Rect {
  x: number
  y: number
  width: number
  height: number
}

export interface OcrTextBlock {
  id: number
  image_id: number
  text: string
  paragraph: string
  paragraph_idx: number
  x: number
  y: number
  width: number
  height: number
  confidence: number
  lang: string
  created_at: string
}

export interface TranslateResult {
  id: number
  ocr_block_id: number
  source_text: string
  translated_text: string
  source_lang: string
  target_lang: string
  created_at: string
}

export interface ImageRecord {
  id: number
  file_path: string
  file_name: string
  file_size: number
  width: number
  height: number
  hash: string
  mime_type: string
  created_at: string
  ocr_blocks?: OcrTextBlock[]
}

export interface SearchResult {
  images: ImageRecord[]
  blocks: OcrTextBlock[]
}

export interface OcrProcessOptions {
  lang?: string
  psm?: number
}

export interface TranslateOptions {
  source_lang?: string
  target_lang?: string
}
