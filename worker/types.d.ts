declare module "*.wasm" {
  const content: WebAssembly.Module;
  export default content;
}

declare module "*.wasm?module" {
  const content: WebAssembly.Module;
  export default content;
}

declare module "*.wasm?url" {
  const content: string;
  export default content;
}

export interface IntelligentFilter {
  max_file_size: number;
  ignore_hidden: boolean;
  ignore_gitignore: boolean;
}

export interface AnalysisOptions {
  timeout?: number;
  max_redirects?: number;
  user_agent?: string;
  accept_invalid_certs: boolean;
  headers: Record<string, string>;
  credentials: Record<string, string>;
  provider_settings: Record<string, string>;
  max_file_size?: number;
  use_compression: boolean;
  proxy?: string;
  ignore_hidden: boolean;
  ignore_gitignore: boolean;
  aggressive_filtering?: boolean;
  custom_filter?: IntelligentFilter;
}
