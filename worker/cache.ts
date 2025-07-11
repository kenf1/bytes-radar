interface CacheEntry {
  data: any;
  timestamp: number;
  ttl: number;
}

export class CacheManager {
  private cache: Map<string, CacheEntry>;
  private readonly defaultTTL: number;
  private readonly maxSize: number;
  private readonly cleanupInterval: number;
  private cleanupTimer: number | null;

  constructor(
    options: {
      ttl?: number;
      maxSize?: number;
      cleanupInterval?: number;
    } = {},
  ) {
    this.cache = new Map();
    this.defaultTTL = options.ttl || 3600;
    this.maxSize = options.maxSize || 1000;
    this.cleanupInterval = options.cleanupInterval || 300;
    this.cleanupTimer = null;
    this.startCleanupTimer();
  }

  private startCleanupTimer(): void {
    if (this.cleanupTimer) return;
    this.cleanupTimer = setInterval(
      () => this.cleanup(),
      this.cleanupInterval * 1000,
    ) as unknown as number;
  }

  private stopCleanupTimer(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = null;
    }
  }

  set(key: string, value: any, ttl: number = this.defaultTTL): void {
    if (this.cache.size >= this.maxSize) {
      const oldestKey = Array.from(this.cache.entries()).sort(
        ([, a], [, b]) => a.timestamp - b.timestamp,
      )[0][0];
      this.cache.delete(oldestKey);
    }
    this.cache.set(key, {
      data: value,
      timestamp: Date.now(),
      ttl: ttl * 1000,
    });
  }

  get(key: string): any | null {
    const entry = this.cache.get(key);
    if (!entry) return null;

    const now = Date.now();
    if (now - entry.timestamp > entry.ttl) {
      this.cache.delete(key);
      return null;
    }
    return entry.data;
  }

  has(key: string): boolean {
    const entry = this.cache.get(key);
    if (!entry) return false;

    const now = Date.now();
    if (now - entry.timestamp > entry.ttl) {
      this.cache.delete(key);
      return false;
    }
    return true;
  }

  delete(key: string): void {
    this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  cleanup(): void {
    const now = Date.now();
    for (const [key, entry] of this.cache.entries()) {
      if (now - entry.timestamp > entry.ttl) {
        this.cache.delete(key);
      }
    }
  }

  getStats(): { total: number; expired: number; size: number } {
    let total = 0;
    let expired = 0;
    const now = Date.now();

    for (const entry of this.cache.values()) {
      total++;
      if (now - entry.timestamp > entry.ttl) {
        expired++;
      }
    }

    return { total, expired, size: this.cache.size };
  }

  destroy(): void {
    this.stopCleanupTimer();
    this.clear();
  }
}
