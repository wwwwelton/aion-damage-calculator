// Non-commercial interactive map dataset reference, attribution and regional polygon boundaries.
// Source: https://github.com/aion2-interactive-map/aion2-interactive-map
// License: Creative Commons Attribution-NonCommercial 4.0 International (CC BY-NC 4.0)
// Copyright (c) 2025 Yihao Liu (tc-imba). Non-commercial usage only.

const MAP_METADATA = [
  { id: 'World_L_Starter', name: 'Poeta', faction: 'Elyos', width: 4096, height: 4096, tiles: [4, 4] },
  { id: 'World_L_A', name: 'Verteron / Eltnen', faction: 'Elyos', width: 8192, height: 8192, tiles: [8, 8] },
  { id: 'World_D_Starter', name: 'Ishalgen', faction: 'Asmodian', width: 5120, height: 5120, tiles: [5, 5] },
  { id: 'World_D_A', name: 'Altgard / Morheim', faction: 'Asmodian', width: 8192, height: 8192, tiles: [8, 8] },
  { id: 'Abyss_Reshanta_A', name: 'Reshanta (Lower)', faction: 'Abyss', width: 4096, height: 4096, tiles: [4, 4] },
  { id: 'Abyss_Reshanta_B', name: 'Reshanta (Middle)', faction: 'Abyss', width: 2048, height: 2048, tiles: [2, 2] },
  { id: 'Abyss_Reshanta_C', name: 'Reshanta (Upper)', faction: 'Abyss', width: 1024, height: 1024, tiles: [1, 1] },
];

const TIMER_SCHEDULES = {
  rift: { intervalHours: 3, offsetHours: 2, durationMinutes: 60, portalMinutes: 10 },
  shugoFesta: { minute: 0, durationMinutes: 10 },
  dimensionalInvasion: { minute: 30, durationMinutes: 15 },
  battleground: { windows: [[11, 0, 120], [19, 0, 120]] },
  artifactSiege: { days: [1, 4, 6], hour: 21, minute: 0, durationMinutes: 30 },
  siegeBosses: { days: [1, 4, 6], hour: 21, minute: 30, durationMinutes: 30 },
  nahma: { days: [5, 0], hour: 21, minute: 0, durationMinutes: 30 },
  kairah: { intervalHours: 3, offsetHours: 1, durationMinutes: 30 },
};

function formatDuration(seconds) {
  if (seconds <= 0) return '00:00';
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rem = Math.floor(seconds % 60);
  if (hours > 0) return `${hours}h ${String(minutes).padStart(2, '0')}m`;
  return `${String(minutes).padStart(2, '0')}:${String(rem).padStart(2, '0')}`;
}

