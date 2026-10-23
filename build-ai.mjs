import { build } from 'esbuild';
import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { copyFile, mkdir, rename, rm } from 'node:fs/promises';
import { pipeline } from 'node:stream/promises';
import { convertToExternalData } from './onnx-external-data.mjs';

const revision = '22942cb7d7ba4cc81bb4673549ca4d4614469b5e';
const modelDir = 'assets/ai/models/external/onnx-community/Qwen2.5-0.5B-Instruct';
const sourcePath = `${modelDir}/onnx/model_quantized.source.onnx`;
const sourceSha256 = '41834041ab1b29eff9fc592f1a29a1844133aea35832ea9fa91682be13016100';
const modelPath = `${modelDir}/onnx/model_quantized.onnx`;
const dataPath = `${modelDir}/onnx/model_quantized.onnx_data`;
const modelSha256 = '497c8042369d8081e0b02df57ab5f36d74342cb9673a7cee5a9e55eceffbb1ba';
const dataSha256 = '48a4aeab7e96d13ca0e9cccc9df7fbcbb4e64886f1533c94f0cc0c7e4ecef40c';
async function checksum(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

await mkdir(`${modelDir}/onnx`, { recursive: true });
const verified = async (path, expected) => checksum(path).then(actual => actual === expected, error => {
  if (error.code !== 'ENOENT') throw error;
  return false;
});
if (!(await verified(modelPath, modelSha256) && await verified(dataPath, dataSha256))) {
  if (!await verified(sourcePath, sourceSha256)) {
    const url = `https://huggingface.co/onnx-community/Qwen2.5-0.5B-Instruct/resolve/${revision}/onnx/model_quantized.onnx`;
    const response = await fetch(url);
    if (!response.ok || !response.body) throw new Error(`Model download failed: HTTP ${response.status}`);
    const temporary = `${sourcePath}.download`;
    try {
      await pipeline(response.body, createWriteStream(temporary));
      if (await checksum(temporary) !== sourceSha256) throw new Error('Qwen2.5 q8 model checksum mismatch.');
      await rename(temporary, sourcePath);
    } finally {
      await rm(temporary, { force: true });
    }
  }
  // ONNX Runtime Web holds the protobuf and a copy of every weight while
  // loading a single-file model; the external-data layout removes ~0.5 GB of
  // resident memory for this model (see onnx-external-data.mjs).
  convertToExternalData(sourcePath, modelPath, dataPath);
  if (await checksum(modelPath) !== modelSha256 || await checksum(dataPath) !== dataSha256) {
    throw new Error('Converted Qwen2.5 q8 external-data files do not match the pinned checksums.');
  }
  await rm(sourcePath, { force: true });
}
for (const [file, expected] of Object.entries({
  'config.json': '777e01f0fbb3346eb229cb6fb278ed6533c1e4dcb9ebf4bed0f6e94ef17fa1b5',
  'generation_config.json': 'f7e7ce458658b2d40d9eb213b91b77a8bf698845ab89360976722d7ac46928a3',
  'tokenizer.json': 'a8506e7111b80c6d8635951a02eab0f4e1a8e4e5772da83846579e97b16f61bf',
  'tokenizer_config.json': '6a14243f0ae04884e597b3f032c05e15de9f417318fdcac24ab4e2680e691430'
})) {
  const path = `${modelDir}/${file}`;
  if (await verified(path, expected)) continue;
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
