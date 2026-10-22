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
    const { id, question, context, research, history } = pending;
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

      const systemPrompt = 'You are the Avatarnoia Flow combat advisor for AION 2.\n'
        + `Active build calculation:\n${context}\n`
        + (research ? `AION 2 Guide Note: ${research}\n` : '')
        + 'Guidelines:\n'
        + '- Give direct, practical, mathematically sound English advice for AION 2.\n'
        + '- Target stat priorities, skill coefficients, or mitigation.\n'
        + '- Do NOT output <think> tags or internal thinking. Answer directly.';

      const messages = [{ role: 'system', content: systemPrompt }];

      if (Array.isArray(history)) {
        for (const item of history.slice(-6)) {
          if (item?.role && item?.content) {
            messages.push({ role: item.role, content: item.content });
          }
        }
      }

      const query = question || 'Which stat should I improve right now to increase my DPS?';
      if (!history || history.length === 0 || history[history.length - 1]?.content !== query) {
        messages.push({ role: 'user', content: query });
      }

      const output = await generator(messages, {
        max_new_tokens: 220,
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
        text = 'Prioritize the top stat listed in the calculated upgrades below to maximize your DPS.';
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
