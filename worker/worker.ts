import type { AnalyzeOptions } from "./pkg/bytes_radar";
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

  async fetch(request: Request) {
    const url = new URL(request.url);
    if (url.pathname === "/favicon.ico") {
      return new Response(null, { status: 404 });
    }

    const startTime = performance.now();
    const debugInfo: any = {
      timestamp: new Date().toISOString(),
      wasm_initialized: this.wasmInitialized,
    };

    try {
      await this.initializeWasm();
      debugInfo.wasm_initialized = true;

      const pathParts = url.pathname.split("/").filter(Boolean);
      const targetUrl = pathParts.join("/");

      if (!targetUrl) {
        debugInfo.error = "Missing repository path";
        debugInfo.duration_ms = performance.now() - startTime;
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
            debug_info: debugInfo,
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

      const maxSizeParam = url.searchParams.get("max_size");
      const ignoreHiddenParam = url.searchParams.get("ignore_hidden");
      const ignoreGitignoreParam = url.searchParams.get("ignore_gitignore");

      const options: AnalyzeOptions = {
        ignore_hidden: ignoreHiddenParam === "false" ? false : true,
        ignore_gitignore: ignoreGitignoreParam === "false" ? false : true,
        max_file_size:
          maxSizeParam === "-1"
            ? -1
            : maxSizeParam
              ? parseInt(maxSizeParam)
              : -1,
      };

      debugInfo.target_url = targetUrl;
      debugInfo.options = options;

      this.log("info", "Starting analysis", { url: targetUrl, options });

      const analysisStartTime = performance.now();
      const result = await this.wasmModule.analyze_url(targetUrl, options);
      const analysisEndTime = performance.now();

      debugInfo.analysis_duration_ms = analysisEndTime - analysisStartTime;
      debugInfo.total_duration_ms = analysisEndTime - startTime;

      if (result && result.wasm_debug_info) {
        Object.assign(debugInfo, result.wasm_debug_info);
        delete result.wasm_debug_info;
      }

      const response = {
        ...result,
        debug_info: debugInfo,
      };

      this.log("info", "Analysis completed successfully", debugInfo);

      return new Response(JSON.stringify(response), {
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
        },
      });
    } catch (error: unknown) {
      let errorMessage = error instanceof Error ? error.message : String(error);
      const errorStack = error instanceof Error ? error.stack : undefined;
      let errorType = "UnknownError";

      if (error && typeof error === "object" && "error" in error) {
        const wasmError = error as any;
        errorMessage = wasmError.error || errorMessage;
        errorType = wasmError.error_type || "WASMError";

        if (wasmError.wasm_debug_info) {
          Object.assign(debugInfo, wasmError.wasm_debug_info);
          delete wasmError.wasm_debug_info;
        }
      }

      debugInfo.error = errorMessage;
      debugInfo.error_type = errorType;
      debugInfo.error_stack = errorStack;
      debugInfo.duration_ms = performance.now() - startTime;

      if (errorMessage.includes("URL parsing error")) {
        debugInfo.error_category = "URL_PARSING";
        debugInfo.suggested_fix =
          "Please check the URL format. Use formats like: user/repo, user/repo@branch, or full GitHub URLs";
      } else if (
        errorMessage.includes("network") ||
        errorMessage.includes("download")
      ) {
        debugInfo.error_category = "NETWORK";
        debugInfo.suggested_fix =
          "Check your internet connection and ensure the repository is accessible";
      } else if (errorMessage.includes("branch")) {
        debugInfo.error_category = "BRANCH_ACCESS";
        debugInfo.suggested_fix =
          "The repository may not have the expected default branches (main, master, develop, dev)";
      } else {
        debugInfo.error_category = "UNKNOWN";
        debugInfo.suggested_fix =
          "Please check the error details and try again";
      }

      this.log("error", "Error in BytesRadar fetch", debugInfo);

      const errorResponse: any = {
        error: errorMessage,
        error_type: errorType,
        error_category: debugInfo.error_category,
        suggested_fix: debugInfo.suggested_fix,
        debug_info: debugInfo,
      };

      return new Response(JSON.stringify(errorResponse), {
        status: 500,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
        },
      });
    }
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
