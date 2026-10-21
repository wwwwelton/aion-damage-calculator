import { build } from 'esbuild';
import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { copyFile, mkdir, rename, rm } from 'node:fs/promises';
import { pipeline } from 'node:stream/promises';

const modelPath = 'assets/ai/models/HuggingFaceTB/SmolLM2-135M-Instruct/onnx/model_q4f16.onnx';
const modelSha256 = '9358cd4ce037c304621f8c194a525607ae7c5ea73239fcae4c21bd02f2e34ff7';
async function checksum(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

await mkdir('assets/ai/models/HuggingFaceTB/SmolLM2-135M-Instruct/onnx', { recursive: true });
let matches = false;
try { matches = await checksum(modelPath) === modelSha256; } catch (error) {
  if (error.code !== 'ENOENT') throw error;
}
if (!matches) {
  const url = 'https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct/resolve/a91318be21aeaf0879874faa161dcb40c68847e9/onnx/model_q4f16.onnx';
  const response = await fetch(url);
  if (!response.ok || !response.body) throw new Error(`Model download failed: HTTP ${response.status}`);
  const temporary = `${modelPath}.download`;
  try {
    await pipeline(response.body, createWriteStream(temporary));
    if (await checksum(temporary) !== modelSha256) throw new Error('Q4F16 model checksum mismatch.');
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
