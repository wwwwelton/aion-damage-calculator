const aiQuestion = document.querySelector('#ai-question');
const aiAnswer = document.querySelector('#ai-answer');
const aiSource = document.querySelector('#ai-source');
const aiApply = document.querySelector('#ai-apply');
const aiChatDialog = document.querySelector('#ai-chat-dialog');
const aiSend = document.querySelector('#ai-send');

let aiWorker;
let aiStartup;
let aiTimer;
let aiRevision = 0;
let currentPendingTarget;
const conversationHistory = [];
let researchRequest = -1;
const researchPending = new Map();
// Guide cache with session TTL - auto-clear on session end to avoid permanent error cache
const guideCache = new Map();
let guideCacheSessionId = null;
function initGuideCache() {
  const sessionId = localStorage.getItem('aiSessionId') || crypto.randomUUID();
  if (guideCacheSessionId !== sessionId) {
    guideCache.clear();
    guideCacheSessionId = sessionId;
  }
}
if (typeof window !== 'undefined') {
  initGuideCache();
}
function scrollChatToBottom() {
  if (aiChatDialog) {
    aiChatDialog.scrollTo({ top: aiChatDialog.scrollHeight, behavior: 'smooth' });
  }
}

function appendChatMessage(role, author, text) {
  if (!aiChatDialog) return null;
  const msg = document.createElement('div');
  msg.className = `ai-msg ai-msg-${role}`;
  const authorSpan = document.createElement('span');
  authorSpan.className = 'ai-msg-author';
  authorSpan.textContent = author;
  const bodyDiv = document.createElement('div');
  bodyDiv.className = 'ai-msg-body';
  bodyDiv.textContent = text;
  msg.appendChild(authorSpan);
  msg.appendChild(bodyDiv);
  aiChatDialog.appendChild(msg);
  scrollChatToBottom();
  return bodyDiv;
}

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
  const tMode = document.querySelector('#target-mode').value;
  const cType = document.querySelector('#creature-type-header').value || 'Unknown';
  const targetClass = tMode === 'player' ? document.querySelector('#target-class').value || 'Unspecified class' : '';
  const characterStats = [];
  const enemyStats = [];
  for (const input of document.querySelectorAll('#calculator input[name]')) {
    if (input.closest('[hidden]') || (tMode === 'player' && input.closest('.pve-only, .boss-only'))
        || (tMode === 'monster' && input.closest('.boss-only, .player-only'))
        || (tMode === 'boss' && input.closest('.player-only'))) continue;
    if (!input.value || input.value === '0' || input.value === input.defaultValue) continue;
    const stat = `${input.name}: ${input.value}`;
    (input.closest('label.enemy') ? enemyStats : characterStats).push(stat);
  }
  const hit = document.querySelector('#result').textContent || '0';
  const dps = document.querySelector('#estimated-dps').textContent || '0';
  const crit = document.querySelector('#critical').value;
  const pos = document.querySelector('#position').value;
  const topUpgrades = [...document.querySelectorAll('#upgrade-list li')]
    .slice(0, 3).map(li => li.textContent.trim()).join(' | ') || 'None';

  return `Personagem: ${attacker}; classe: ${aClass}\n`
    + `Inimigo: ${target}; modo: ${tMode}; tipo: ${cType}${targetClass ? `; classe: ${targetClass}` : ''}\n`
    + `Atributos do personagem: ${characterStats.join(', ')}\n`
    + `Atributos do inimigo: ${enemyStats.join(', ')}\n`
    + `Golpe calculado: ${hit}; DPS estimado: ${dps}; crítico: ${crit}; posição: ${pos}\n`
    + `Melhorias calculadas (+1%): ${topUpgrades}`;
}

function requestGuide(topic) {
  const id = researchRequest--;
  return new Promise((resolve, reject) => {
    researchPending.set(id, { resolve, reject });
    window.aion2Host.post({ id, kind: 'research', topic });
  });
}

