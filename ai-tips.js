const aiQuestion = document.querySelector('#ai-question');
const aiAnswer = document.querySelector('#ai-answer');
const aiSource = document.querySelector('#ai-source');
const aiApply = document.querySelector('#ai-apply');
const aiChatDialog = document.querySelector('#ai-chat-dialog');
const aiSend = document.querySelector('#ai-send');
const pending = new Map();
const conversationHistory = [];
const guideCache = new Map();
let requestId = -1000000;

function scrollChatToBottom() {
  aiChatDialog?.scrollTo({ top: aiChatDialog.scrollHeight, behavior: 'smooth' });
}

function appendChatMessage(role, author, text) {
  if (!aiChatDialog) return null;
  const msg = document.createElement('div');
  msg.className = `ai-msg ai-msg-${role}`;
  const authorSpan = document.createElement('span');
  authorSpan.className = 'ai-msg-author';
  authorSpan.textContent = author;
  const body = document.createElement('div');
  body.className = 'ai-msg-body';
  body.textContent = text;
  msg.append(authorSpan, body);
  aiChatDialog.append(msg);
  scrollChatToBottom();
  return body;
}

function hostRequest(kind, values = {}) {
  if (!window.aion2Host) return Promise.reject(new Error(I18N.t('requiresWindows')));
  const id = ++requestId;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    window.aion2Host.post({ id, kind, ...values });
  });
}

window.aion2Host?.onMessage(({ data }) => {
  const request = pending.get(data?.id);
  if (!request) return;
  pending.delete(data.id);
  if (data.error) request.reject(new Error(data.error));
  else request.resolve(data.data);
});

function guideTopic(question) {
  if (/creature|species|cogni|fera|natura|varian/i.test(question)) return 'creature';
  if (/critical|block|evasion|chance|resist|double/i.test(question)) return 'chance';
  if (document.querySelector('#target-mode').value === 'player') return 'pvp';
  if (/attack|weapon|penetration/i.test(question)) return 'attack';
  return 'pve';
}

function currentAiContext() {
  const attacker = document.querySelector('#attacker-name').value || 'Player';
  const aClass = document.querySelector('#attacker-class').value || 'Unspecified class';
  const target = document.querySelector('#target-name').value || 'Target';
  const mode = document.querySelector('#target-mode').value;
  const type = document.querySelector('#creature-type-header').value || 'Unknown';
  const targetClass = mode === 'player' ? document.querySelector('#target-class').value || 'Unspecified class' : '';
  const characterStats = [];
  const enemyStats = [];
  for (const input of document.querySelectorAll('#calculator input[name]')) {
    if (input.closest('[hidden]') || (mode === 'player' && input.closest('.pve-only, .boss-only'))
        || (mode === 'monster' && input.closest('.boss-only, .player-only'))
        || (mode === 'boss' && input.closest('.player-only'))) continue;
    if (!input.value || input.value === '0' || input.value === input.defaultValue) continue;
    (input.closest('label.enemy') ? enemyStats : characterStats).push(`${input.name}: ${input.value}`);
  }
  const upgrades = [...document.querySelectorAll('#upgrade-list li')].slice(0, 3).map(li => li.textContent.trim()).join(' | ') || 'None';
  const assumptions = [...document.querySelectorAll('#estimate-assumptions li')].map(li => li.textContent).join(' ');
  return `Character: ${attacker}; class: ${aClass}\nTarget: ${target}; mode: ${mode}; type: ${type}${targetClass ? `; class: ${targetClass}` : ''}\nCharacter stats: ${characterStats.join(', ')}\nTarget stats: ${enemyStats.join(', ')}\nCalculated hit: ${document.querySelector('#result').textContent}; estimated DPS: ${document.querySelector('#estimated-dps').textContent}\nCalculated +1% upgrades: ${upgrades}\nEstimate assumptions: ${assumptions}`;
}

