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
  text_tokenized: string
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

export interface BlockWithTranslation {
  block: OcrTextBlock
  translation: TranslateResult
}

export interface TextStyle {
  font_path?: string
  font_size: number
  color: [number, number, number]
  stroke_color?: [number, number, number]
  stroke_width: number
  bold: boolean
  italic: boolean
}

export interface InpaintOptions {
  inpaint_radius: number
  text_style: TextStyle
  export_format: string
  quality: number
}

export function defaultTextStyle(): TextStyle {
  return {
    font_path: undefined,
    font_size: 24,
    color: [0, 0, 0],
    stroke_color: [255, 255, 255],
    stroke_width: 1,
    bold: false,
    italic: false
  }
}

export function defaultInpaintOptions(): InpaintOptions {
  return {
    inpaint_radius: 3,
    text_style: defaultTextStyle(),
    export_format: 'png',
    quality: 95
  }
}
