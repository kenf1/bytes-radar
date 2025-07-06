import type { Analysis, ComparisonResult } from './types';

declare const init: () => Promise<void>;
declare const analyze_repository: (url: string) => Promise<Analysis>;
declare const compare_repositories: (urls: string[]) => Promise<ComparisonResult>;

let initialized = false;

export async function initWasm(): Promise<void> {
  if (!initialized) {
    await init();
    initialized = true;
  }
}

export async function analyzeRepository(url: string): Promise<Analysis> {
  if (!initialized) {
    await initWasm();
  }
  return analyze_repository(url);
}

export async function compareRepositories(urls: string[]): Promise<ComparisonResult> {
  if (!initialized) {
    await initWasm();
  }
  return compare_repositories(urls);
}

export type { Analysis, ComparisonResult } from './types'; 