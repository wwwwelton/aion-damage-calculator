const classIcons = new Set(['gladiator', 'templar', 'assassin', 'ranger', 'sorcerer', 'spiritmaster', 'cleric', 'chanter']);

function iconElement(kind, label = '') {
  const normalized = String(label).trim().toLowerCase().replace(/\s+/g, '');
  if (kind === 'class' && classIcons.has(normalized)) {
    const image = document.createElement('img');
    image.className = 'icon icon-class';
    image.src = `assets/classes/${normalized}.webp`;
    image.alt = '';
    image.setAttribute('aria-hidden', 'true');
    return image;
  }
  const symbols = new Set(['boss', 'mob', 'skill', 'attack', 'defense', 'health', 'chance', 'damage', 'stat']);
  const symbol = symbols.has(kind) ? kind : 'stat';
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.classList.add('icon', `icon-${symbol}`);
  svg.setAttribute('aria-hidden', 'true');
  svg.setAttribute('viewBox', '0 0 24 24');
  const use = document.createElementNS('http://www.w3.org/2000/svg', 'use');
  use.setAttribute('href', `#icon-${symbol}`);
  svg.append(use);
  return svg;
}

function profileImage(url, kind = 'stat', label = '') {
  let parsed;
  try { parsed = new URL(url); } catch { return iconElement(kind, label); }
  const gameAsset = parsed.hostname === 'assets.playnccdn.com'
    && parsed.pathname.startsWith('/static-aion2-gamedata/resources/');
  const portrait = parsed.hostname === 'profileimg.plaync.com'
    && parsed.pathname.startsWith('/game_profile_images/');
  const monster = parsed.hostname === 'aion2data.com' && parsed.pathname.startsWith('/game/monsters/');
  if (parsed.protocol !== 'https:' || (!gameAsset && !portrait && !monster)) return iconElement(kind, label);
  const image = document.createElement('img');
  image.className = 'icon remote-icon';
  image.alt = '';
  image.loading = 'lazy';
  image.referrerPolicy = 'no-referrer';
  image.addEventListener('error', () => {
    const fallback = iconElement(kind, label);
    if (image.classList.contains('dossier-portrait')) fallback.classList.add('dossier-portrait');
    image.replaceWith(fallback);
  }, { once: true });
  image.src = parsed.href;
  return image;
}

function attributeIcon(name) {
  if (/\b(hp|health|recovery|regeneration|constitution)\b/i.test(name)) return 'health';
  if (/\b(defense|tolerance|resist|block|evasion|endurance)\b/i.test(name)) return 'defense';
  if (/\b(chance|critical hit|accuracy|precision|perfect|multi-hit)\b/i.test(name)) return 'chance';
  if (/\b(damage|boost|coefficient|specialty)\b/i.test(name)) return 'damage';
  if (/\b(attack|penetration|power shard|might)\b/i.test(name)) return 'attack';
  return 'stat';
}

function setDisplayIcon(container, kind, label = '') {
  container.replaceChildren(iconElement(kind, label));
  container.hidden = kind === 'class' && !classIcons.has(String(label).trim().toLowerCase().replace(/\s+/g, ''));
}

if (typeof module !== 'undefined') module.exports = { attributeIcon };