function nextScheduledEvent(schedule, now = new Date()) {
  const current = new Date(now);
  if (schedule.minute !== undefined) {
    const next = new Date(current);
    next.setSeconds(0, 0);
    if (next.getMinutes() >= schedule.minute) next.setHours(next.getHours() + 1);
    next.setMinutes(schedule.minute);
    return { next, durationMinutes: schedule.durationMinutes };
  }
  if (schedule.intervalHours) {
    const epochHours = (current.getTime() / 3600000);
    const cycle = schedule.intervalHours;
    const baseHour = schedule.offsetHours % cycle;
    const currentHour = current.getUTCHours();
    // Local civil date alignment
    const next = new Date(current);
    next.setMinutes(0, 0, 0);
    let step = 0;
    while (step < 48) {
      if ((next.getHours() % cycle) === (schedule.offsetHours % cycle) && next > current) break;
      next.setHours(next.getHours() + 1);
      step++;
    }

    return { next, durationMinutes: schedule.durationMinutes };
  }
  return null;
}
if (typeof document !== 'undefined') {
  const select = document.querySelector('#map-select');
  const summary = document.querySelector('#map-summary');
  const timerCards = document.querySelector('#timer-cards');
  if (select) {
    select.replaceChildren(...MAP_METADATA.map(map => new Option(`${map.name} (${map.faction})`, map.id)));
    function updateMap() {
      const chosen = MAP_METADATA.find(map => map.id === select.value) || MAP_METADATA[0];
      summary.textContent = `${chosen.name} · ${chosen.faction} · ${chosen.width}x${chosen.height} px · ${chosen.tiles[0]}x${chosen.tiles[1]} tiles`;
    }
    select.addEventListener('change', updateMap);
    updateMap();
  }
  if (timerCards) {
    function renderTimers() {
      const now = new Date();
      const items = [
        ['Fenda Espaço-Temporal', nextScheduledEvent(TIMER_SCHEDULES.rift, now)],
        ['Shugo Festa', nextScheduledEvent(TIMER_SCHEDULES.shugoFesta, now)],
        ['Invasão Dimensional', nextScheduledEvent(TIMER_SCHEDULES.dimensionalInvasion, now)],
        ['Vigilante Kairah', nextScheduledEvent(TIMER_SCHEDULES.kairah, now)],
      ];
      timerCards.replaceChildren(...items.map(([title, event]) => {
        const card = document.createElement('article');
        card.className = 'progression-card';
        const strong = document.createElement('strong');
        strong.textContent = title;
        const small = document.createElement('small');
        const diff = event ? Math.max(0, Math.floor((event.next.getTime() - now.getTime()) / 1000)) : 0;
        small.textContent = `Próximo: ${event ? event.next.toLocaleTimeString() : '—'} (em ${formatDuration(diff)})`;
        card.append(strong, small);
        return card;
      }));
    }
    renderTimers();
    setInterval(renderTimers, 10000);
  }
}
const CLASS_BUILDS = {
  Gladiator: { weapon: 'Montante do Lorde Dragão Sombrio Brilhante +15', stigmas: ['Postura de Assalto', 'Bênção de Zikel', 'Explosão de Ira', 'Perseverança', 'Bloqueio Concentrado', 'Arremesso de Lâmina'] },
  Templar: { weapon: 'Espada Longa do Lorde Dragão Sombrio Brilhante +15', stigmas: ['Bandeira do Campo de Batalha', 'Escudo da Perdição', 'Armadura Nobre', 'Provocação', 'Apreensão', 'Escudo da Proteção', 'Punição do Senhor Empíreo'] },
  Ranger: { weapon: 'Arco do Lorde Dragão Sombrio Brilhante +15', stigmas: ['Soberania de Vaizel', 'Arco da Bênção', 'Fogo de Supressão', 'Flecha de Grifo', 'Sopro da Mãe Natureza', 'Chute Surpresa'] },
  Assassin: { weapon: 'Adaga do Lorde Dragão Sombrio Brilhante +15', stigmas: ['Pacto da Celeridade', 'Clone Ilusório', 'Lançamento de Espada Sombria', 'Presa da Besta', 'Punhal de Triniel', 'Caminhada nas Sombras', 'Postura de Evasão'] },
  Sorcerer: { weapon: 'Grimório do Lorde Dragão Sombrio Brilhante +15', stigmas: ['Aprimoramento Elemental', 'Barreira de Fogo', 'Explosão Tardia', 'Tempestade Gélida', 'Escudo Protetivo de Aço', 'Armadura Ártica', 'Hibernação'] },
  Spiritmaster: { weapon: 'Orbe do Lorde Dragão Sombrio Brilhante +15', stigmas: ['Invocação: Espírito Ancestral', 'Ataque Conjunto: Corrosão', 'Aprimoramento: Bendição do Espírito', 'Bênção do Fogo Ardente', 'Comando: Substituto', 'Sucção'] },
  Cleric: { weapon: 'Maça do Lorde Dragão Sombrio Brilhante +15', stigmas: ['Punição da Terra', 'Oração da Amplificação', 'Luz da Proteção', 'Aura Nobre', 'Salvação', 'Ressurreição por Invocação', 'Soberania de Yustiel'] },
  Chanter: { weapon: 'Cajado do Lorde Dragão Sombrio Brilhante +15', stigmas: ['Mantra do Imbatível', 'Soberania do Vendaval', 'Bênção da Guarda', 'Fúria de Marchutan', 'Mantra da Disparada', 'Defesa Concentrada'] }
};
const LEADERBOARD_SNAPSHOT = [
  { rank: 1, name: 'Asulgard', className: 'Ranger', server: 'Siel (EU)', cp: 287070 },
  { rank: 2, name: 'dieyoung', className: 'Ranger', server: 'Israphel (EU)', cp: 286732 },
  { rank: 3, name: 'gigawhaleYT', className: 'Chanter', server: 'Israphel (EU)', cp: 286058 },
  { rank: 4, name: '白菜菜', className: 'Chanter', server: 'Israphel (EU)', cp: 284746 },
  { rank: 5, name: 'kbbb', className: 'Gladiator', server: 'Israphel (Asia)', cp: 283814 },
  { rank: 6, name: '小小时光', className: 'Ranger', server: 'Israphel (EU)', cp: 282417 },
  { rank: 7, name: 'DaBaiCai', className: 'Ranger', server: 'Israphel (EU)', cp: 282330 },
  { rank: 8, name: 'LeOz', className: 'Ranger', server: 'Siel (Asia)', cp: 281614 },
  { rank: 9, name: 'まつびさいき', className: 'Sorcerer', server: 'Zikel (Asia)', cp: 281476 },
  { rank: 10, name: 'ZZ', className: 'Cleric', server: 'Zikel (NA East)', cp: 280882 },
];
const DATABASE_ENTRIES = [
  { type: 'item', name: 'Montante do Lorde Dragão Sombrio Brilhante', category: 'Arma', detail: 'Espada de duas mãos · Nível 102 · Heroico' },
  { type: 'item', name: 'Asas do Guerreiro', category: 'Asas', detail: 'Grau Lendário · Nível de encantamento +0' },
  { type: 'skill', name: 'Postura de Assalto', category: 'Habilidade', detail: 'Estigma ativa · Gladiador · Aumento ofensivo' },
  { type: 'dungeon', name: 'Abyss Outland', category: 'Conteúdo', detail: 'Battle royale do Abismo · Partidas a cada 30 minutos' },
  { type: 'monster', name: 'Vigilante Kairah', category: 'Chefe', detail: 'Chefe de campo do Abismo Inferior · Ciclo de 3 horas' },
  { type: 'title', name: 'Treading on Black Wings', category: 'Título', detail: 'Título de ataque · Bônus de Ataque PvP' }
];