async function requestGuide(topic) {
  if (!guideCache.has(topic)) {
    guideCache.set(topic, hostRequest('research', { topic }).catch(error => {
      guideCache.delete(topic);
      throw error;
    }));
  }
  return guideCache.get(topic);
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

async function dispatchAiRequest(question, target) {
  aiSend.disabled = true;
  target.textContent = I18N.t('aiGenerating');
  aiSource.textContent = '';
  scrollChatToBottom();
  try {
    let guide = '';
    try {
      const result = await requestGuide(guideTopic(question));
      guide = result.excerpt.slice(0, 280);
      aiSource.textContent = I18N.format('aiGuide', { url: result.url });
    } catch (error) {
      aiSource.textContent = I18N.format('aiGuideUnavailable', { error: error.message });
    }
    const model = localStorage.getItem('aionLabsModel') || 'aion-labs/aion-3.5-mini';
    const result = await hostRequest('ai_chat', { model, messages: [
      { role: 'system', content: `You are the expert combat analyst for AION 2. All responses are generated directly by this remote model without any local inference. Deliver authoritative, high-level, mathematically sound tactical advice in ${I18N.currentLang === 'pt' ? 'Portuguese' : 'English'}. Explain exact formula mechanics, stat priorities, marginal gains, and combat matchups clearly and concisely. Clearly differentiate between raw Combat Power (CP), per-hit damage, sustained estimated DPS, and actual measured rotation DPS. Never hallucinate unavailable data or invent unverified game mechanics; explicitly state which stats or rotation logs are required when data is incomplete. Calculator data:\n${currentAiContext()}\nOnline guide:\n${guide || 'none'}` },
      ...conversationHistory.slice(-6),
      { role: 'user', content: question },
    ] });
    target.textContent = result.text;
    conversationHistory.push({ role: 'user', content: question }, { role: 'assistant', content: result.text });
    if (conversationHistory.length > 8) conversationHistory.splice(0, conversationHistory.length - 8);
  } catch (error) {
    target.textContent = I18N.format('aiUnavailable', { error: error.message });
  } finally {
    aiSend.disabled = false;
  }
  scrollChatToBottom();
}

function submitUserQuestion() {
  const question = aiQuestion.value.trim();
  if (!question || aiSend.disabled) return;
  aiQuestion.value = '';
  appendChatMessage('user', I18N.t('aiYou'), question);
  const target = appendChatMessage('assistant', 'AionLabs', I18N.t('aiGenerating'));
  void dispatchAiRequest(question, target);
}

function updateApplyButton() {
  const suggestion = recommendedUpgrade();
  aiApply.hidden = !suggestion;
  if (suggestion) aiApply.textContent = I18N.format('aiApplyDynamic', { label: suggestion.label, gain: suggestion.gain });
}

aiApply.addEventListener('click', () => {
  const suggestion = recommendedUpgrade();
  if (!suggestion) return;
  const value = parseNumericInput(suggestion.input.value, 0);
  if (value === null) return;
  suggestion.input.value = String(value + 1);
  suggestion.input.dispatchEvent(new Event('input', { bubbles: true }));
});
aiSend?.addEventListener('click', submitUserQuestion);
aiQuestion.addEventListener('keydown', event => {
  if (event.key === 'Enter') { event.preventDefault(); submitUserQuestion(); }
});
document.querySelector('#ai-configure').addEventListener('click', async () => {
  const dialog = document.querySelector('#ai-settings');
  dialog.showModal();
  const status = document.querySelector('#ai-key-status');
  const modelSelect = document.querySelector('#ai-model');
  status.textContent = I18N.t('aiCheckingSettings');
  try {
    const [configured, models] = await Promise.all([hostRequest('ai_key_status'), hostRequest('ai_models')]);
    status.textContent = I18N.t(configured.configured ? 'aiKeyConfigured' : 'aiKeyMissing');
    const selected = localStorage.getItem('aionLabsModel') || 'aion-labs/aion-3.5-mini';
    modelSelect.replaceChildren(...models.map(id => {
      const option = document.createElement('option');
      option.value = id;
      option.textContent = id;
      return option;
    }));
    modelSelect.value = models.includes(selected) ? selected : models[0] || '';
    modelSelect.disabled = models.length === 0;
  } catch (error) {
    status.textContent = I18N.format('aiUnavailable', { error: error.message });
  }
});
document.querySelector('#ai-save-key').addEventListener('click', async () => {
  const field = document.querySelector('#ai-api-key');
  const status = document.querySelector('#ai-key-status');
  try {
    await hostRequest('ai_key_save', { key: field.value });
    field.value = '';
    status.textContent = I18N.t('aiKeyConfigured');
  } catch (error) { status.textContent = error.message; }
});
document.querySelector('#ai-remove-key').addEventListener('click', async () => {
  const status = document.querySelector('#ai-key-status');
  try {
    await hostRequest('ai_key_delete');
    status.textContent = I18N.t('aiKeyMissing');
  } catch (error) { status.textContent = error.message; }
});
document.querySelector('#ai-settings').addEventListener('close', () => {
  document.querySelector('#ai-api-key').value = '';
});
document.querySelector('#ai-model').addEventListener('change', event => localStorage.setItem('aionLabsModel', event.target.value));
for (const selector of ['#attacker-name', '#attacker-class', '#target-name', '#target-class', '#target-mode', '#creature-type-header', '#critical', '#position']) {
  document.querySelector(selector)?.addEventListener('input', updateApplyButton);
  document.querySelector(selector)?.addEventListener('change', updateApplyButton);
}
I18N.onLanguageChange(updateApplyButton);
updateApplyButton();
