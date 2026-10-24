import { build } from 'esbuild';
import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream } from 'node:fs';
import { copyFile, mkdir, rename, rm } from 'node:fs/promises';
import { pipeline } from 'node:stream/promises';
import { convertToExternalData } from './onnx-external-data.mjs';

const revision = '58ca03a5e7b354b480b1a82b34bde8dc7b854cc0';
const modelDir = 'assets/ai/models/external/onnx-community/Qwen3-0.6B-ONNX';
const sourcePath = `${modelDir}/onnx/model_quantized.source.onnx`;
const sourceSha256 = 'ccf8734e59fdf7475b2dc0c117005e267a3a08342c1d951e0a79f0bce57bf62a';
const modelPath = `${modelDir}/onnx/model_quantized.onnx`;
const dataPath = `${modelDir}/onnx/model_quantized.onnx_data`;
const modelSha256 = '2e74feff0dd03ce77d54036dafffd85e75b8bcb6945e9288f0832816aacbbea0';
const dataSha256 = '13f689fcf04480236bf928c6631d48fa6b11a30195d4851e4e526528cdba4e35';
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
    const url = `https://huggingface.co/onnx-community/Qwen3-0.6B-ONNX/resolve/${revision}/onnx/model_quantized.onnx`;
    const response = await fetch(url);
    if (!response.ok || !response.body) throw new Error(`Model download failed: HTTP ${response.status}`);
    const temporary = `${sourcePath}.download`;
    try {
      await pipeline(response.body, createWriteStream(temporary));
      if (await checksum(temporary) !== sourceSha256) throw new Error('Qwen3 q8 model checksum mismatch.');
      await rename(temporary, sourcePath);
    } finally {
      await rm(temporary, { force: true });
    }
  }
  // ONNX Runtime Web holds the protobuf and a copy of every weight while
  // loading a single-file model; the external-data layout removes ~0.6 GB of
  // resident memory for this model (see onnx-external-data.mjs).
  convertToExternalData(sourcePath, modelPath, dataPath);
  if (await checksum(modelPath) !== modelSha256 || await checksum(dataPath) !== dataSha256) {
    throw new Error('Converted Qwen3 q8 external-data files do not match the pinned checksums.');
  }
  await rm(sourcePath, { force: true });
}
for (const [file, expected] of Object.entries({
  'config.json': '8a04114ba59cc42b47d804d35d1d5c61d746ae4634f41f796768c6e302d39b9e',
  'generation_config.json': '9e9e031ae8bca36eefcdcdd0b35d83c61baa01e31d68d5f5a961c9ca1a4b95bf',
  'tokenizer.json': 'e7a95fce95bf5b0946d0ddb3f9d7caa030b7e850bbe92b0edb26bcf563e9f3d5',
  'tokenizer_config.json': '7c5194eecfd95a4dcbd6b4b502af785d2541fd65a388d1cae7ad43693c67b802'
})) {
  const path = `${modelDir}/${file}`;
  if (await verified(path, expected)) continue;
  const response = await fetch(`https://huggingface.co/onnx-community/Qwen3-0.6B-ONNX/resolve/${revision}/${file}`);
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
