import { pipeline, env } from '@huggingface/transformers';

const MODEL = 'onnx-community/Qwen3-0.6B-ONNX';
env.allowLocalModels = true;
env.allowRemoteModels = false;
env.localModelPath = new URL('assets/ai/models/', self.location.href).href;
env.backends.onnx.wasm.wasmPaths = new URL('assets/ai/', self.location.href).href;
let generator;
let pending;
let running = false;

async function generate() {
  if (running) return;
  running = true;
  while (pending) {
    const { id, question, context, research, history, language = 'en' } = pending;
    pending = null;
    try {
      if (!generator) {
        postMessage({ id, state: 'loading' });
        generator = await pipeline('text-generation', MODEL, {
          device: 'webgpu', dtype: 'q4f16',
          progress_callback: progress => {
            if (progress.status === 'progress') postMessage({ id, state: 'downloading', progress: Math.round(progress.progress) });
          }
        });
      }
      postMessage({ id, state: 'generating' });

      const portuguese = language === 'pt';
      const systemPrompt = portuguese
        ? 'Você é o consultor de combate Avatarnoia Flow para AION 2. Responda somente em português brasileiro, de forma direta e concisa.\n'
          + `Cálculo ativo da configuração:\n${context}\n`
          + (research ? `Nota de guia do AION 2: ${research}\n` : '')
          + 'Priorize atributos, coeficientes de habilidade e mitigação com base nos números fornecidos. Não mostre raciocínio interno nem tags <think>.'
        : 'You are the Avatarnoia Flow combat advisor for AION 2. Answer only in concise English.\n'
          + `Active build calculation:\n${context}\n`
          + (research ? `AION 2 Guide Note: ${research}\n` : '')
          + 'Prioritize stats, skill coefficients and mitigation from the supplied numbers. Do not show internal reasoning or <think> tags.';

      const messages = [{ role: 'system', content: systemPrompt }];

      if (Array.isArray(history)) {
        for (const item of history.slice(-6)) {
          if (item?.role && item?.content) {
            messages.push({ role: item.role, content: item.content });
          }
        }
      }

      const query = question || (portuguese
        ? 'Qual atributo devo melhorar agora para aumentar meu DPS?'
        : 'Which stat should I improve right now to increase my DPS?');
      if (!history || history.length === 0 || history[history.length - 1]?.content !== query) {
        messages.push({ role: 'user', content: query });
      }

      const output = await generator(messages, {
        max_new_tokens: 180,
        repetition_penalty: 1.15,
        do_sample: false,
        chat_template_kwargs: { enable_thinking: false }
      });

      const raw = output[0]?.generated_text?.at(-1)?.content || '';
      let text = raw.replace(/<think>[\s\S]*?<\/think>/gi, '').trim();
      if (!text && raw.includes('<think>')) {
        text = raw.replace(/<\/?think>/gi, '').trim();
      }
      if (!text) {
        text = portuguese
          ? 'Priorize o primeiro atributo da comparação calculada abaixo para maximizar seu DPS.'
          : 'Prioritize the top stat listed in the calculated upgrades below to maximize your DPS.';
      }
      postMessage({ id, state: 'done', text });
    } catch (error) {
      postMessage({ id, state: 'error', error: error.message || String(error) });
    }
  }
  running = false;
}

onmessage = ({ data }) => {
  pending = data;
  void generate();
};