if (window.aion2Host) {
  window.aion2Host.onMessage(({ data }) => {
    const pending = researchPending.get(data?.id);
    if (!pending) return;
    researchPending.delete(data.id);
    if (data.error) {
      // Clear failed guide from cache to allow retry
      const topic = guideTopic(data.topic || '');
      guideCache.delete(topic);
      pending.reject(new Error(data.error));
    } else {
      pending.resolve(data.data);
    }
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

const pendingPattern = () => new RegExp(`^(${['aiGenerating', 'aiGeneratingLocal', 'aiReading', 'aiLoading']
  .map(key => I18N.t(key).split('{')[0].replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|')})`);

async function ensureAiWorker() {
  if (aiStartup) return aiStartup;
  aiStartup = (async () => {
    aiWorker = new Worker('ai-worker.bundle.js', { type: 'module' });
    aiWorker.onmessage = ({ data }) => {
      if (data.id !== aiRevision) return;
      const target = currentPendingTarget || aiAnswer;
      if (data.state === 'loading') target.textContent = I18N.t('aiLoading');
      if (data.state === 'downloading') target.textContent = I18N.format('aiReading', { progress: data.progress });
      if (data.state === 'generating') target.textContent = I18N.t('aiGenerating');
      if (data.state === 'done') {
        target.textContent = data.text || I18N.t('aiNoSuggestion');
        if (target !== aiAnswer) {
          conversationHistory.push({ role: 'assistant', content: target.textContent });
          if (conversationHistory.length > 6) conversationHistory.splice(0, conversationHistory.length - 6);
        }
        scrollChatToBottom();
      }
      if (data.state === 'error') {
        target.textContent = I18N.format('aiUnavailable', { error: data.error });
        scrollChatToBottom();
      }
    };
    aiWorker.onerror = () => {
      const target = currentPendingTarget || aiAnswer;
      target.textContent = I18N.t('aiFailedStart');
      scrollChatToBottom();
    };
  })();
  return aiStartup;
}

async function dispatchAiRequest(question, targetElem) {
  const id = ++aiRevision;
  if (currentPendingTarget && currentPendingTarget !== aiAnswer && currentPendingTarget !== targetElem
      && pendingPattern().test(currentPendingTarget.textContent)) {
    currentPendingTarget.textContent = I18N.t('aiSuperseded');
  }
  currentPendingTarget = targetElem;
  const context = currentAiContext();
  let research = '';
  aiSource.textContent = '';
  if (window.aion2Host) {
    const topic = guideTopic(question);
    if (!guideCache.has(topic)) guideCache.set(topic, requestGuide(topic));
    try {
      const guide = await guideCache.get(topic);
      if (id !== aiRevision) return;
      research = guide.excerpt.slice(0, 280);
      aiSource.textContent = I18N.format('aiGuide', { url: guide.url });
    } catch (error) {
      if (id !== aiRevision) return;
      aiSource.textContent = I18N.format('aiGuideUnavailable', { error: error.message });
    }
  }
  if (id !== aiRevision) return;
  try {
    await ensureAiWorker();
    if (id !== aiRevision) return;
    targetElem.textContent = I18N.t('aiGeneratingLocal');
    scrollChatToBottom();
    lastAdviceContext = `${I18N.currentLang}\n${context}`;
    aiWorker.postMessage({ id, question, context, research, history: conversationHistory, language: I18N.currentLang });
  } catch (error) {
    if (id === aiRevision) {
      targetElem.textContent = `${error.message} ${I18N.t('aiUseComparison')}`;
      scrollChatToBottom();
    }
  }
}

function submitUserQuestion() {
  const question = aiQuestion.value.trim();
  if (!question) return;
  clearTimeout(aiTimer);
  aiQuestion.value = '';

  appendChatMessage('user', I18N.t('aiYou'), question);
  conversationHistory.push({ role: 'user', content: question });
  if (conversationHistory.length > 6) conversationHistory.splice(0, conversationHistory.length - 6);

  const assistantTarget = appendChatMessage('assistant', 'Avatarnoia Flow', I18N.t('aiGenerating'));
  scrollChatToBottom();

  void dispatchAiRequest(question, assistantTarget);
}

// Each generation pins a CPU core for tens of seconds, so automatic advice
// waits for the user to pause and only runs when the build context changed.
let lastAdviceContext = '';
function scheduleAiTips() {
  clearTimeout(aiTimer);
  const suggestion = recommendedUpgrade();
  aiApply.hidden = !suggestion;
  if (suggestion) aiApply.textContent = I18N.format('aiApplyDynamic', { label: suggestion.label, gain: suggestion.gain });
  aiTimer = setTimeout(() => {
    if (`${I18N.currentLang}\n${currentAiContext()}` === lastAdviceContext) return;
    void dispatchAiRequest('Como melhorar minha build ou DPS', aiAnswer);
  }, 2000);
}

aiApply.addEventListener('click', () => {
  const suggestion = recommendedUpgrade();
  if (!suggestion) return;
  const value = parseNumericInput(suggestion.input.value, 0);
  if (value === null) return;
  suggestion.input.value = String(value + 1);
  suggestion.input.dispatchEvent(new Event('input', { bubbles: true }));
});

if (aiSend) {
  aiSend.addEventListener('click', submitUserQuestion);
}

aiQuestion.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') {
    e.preventDefault();
    submitUserQuestion();
  }
});

for (const selector of ['#attacker-name', '#attacker-class', '#target-name', '#target-class', '#target-mode', '#creature-type-header', '#critical', '#position']) {
  const elem = document.querySelector(selector);
  if (elem) {
    elem.addEventListener('input', scheduleAiTips);
    elem.addEventListener('change', scheduleAiTips);
  }
}

I18N.onLanguageChange(() => {
  const suggestion = recommendedUpgrade();
  if (suggestion) aiApply.textContent = I18N.format('aiApplyDynamic', { label: suggestion.label, gain: suggestion.gain });
});
