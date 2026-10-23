import { pipeline, env } from '@huggingface/transformers';

const MODEL = 'onnx-community/Qwen2.5-0.5B-Instruct';
env.allowLocalModels = true;
env.allowRemoteModels = false;
env.useBrowserCache = false; // assets are embedded; caching would duplicate 512 MB in the profile
env.localModelPath = new URL('assets/ai/models/external/', self.location.href).href;
env.backends.onnx.wasm.wasmPaths = new URL('assets/ai/', self.location.href).href;
let generator;
let pending;
let running = false;
const MAX_PROMPT_TOKENS = 320;

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
          device: 'wasm', dtype: 'q8', use_external_data_format: true,
          progress_callback: progress => {
            if (progress.status === 'progress') postMessage({ id, state: 'downloading', progress: Math.round(progress.progress) });
          }
        });
      }
      postMessage({ id, state: 'generating' });

      const portuguese = language === 'pt';
      const turns = (Array.isArray(history) ? history : []).filter(item => item?.role && item?.content).slice(-6);
      const query = question || 'Como melhorar minha build ou DPS';
      if (turns.at(-1)?.content !== query) turns.push({ role: 'user', content: query });
      const buildMessages = (buildContext, note) => [{ role: 'system', content: portuguese
        ? 'Você é o consultor de combate Avatarnoia Flow para AION 2. Responda somente em português brasileiro, de forma direta e concisa.\n'
          + `Cálculo ativo da configuração:\n${buildContext}\n`
          + (note ? `Nota de guia do AION 2: ${note}\n` : '')
          + 'Responda à pergunta com uma recomendação de jogo em uma ou duas frases de prosa. Cite a melhor melhoria calculada e um atributo do inimigo quando forem relevantes. Não escreva código, listas de programação ou dados inventados. Valores iniciais são exemplos, não dados confirmados do jogador.'
        : 'You are the Avatarnoia Flow combat advisor for AION 2. Answer only in concise English.\n'
          + `Active build calculation:\n${buildContext}\n`
          + (note ? `AION 2 Guide Note: ${note}\n` : '')
          + 'Answer the question with a gameplay recommendation in one or two prose sentences. Mention the top calculated upgrade and a relevant enemy stat when available. Do not write code or invent stats. Prefilled values are examples, not confirmed player data.'
      }, ...turns];

      // The exported graph emits logits for every prompt position and the WASM
      // heap never shrinks, so each prompt token costs ~1.3 MB of resident
      // memory for the rest of the session. Trim oldest history first, then the
      // guide note, then the build context, until the prompt fits.
      const promptTokens = messages => generator.tokenizer.apply_chat_template(messages,
        { tokenize: true, add_generation_prompt: true, return_tensor: false }).length;
      let note = research;
      let buildContext = context;
      let messages = buildMessages(buildContext, note);
      while (promptTokens(messages) > MAX_PROMPT_TOKENS) {
        if (turns.length > 1) turns.splice(0, turns.length > 2 ? 2 : 1);
        else if (note) note = '';
        else if (buildContext.length > 200) buildContext = buildContext.slice(0, Math.floor(buildContext.length / 2));
        else break;
        messages = buildMessages(buildContext, note);
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
