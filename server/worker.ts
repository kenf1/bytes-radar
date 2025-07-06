import type { AnalyzeOptions } from './pkg/bytes_radar';
import wasmBinary from './pkg/bytes_radar_bg.wasm';

export interface Env {
  BYTES_RADAR: DurableObjectNamespace;
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

  private async initializeWasm() {
    if (!this.wasmInitialized) {
      try {
        this.wasmModule = await import('./pkg/bytes_radar');
        
        // Initialize the WebAssembly module with the imported binary
        await this.wasmModule.default(wasmBinary);
        
        this.wasmInitialized = true;
        console.log('WebAssembly module initialized successfully');
      } catch (error) {
        console.error('Failed to initialize WebAssembly module:', error);
        throw error;
      }
    }
  }

  async fetch(request: Request) {
    try {
      await this.initializeWasm();
      
      const url = new URL(request.url);
      const targetUrl = url.searchParams.get('url');
      
      if (!targetUrl) {
        return new Response('Missing url parameter', { status: 400 });
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

      console.log('Analyzing URL:', targetUrl, 'with options:', options);
      
      const result = await this.wasmModule.analyze_url(targetUrl, options);
      return new Response(JSON.stringify(result), {
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
        },
      });
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Error in BytesRadar fetch:', errorMessage);
      return new Response(errorMessage, { status: 500 });
    }
  }
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const id = env.BYTES_RADAR.idFromName('default');
    const obj = env.BYTES_RADAR.get(id);
    return obj.fetch(request);
  },
}; 