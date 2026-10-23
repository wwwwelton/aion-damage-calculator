import { pipeline, env } from '@huggingface/transformers';
import { resolve } from 'node:path';

const root = resolve(process.argv[2] || 'assets/ai/models/external');
const model = 'onnx-community/Qwen2.5-0.5B-Instruct';
env.allowLocalModels = true;
env.allowRemoteModels = false;
env.localModelPath = `${root}/`;

const generator = await pipeline('text-generation', model, { device: 'cpu', dtype: 'q8', use_external_data_format: true });
for (const [language, question] of [
  ['English', 'Which combat stat should I prioritize for damage? Answer in one short English sentence.'],
  ['Português', 'Qual atributo de combate devo priorizar para causar mais dano? Responda em uma frase curta em português brasileiro.']
]) {
  const output = await generator([
    { role: 'system', content: `You are a concise AION 2 combat advisor. Answer in ${language}.` },
    { role: 'user', content: question }
  ], { max_new_tokens: 48, do_sample: false });
  const answer = output[0]?.generated_text?.at(-1)?.content?.trim();
  if (!answer || answer === question) throw new Error(`No generated ${language} answer`);
  console.log(`${language}: ${answer}`);
}
