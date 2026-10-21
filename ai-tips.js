const aiQuestion = document.querySelector('#ai-question');
const aiAnswer = document.querySelector('#ai-answer');
const aiSource = document.querySelector('#ai-source');
const aiApply = document.querySelector('#ai-apply');
let aiWorker;
let aiStartup;
let aiTimer;
let aiRevision = 0;
let researchRequest = -1;
const researchPending = new Map();
const guideCache = new Map();

function guideTopic(question) {
  if (/creature|species|cogni|fera|natura|varian/i.test(question)) return 'creature';
  if (/critical|block|evasion|chance|resist|double/i.test(question)) return 'chance';
  if (document.querySelector('#target-mode').value === 'player') return 'pvp';
  if (/attack|weapon|penetration/i.test(question)) return 'attack';
  return 'pve';
}

function currentAiContext() {
  const important = new Set(['pureAttack', 'weaponDamageBoost', 'damageBoost', 'skillCoefficient',
    'criticalDamageBoost', 'defense', 'penetration', 'enemyHp', 'hitsPerSecond']);
  const fields = [...document.querySelectorAll('#calculator input')]
    .filter(input => input.name && (important.has(input.name) || input.value !== input.defaultValue && input.value !== '0'))
    .sort((a, b) => Number(important.has(b.name)) - Number(important.has(a.name)))
    .slice(0, 12).map(input => `${input.name} ${input.value}`);
  const attacker = document.querySelector('#attacker-name').value || 'unnamed attacker';
  const target = document.querySelector('#target-name').value || 'unnamed target';
  return `${attacker} (${document.querySelector('#attacker-class').value || 'unknown class'}) versus ${target} (${document.querySelector('#target-mode').value}, ${document.querySelector('#creature-type-header').value || 'unknown creature'}). `
    + `Estimated hit ${document.querySelector('#result').textContent}; DPS ${document.querySelector('#estimated-dps').textContent}. `
    + `Critical ${document.querySelector('#critical').value}; position ${document.querySelector('#position').value}. `
    + `Best +1% gains: ${document.querySelector('#upgrade-list').textContent.slice(0, 110) || 'unknown'}. `
    + `Current fields: ${fields.join(', ')}.`;
}

function requestGuide(topic) {
  const id = researchRequest--;
  return new Promise((resolve, reject) => {
    researchPending.set(id, { resolve, reject });
    window.chrome.webview.postMessage({ id, kind: 'research', topic });
  });
}
if (window.chrome?.webview) {
  window.chrome.webview.addEventListener('message', ({ data }) => {
    const pending = researchPending.get(data?.id);
    if (!pending) return;
    researchPending.delete(data.id);
    if (data.error) pending.reject(new Error(data.error));
    else pending.resolve(data.data);
  });
}

function recommendedUpgrade() {
  const first = document.querySelector('#upgrade-list li')?.textContent || '';
  const label = first.split(':')[0];
  const map = { 'Weapon Damage Boost': 'weaponDamageBoost', 'Damage Boost': 'damageBoost',
    'Critical Damage Boost': 'criticalDamageBoost', 'Front Attack Damage Boost': 'frontBoost',
    'Back Attack Damage Boost': 'backBoost' };
  const field = map[label];
  const input = field && document.querySelector(`#calculator input[name="${field}"]`);
  return input ? { input, label, gain: first.split(': ')[1] } : null;
}

async function ensureAiWorker() {
  if (aiStartup) return aiStartup;
  aiStartup = (async () => {
    const adapter = await navigator.gpu?.requestAdapter();
    if (!adapter?.features.has('shader-f16')) throw new Error('Q4F16 requires WebGPU with shader-f16 support.');
    aiWorker = new Worker('ai-worker.bundle.js', { type: 'module' });
    aiWorker.onmessage = ({ data }) => {
      if (data.id !== aiRevision) return;
      if (data.state === 'loading') aiAnswer.textContent = 'Loading bundled Avatarnoia Flow Engine 2.0…';
      if (data.state === 'downloading') aiAnswer.textContent = `Reading embedded model: ${data.progress}%`;
      if (data.state === 'done') aiAnswer.textContent = data.text || 'The model did not return a suggestion. Use the calculated comparison below.';
      if (data.state === 'error') aiAnswer.textContent = `AI unavailable: ${data.error}. Use the calculated comparison below.`;
    };
    aiWorker.onerror = () => {
      aiAnswer.textContent = 'Local model failed to start. Use the calculated comparison below.';
    };
  })();
  return aiStartup;
}

async function generateAiTip(id) {
  const question = aiQuestion.value.trim();
  const context = currentAiContext();
  let research = '';
  aiSource.textContent = '';
  if (window.chrome?.webview) {
    const topic = guideTopic(question);
    if (!guideCache.has(topic)) guideCache.set(topic, requestGuide(topic));
    try {
      const guide = await guideCache.get(topic);
      if (id !== aiRevision) return;
      research = guide.excerpt.slice(0, 280);
      aiSource.textContent = `Online guide: ${guide.url}`;
    } catch (error) {
      if (id !== aiRevision) return;
      aiSource.textContent = `Guide unavailable (${error.message}); local calculator values remain available.`;
    }
  }
  if (id !== aiRevision) return;
  try {
    await ensureAiWorker();
    if (id !== aiRevision) return;
    aiAnswer.textContent = 'Generating AION 2 advice locally…';
    aiWorker.postMessage({ id, question, context, research });
  } catch (error) {
    if (id === aiRevision) aiAnswer.textContent = `${error.message} Use the calculated comparison below.`;
  }
}

function scheduleAiTips() {
  clearTimeout(aiTimer);
  const id = ++aiRevision;
  const suggestion = recommendedUpgrade();
  aiApply.hidden = !suggestion;
  if (suggestion) aiApply.textContent = `Apply +1% ${suggestion.label} (${suggestion.gain})`;
  aiTimer = setTimeout(() => void generateAiTip(id), 850);
}
aiApply.addEventListener('click', () => {
  const suggestion = recommendedUpgrade();
  if (!suggestion) return;
  const value = parseNumericInput(suggestion.input.value, 0);
  if (value === null) return;
  suggestion.input.value = String(value + 1);
  suggestion.input.dispatchEvent(new Event('input', { bubbles: true }));
});
aiQuestion.addEventListener('input', scheduleAiTips);
for (const selector of ['#attacker-name', '#attacker-class', '#target-name', '#target-class', '#target-mode', '#creature-type-header', '#critical', '#position']) {
  document.querySelector(selector).addEventListener('input', scheduleAiTips);
  document.querySelector(selector).addEventListener('change', scheduleAiTips);
}
