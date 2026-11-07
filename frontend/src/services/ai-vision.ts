import { invoke, isTauri } from '@tauri-apps/api/core';

export interface VisionWord {
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface VisionAnalysis {
  model: string;
  observed_at_utc: number;
  visual_observation: string;
  ocr_text: string;
  ocr_words: VisionWord[];
  caveat: string;
}

export interface ScreenTextRecognition {
  text: string;
  words: VisionWord[];
}

export async function recognizeScreenText(image: Uint8Array): Promise<ScreenTextRecognition> {
  if (!isTauri()) throw new Error('Local Windows OCR is available only in the desktop app.');
  if (image.byteLength > 2 * 1024 * 1024) throw new Error('The resized PNG must be no larger than 2 MiB.');
  return invoke<ScreenTextRecognition>('recognize_screen_text', {
    image: Array.from(image),
  });
}

export async function resizeImageToPng(file: Blob, maximumBytes = 2 * 1024 * 1024): Promise<Uint8Array> {
  const bitmap = await createImageBitmap(file);
  try {
    const scale = Math.min(1, 1_600 / bitmap.width, 1_600 / bitmap.height, Math.sqrt(2_560_000 / (bitmap.width * bitmap.height)));
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context) throw new Error('The browser could not prepare the image for local analysis.');
    let width = Math.max(1, Math.round(bitmap.width * scale));
    let height = Math.max(1, Math.round(bitmap.height * scale));
    for (let attempt = 0; attempt < 7; attempt += 1) {
      canvas.width = width;
      canvas.height = height;
      context.drawImage(bitmap, 0, 0, width, height);
      const png = await new Promise<Blob>((resolve, reject) => {
        canvas.toBlob((result) => result ? resolve(result) : reject(new Error('The image could not be encoded as PNG.')), 'image/png');
      });
      if (png.size <= maximumBytes) return new Uint8Array(await png.arrayBuffer());
      width = Math.max(1, Math.floor(width * 0.75));
      height = Math.max(1, Math.floor(height * 0.75));
    }
    throw new Error('The image could not be reduced below the 2 MiB local analysis limit.');
  } finally {
    bitmap.close();
  }
}