if (typeof document !== 'undefined') {
  const buildCards = document.querySelector('#build-cards');
  if (buildCards) {
    buildCards.replaceChildren(...Object.entries(CLASS_BUILDS).map(([className, build]) => {
      const card = document.createElement('article');
      card.className = 'progression-card';
      const strong = document.createElement('strong');
      strong.textContent = className;
      const weapon = document.createElement('small');
      weapon.textContent = `Arma meta: ${build.weapon}`;
      const stigmas = document.createElement('p');
      stigmas.className = 'reference-note';
      stigmas.textContent = `Estigmas recomendados: ${build.stigmas.join(', ')}`;
      card.append(strong, weapon, stigmas);
      return card;
    }));
  }
  const leaderboardList = document.querySelector('#leaderboard-list');
  const leaderboardSummary = document.querySelector('#leaderboard-summary');
  if (leaderboardList && leaderboardSummary) {
    const median = Math.round(LEADERBOARD_SNAPSHOT.reduce((sum, item) => sum + item.cp, 0) / LEADERBOARD_SNAPSHOT.length);
    leaderboardSummary.textContent = `Poder de combate mediano (Top 10): ${median.toLocaleString()} · Limite top 10: 280.882 CP`;
    leaderboardList.replaceChildren(...LEADERBOARD_SNAPSHOT.map(entry => {
      const li = document.createElement('li');
      li.textContent = `${entry.name} · ${entry.className} · ${entry.server} · ${entry.cp.toLocaleString()} CP`;
      return li;
    }));
  }
  const dbInput = document.querySelector('#db-search-input');
  const dbResults = document.querySelector('#db-search-results');
  if (dbInput && dbResults) {
    function renderDatabase() {
      const query = dbInput.value.trim().toLowerCase();
      const matched = DATABASE_ENTRIES.filter(entry => !query || entry.name.toLowerCase().includes(query) || entry.category.toLowerCase().includes(query));
      dbResults.replaceChildren(...matched.map(entry => {
        const card = document.createElement('article');
        card.className = 'progression-card';
        const strong = document.createElement('strong');
        strong.textContent = `${entry.name} (${entry.category})`;
        const small = document.createElement('small');
        small.textContent = entry.detail;
        card.append(strong, small);
        return card;
      }));
    }
    dbInput.addEventListener('input', renderDatabase);
    renderDatabase();
  }
}

if (typeof module !== 'undefined') {
  module.exports = { MAP_METADATA, TIMER_SCHEDULES, CLASS_BUILDS, LEADERBOARD_SNAPSHOT, DATABASE_ENTRIES, formatDuration, nextScheduledEvent };
}
