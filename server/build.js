const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

if (!fs.existsSync('pkg')) {
  fs.mkdirSync('pkg');
}

console.log('Building WebAssembly module...');
execSync('wasm-pack build --target web --out-dir server/pkg --no-default-features --features worker', {
  stdio: 'inherit',
  cwd: path.resolve(__dirname, '..'),
});

console.log('Generating TypeScript types...');
const typesContent = `
export interface AnalyzeOptions {
  ignore_hidden: boolean;
  ignore_gitignore: boolean;
  max_file_size: number;
}

export function analyze_url(url: string, options: AnalyzeOptions): Promise<any>;
`;

fs.writeFileSync(path.join(__dirname, 'pkg', 'bytes_radar.d.ts'), typesContent);

console.log('Build complete!'); 