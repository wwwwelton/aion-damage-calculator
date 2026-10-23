import { pipeline, env } from '@huggingface/transformers';

const MODEL = 'onnx-community/Qwen2.5-0.5B-Instruct';
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
          device: 'wasm', dtype: 'q8',
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
          + 'Responda à pergunta com uma recomendação de jogo em uma ou duas frases de prosa. Cite a melhor melhoria calculada e um atributo do inimigo quando forem relevantes. Não escreva código, listas de programação ou dados inventados. Valores iniciais são exemplos, não dados confirmados do jogador.'
        : 'You are the Avatarnoia Flow combat advisor for AION 2. Answer only in concise English.\n'
          + `Active build calculation:\n${context}\n`
          + (research ? `AION 2 Guide Note: ${research}\n` : '')
          + 'Answer the question with a gameplay recommendation in one or two prose sentences. Mention the top calculated upgrade and a relevant enemy stat when available. Do not write code or invent stats. Prefilled values are examples, not confirmed player data.';

      const messages = [{ role: 'system', content: systemPrompt }];

      if (Array.isArray(history)) {
        for (const item of history.slice(-6)) {
          if (item?.role && item?.content) {
            messages.push({ role: item.role, content: item.content });
          }
        }
      }

      const query = question || 'Como melhorar minha build ou DPS';
      if (!history || history.length === 0 || history[history.length - 1]?.content !== query) {
        messages.push({ role: 'user', content: query });
      }

      const output = await generator(messages, {
        max_new_tokens: 120,
        repetition_penalty: 1.15,
        do_sample: false
      });
      const raw = output[0]?.generated_text?.at(-1)?.content || '';
      const text = raw.trim();
      if (!text) throw new Error('Local model returned an empty response.');
      postMessage({ id, state: 'done', text });
    } catch (error) {
      console.error('AI Worker error:', error);
      console.error('AI Worker error stack:', error.stack);
      postMessage({ id, state: 'error', error: error.message || String(error) });
    }
  }
  running = false;
}

onmessage = ({ data }) => {
  pending = data;
  void generate();
};
