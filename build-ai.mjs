import { build } from 'esbuild';
import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { copyFile, mkdir, rename, rm } from 'node:fs/promises';
import { pipeline } from 'node:stream/promises';

const revision = '22942cb7d7ba4cc81bb4673549ca4d4614469b5e';
const modelDir = 'assets/ai/models/onnx-community/Qwen2.5-0.5B-Instruct';
const modelPath = `${modelDir}/onnx/model_quantized.onnx`;
const modelSha256 = '41834041ab1b29eff9fc592f1a29a1844133aea35832ea9fa91682be13016100';
async function checksum(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

await mkdir(`${modelDir}/onnx`, { recursive: true });
let matches = false;
try { matches = await checksum(modelPath) === modelSha256; } catch (error) {
  if (error.code !== 'ENOENT') throw error;
}
if (!matches) {
  const url = `https://huggingface.co/onnx-community/Qwen2.5-0.5B-Instruct/resolve/${revision}/onnx/model_quantized.onnx`;
  const response = await fetch(url);
  if (!response.ok || !response.body) throw new Error(`Model download failed: HTTP ${response.status}`);
  const temporary = `${modelPath}.download`;
  try {
    await pipeline(response.body, createWriteStream(temporary));
    if (await checksum(temporary) !== modelSha256) throw new Error('Qwen2.5 q8 model checksum mismatch.');
    await rename(temporary, modelPath);
  } finally {
    await rm(temporary, { force: true });
  }
}
for (const [file, expected] of Object.entries({
  'config.json': '777e01f0fbb3346eb229cb6fb278ed6533c1e4dcb9ebf4bed0f6e94ef17fa1b5',
  'generation_config.json': 'f7e7ce458658b2d40d9eb213b91b77a8bf698845ab89360976722d7ac46928a3',
  'tokenizer.json': 'a8506e7111b80c6d8635951a02eab0f4e1a8e4e5772da83846579e97b16f61bf',
  'tokenizer_config.json': '6a14243f0ae04884e597b3f032c05e15de9f417318fdcac24ab4e2680e691430'
})) {
  const path = `${modelDir}/${file}`;
  if (await checksum(path).catch(error => {
    if (error.code !== 'ENOENT') throw error;
    return null;
  }) === expected) continue;
  const response = await fetch(`https://huggingface.co/onnx-community/Qwen2.5-0.5B-Instruct/resolve/${revision}/${file}`);
  if (!response.ok || !response.body) throw new Error(`${file} download failed: HTTP ${response.status}`);
  const temporary = `${path}.download`;
  try {
    await pipeline(response.body, createWriteStream(temporary));
    if (await checksum(temporary) !== expected) throw new Error(`${file} checksum mismatch`);
    await rename(temporary, path);
  } finally {
    await rm(temporary, { force: true });
  }
}

await build({ entryPoints: ['ai-worker.js'], outfile: 'ai-worker.bundle.js', bundle: true, format: 'esm', target: 'es2022' });
await mkdir('assets/ai', { recursive: true });
for (const file of ['ort-wasm-simd-threaded.jsep.wasm', 'ort-wasm-simd-threaded.jsep.mjs']) {
  await copyFile(`node_modules/@huggingface/transformers/dist/${file}`, `assets/ai/${file}`);
}
