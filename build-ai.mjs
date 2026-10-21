import { build } from 'esbuild';
import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { copyFile, mkdir, rename, rm } from 'node:fs/promises';
import { pipeline } from 'node:stream/promises';

const modelPath = 'assets/ai/models/onnx-community/Qwen3-0.6B-ONNX/onnx/model_q4f16.onnx';
const modelSha256 = '9e33a5911974174761d0dfdcc0bec975d9c45af0eae5e9eb647b8ba9442a8f91';
async function checksum(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

await mkdir('assets/ai/models/onnx-community/Qwen3-0.6B-ONNX/onnx', { recursive: true });
let matches = false;
try { matches = await checksum(modelPath) === modelSha256; } catch (error) {
  if (error.code !== 'ENOENT') throw error;
}
if (!matches) {
  const url = 'https://huggingface.co/onnx-community/Qwen3-0.6B-ONNX/resolve/main/onnx/model_q4f16.onnx';
  const response = await fetch(url);
  if (!response.ok || !response.body) throw new Error(`Model download failed: HTTP ${response.status}`);
  const temporary = `${modelPath}.download`;
  try {
    await pipeline(response.body, createWriteStream(temporary));
    if (await checksum(temporary) !== modelSha256) throw new Error('Qwen3 Q4F16 model checksum mismatch.');
    await rename(temporary, modelPath);
  } finally {
    await rm(temporary, { force: true });
  }
}

await build({ entryPoints: ['ai-worker.js'], outfile: 'ai-worker.bundle.js', bundle: true, format: 'esm', target: 'es2022' });
await mkdir('assets/ai', { recursive: true });
for (const file of ['ort-wasm-simd-threaded.jsep.wasm', 'ort-wasm-simd-threaded.jsep.mjs']) {
  await copyFile(`node_modules/@huggingface/transformers/dist/${file}`, `assets/ai/${file}`);
}
