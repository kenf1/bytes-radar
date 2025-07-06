import type { AnalyzeOptions } from './pkg/bytes_radar';
import wasmBinary from './pkg/bytes_radar_bg.wasm';

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

  private log(level: 'debug' | 'info' | 'warn' | 'error', message: string, data?: any) {
    const logLevel = this.env.LOG_LEVEL || 'info';
    const environment = this.env.ENVIRONMENT || 'development';
    
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
        ...(data && { data })
      };
      
      console.log(`[${level.toUpperCase()}] ${message}`, data ? data : '');
      
      if (environment === 'production') {
        console.log(JSON.stringify(logEntry));
      }
    }
  }

  private async initializeWasm() {
    if (!this.wasmInitialized) {
      try {
        this.wasmModule = await import('./pkg/bytes_radar');
        await this.wasmModule.default(wasmBinary);
        this.wasmInitialized = true;
        this.log('info', 'WebAssembly module initialized successfully');
      } catch (error) {
        this.log('error', 'Failed to initialize WebAssembly module', { 
          error: error instanceof Error ? error.message : String(error) 
        });
        throw error;
      }
    }
  }

  async fetch(request: Request) {
    const startTime = performance.now();
    const debugInfo: any = {
      timestamp: new Date().toISOString(),
      wasm_initialized: this.wasmInitialized,
    };
    
    try {
      await this.initializeWasm();
      debugInfo.wasm_initialized = true;
      
      const url = new URL(request.url);
      const pathParts = url.pathname.split('/').filter(Boolean);
      const targetUrl = pathParts.join('/');
      
      if (!targetUrl) {
        debugInfo.error = 'Missing repository path';
        debugInfo.duration_ms = performance.now() - startTime;
        return new Response(JSON.stringify({
          error: 'Missing repository path',
          debug_info: debugInfo
        }), { 
          status: 400,
          headers: {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*',
          }
        });
      }

      const maxSizeParam = url.searchParams.get('max_size');
      const ignoreHiddenParam = url.searchParams.get('ignore_hidden');
      const ignoreGitignoreParam = url.searchParams.get('ignore_gitignore');

      const options: AnalyzeOptions = {
        ignore_hidden: ignoreHiddenParam === 'false' ? false : true,
        ignore_gitignore: ignoreGitignoreParam === 'false' ? false : true,
        max_file_size: maxSizeParam === '-1' ? -1 : 
                      maxSizeParam ? parseInt(maxSizeParam) : 
                      -1,
      };

      debugInfo.target_url = targetUrl;
      debugInfo.options = options;
      
      this.log('info', 'Starting analysis', { url: targetUrl, options });
      
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
        debug_info: debugInfo
      };
      
      console.log('Analysis completed successfully:', {
        url: targetUrl,
        project: debugInfo.project_name,
        files: debugInfo.files_analyzed,
        lines: debugInfo.total_lines,
        languages: debugInfo.languages_detected,
        size: debugInfo.total_size_formatted,
        duration: debugInfo.total_duration_ms + 'ms'
      });
      
      return new Response(JSON.stringify(response), {
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        },
      });
    } catch (error: unknown) {
      let errorMessage = error instanceof Error ? error.message : String(error);
      const errorStack = error instanceof Error ? error.stack : undefined;
      let errorType = 'UnknownError';
      
      if (error && typeof error === 'object' && 'error' in error) {
        const wasmError = error as any;
        errorMessage = wasmError.error || errorMessage;
        errorType = wasmError.error_type || 'WASMError';
        
        if (wasmError.wasm_debug_info) {
          Object.assign(debugInfo, wasmError.wasm_debug_info);
          delete wasmError.wasm_debug_info;
        }
      }
      
      debugInfo.error = errorMessage;
      debugInfo.error_type = errorType;
      debugInfo.error_stack = errorStack;
      debugInfo.duration_ms = performance.now() - startTime;
      
      if (errorMessage.includes('URL parsing error')) {
        debugInfo.error_category = 'URL_PARSING';
        debugInfo.suggested_fix = 'Please check the URL format. Use formats like: user/repo, user/repo@branch, or full GitHub URLs';
      } else if (errorMessage.includes('network') || errorMessage.includes('download')) {
        debugInfo.error_category = 'NETWORK';
        debugInfo.suggested_fix = 'Check your internet connection and ensure the repository is accessible';
      } else if (errorMessage.includes('branch')) {
        debugInfo.error_category = 'BRANCH_ACCESS';
        debugInfo.suggested_fix = 'The repository may not have the expected default branches (main, master, develop, dev)';
      } else {
        debugInfo.error_category = 'UNKNOWN';
        debugInfo.suggested_fix = 'Please check the error details and try again';
      }
      
      console.error('Error in BytesRadar fetch:', {
        error: errorMessage,
        type: errorType,
        category: debugInfo.error_category,
        stack: errorStack,
        url: debugInfo.target_url,
        duration: debugInfo.duration_ms + 'ms'
      });
      
      const errorResponse: any = {
        error: errorMessage,
        error_type: errorType,
        error_category: debugInfo.error_category,
        suggested_fix: debugInfo.suggested_fix,
        debug_info: debugInfo
      };
      
      return new Response(JSON.stringify(errorResponse), { 
        status: 500,
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        }
      });
    }
  }

  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const id = env.BYTES_RADAR.idFromName('default');
    const obj = env.BYTES_RADAR.get(id);
    return obj.fetch(request);
  },
}; 