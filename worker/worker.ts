import type { AnalysisOptions } from "./types";
import wasmBinary from "./pkg/bytes_radar_bg.wasm";
import { CacheManager } from "./cache";

export interface Env {
  BYTES_RADAR: DurableObjectNamespace;
  LOG_LEVEL?: string;
  ENVIRONMENT?: string;
}

export class BytesRadar {
  private wasmModule: any;
  private wasmInitialized: boolean = false;
  private cacheManager: CacheManager;

  constructor() {
    this.cacheManager = new CacheManager({
      ttl: 7200,
      maxSize: 5000,
      cleanupInterval: 600,
    });
  }

  private log(level: string, message: string, data?: any) {
    console.log(JSON.stringify({ level, message, data }));
  }

  private async initializeWasm() {
    if (this.wasmInitialized) return;
    try {
      this.wasmModule = await import("./pkg/bytes_radar");
      await this.wasmModule.default(wasmBinary);
      this.wasmInitialized = true;
      this.log("info", "WASM initialized");
    } catch (error) {
      this.log("error", "WASM init failed", error);
      throw error;
    }
  }

  private parseQueryOptions(searchParams: URLSearchParams): AnalysisOptions {
    const options: AnalysisOptions = {
      ignore_hidden: searchParams.get("ignore_hidden") !== "false",
      ignore_gitignore: searchParams.get("ignore_gitignore") !== "false",
      accept_invalid_certs: searchParams.get("accept_invalid_certs") === "true",
      use_compression: searchParams.get("use_compression") !== "false",
      headers: {},
      credentials: {},
      provider_settings: {},
    };

    const numericOptions: Record<string, (val: number) => void> = {
      timeout: (val) => (options.timeout = val),
      max_redirects: (val) => (options.max_redirects = val),
      max_file_size: (val) => (options.max_file_size = val),
    };

    for (const [key, setter] of Object.entries(numericOptions)) {
      const value = searchParams.get(key);
      if (value !== null) {
        const numValue = parseInt(value, 10);
        if (!isNaN(numValue)) {
          setter(numValue);
        }
      }
    }

    const stringOptions: Record<string, (val: string) => void> = {
      user_agent: (val) => (options.user_agent = val),
      proxy: (val) => (options.proxy = val),
    };

    for (const [key, setter] of Object.entries(stringOptions)) {
      const value = searchParams.get(key);
      if (value !== null) {
        setter(value);
      }
    }

    if (searchParams.get("aggressive_filtering") !== null) {
      options.aggressive_filtering =
        searchParams.get("aggressive_filtering") === "true";
    }

    for (const [key, value] of searchParams.entries()) {
      if (key.startsWith("header.")) {
        options.headers[key.slice(7)] = value;
      } else if (key.startsWith("credential.")) {
        options.credentials[key.slice(11)] = value;
      } else if (key.startsWith("provider.")) {
        options.provider_settings[key.slice(9)] = value;
      }
    }

    return options;
  }

  private generateCacheKey(url: string, options: AnalysisOptions): string {
    const { headers, credentials, ...cacheableOptions } = options;
    return `${url}:${JSON.stringify(cacheableOptions)}`;
  }

  async fetch(request: Request) {
    const url = new URL(request.url);
    if (url.pathname === "/favicon.ico") {
      return new Response(null, { status: 404 });
    }

    const startTime = performance.now();

    try {
      await this.initializeWasm();

      const pathParts = url.pathname.split("/").filter(Boolean);
      const targetUrl = pathParts.join("/");

      if (!targetUrl) {
        return new Response(
          JSON.stringify({
            error: "Missing path",
            usage: ["/user/repo", "/user/repo@branch", "/github.com/user/repo"],
          }),
          {
            status: 400,
            headers: {
              "Content-Type": "application/json",
              "Access-Control-Allow-Origin": "*",
            },
          },
        );
      }

      const options = this.parseQueryOptions(url.searchParams);
      const cacheKey = this.generateCacheKey(targetUrl, options);

      const cachedResult = this.cacheManager.get(cacheKey);
      if (cachedResult) {
        this.log("info", "Cache hit", { url: targetUrl });
        return new Response(JSON.stringify(cachedResult), {
          headers: {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*",
            "X-Cache": "HIT",
            "X-Cache-TTL": String(this.cacheManager["defaultTTL"]),
          },
        });
      }

      this.log("info", "Analysis start", { url: targetUrl });

      const result = await this.wasmModule.analyze_url(targetUrl, options);
      this.cacheManager.set(cacheKey, result);

      this.log("info", "Analysis done", result);

      return new Response(JSON.stringify(result), {
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
          "X-Cache": "MISS",
        },
      });
    } catch (error: unknown) {
      const errorResponse = this.handleError(error, startTime);
      this.log("error", "Request failed", errorResponse);

      return new Response(JSON.stringify(errorResponse), {
        status: 500,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
        },
      });
    }
  }

  private handleError(error: unknown, startTime: number) {
    let errorMessage = error instanceof Error ? error.message : String(error);
    let errorType = "UnknownError";
    let url = "";

    if (error && typeof error === "object" && "error" in error) {
      const wasmError = error as any;
      errorMessage = wasmError.error || errorMessage;
      errorType = wasmError.error_type || "WASMError";
      url = wasmError.url || "";
    }

    return {
      error: errorMessage,
      error_type: errorType,
      url,
    };
  }
}

export default {
  async fetch(
    request: Request,
    env: Env,
    ctx: ExecutionContext,
  ): Promise<Response> {
    const id = env.BYTES_RADAR.idFromName("default");
    const obj = env.BYTES_RADAR.get(id);
    return obj.fetch(request);
  },
};
