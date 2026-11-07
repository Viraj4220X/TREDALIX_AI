import { invoke, isTauri } from '@tauri-apps/api/core';

export interface ResearchStatus {
  configured: boolean;
  provider: string;
  notice: string;
}

interface ResearchSourceResponse {
  id: string;
  title: string;
  url: string;
  snippet: string;
  excerpt_html: string | null;
  retrieved_at_utc: number;
  retrieval_status: string;
}

interface ResearchResponse {
  provider: string;
  queries: string[];
  sources: ResearchSourceResponse[];
  pages_read: number;
  pages_unavailable: number;
  retrieved_at_utc: number;
}

export interface ResearchSource {
  id: string;
  title: string;
  url: string;
  snippet: string;
  excerpt: string;
  retrievedAtUtc: number;
  retrievalStatus: string;
}

export interface ResearchResult {
  provider: string;
  queries: string[];
  sources: ResearchSource[];
  pagesRead: number;
  pagesUnavailable: number;
  retrievedAtUtc: number;
}

export async function researchStatus(): Promise<ResearchStatus> {
  if (!isTauri()) throw new Error('Web research is available only in the desktop app.');
  return invoke<ResearchStatus>('research_status');
}

export async function searchResearch(queries: string[]): Promise<ResearchResult> {
  if (!isTauri()) throw new Error('Web research is available only in the desktop app.');
  if (!queries.length || queries.length > 8) throw new Error('Enter between one and eight search queries.');
  const response = await invoke<ResearchResponse>('web_research', { queries });
  return {
    provider: response.provider,
    queries: response.queries,
    pagesRead: response.pages_read,
    pagesUnavailable: response.pages_unavailable,
    retrievedAtUtc: response.retrieved_at_utc,
    sources: response.sources.map((source) => ({
      id: source.id,
      title: source.title,
      url: source.url,
      snippet: cleanText(source.snippet, 1_000),
      excerpt: extractPageText(source.excerpt_html),
      retrievedAtUtc: source.retrieved_at_utc,
      retrievalStatus: source.retrieval_status,
    })),
  };
}

function cleanText(value: string, maximum: number): string {
  return value.replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, '').slice(0, maximum).trim();
}

function extractPageText(html: string | null): string {
  if (!html) return '';
  const document = new DOMParser().parseFromString(html, 'text/html');
  document.querySelectorAll('script,style,noscript,svg,iframe,object,form,nav,footer,header').forEach((node) => node.remove());
  return cleanText(document.body.textContent ?? '', 8_000);
}
