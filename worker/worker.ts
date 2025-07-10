import type { AnalysisOptions } from "./types";
import wasmBinary from "./pkg/bytes_radar_bg.wasm";

export interface Env {
  BYTES_RADAR: DurableObjectNamespace;
  LOG_LEVEL?: string;
  ENVIRONMENT?: string;
}

export class BytesRadar {
  state: DurableObjectState;
  env: Env;
  private wasmModule: any = null;
  private wasmInitialized = false;

  constructor(state: DurableObjectState, env: Env) {
    this.state = state;
    this.env = env;
  }

  private log(
    level: "debug" | "info" | "warn" | "error",
    message: string,
    data?: any,
  ) {
    const logLevel = this.env.LOG_LEVEL || "info";
    const environment = this.env.ENVIRONMENT || "development";

    const levels = { debug: 0, info: 1, warn: 2, error: 3 };
    const currentLevel = levels[logLevel as keyof typeof levels] || 1;
    const messageLevel = levels[level];

    if (messageLevel >= currentLevel) {
      const timestamp = new Date().toISOString();
      const logEntry = {
        timestamp,
        level: level.toUpperCase(),
        environment,
        message,
        ...(data && { data }),
      };

      const fn = {
        debug: console.debug,
        info: console.info,
        warn: console.warn,
        error: console.error,
      };

      if (environment === "production") {
        fn[level](JSON.stringify(logEntry));
      } else {
        fn[level](`[${level.toUpperCase()}] ${message}`, data ? data : "");
      }
    }
  }

  private async initializeWasm() {
    if (!this.wasmInitialized) {
      try {
        this.wasmModule = await import("./pkg/bytes_radar");
        await this.wasmModule.default(wasmBinary);
        this.wasmInitialized = true;
        this.log("info", "WebAssembly module initialized successfully");
      } catch (error) {
        this.log("error", "Failed to initialize WebAssembly module", {
          error: error instanceof Error ? error.message : String(error),
        });
        throw error;
      }
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

    // Parse numeric options
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

    // Parse string options
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

    // Parse boolean options
    if (searchParams.get("aggressive_filtering") !== null) {
      options.aggressive_filtering =
        searchParams.get("aggressive_filtering") === "true";
    }

    // Parse custom headers, credentials, and provider settings
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
            error: "Missing repository path",
            usage: [
              "/[user/repo",
              "/user/repo@master",
              "/github.com/user/repo",
              "/gitlab.com/user/repo",
              "http://example.com/example-asset.tar.gz",
            ],
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
      this.log("info", "Starting analysis", { url: targetUrl, options });

      const result = await this.wasmModule.analyze_url(targetUrl, options);
      this.log("info", "Analysis completed successfully", result);

      return new Response(JSON.stringify(result), {
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
        },
      });
    } catch (error: unknown) {
      const errorResponse = this.handleError(error, startTime);
      this.log("error", "Error in BytesRadar fetch", errorResponse);

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
