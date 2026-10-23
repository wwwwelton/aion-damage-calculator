// Enriches boss-data.js with per-monster details published by aion2data.com
// (portrait, spawn points, basic attack and skill list). HP and defenses are
// server-side and never appear on that site. Run occasionally to refresh:
//   node scrape-boss-details.mjs
// Requests are sequential with a delay; existing details are kept when a page
// cannot be fetched.
import { readFile, writeFile } from 'node:fs/promises';

const source = await readFile('boss-data.js', 'utf8');
const bossCatalog = JSON.parse(source.slice(source.indexOf('['), source.lastIndexOf(']') + 1));
const text = html => html.replace(/<!--[\s\S]*?-->/g, '').replace(/<[^>]+>/g, '')
  .replace(/&amp;/g, '&').replace(/&#x27;|&#39;/g, '\'').replace(/&quot;/g, '"').trim();

function parseDetails(html) {
  const details = {};
  const portrait = html.match(/<meta property="og:image" content="([^"]+)"/)?.[1];
  if (portrait?.startsWith('https://aion2data.com/game/monsters/')) details.portrait = portrait.slice('https://aion2data.com'.length);
  const spawn = html.match(/(\d+) spawn point/);
  if (spawn) details.spawnPoints = Number(spawn[1]);
  const basic = html.match(/Basic attack:([\s\S]*?)<\/li>/);
  if (basic) {
    const parts = text(basic[1]).replace(/\.$/, '').split(',').map(part => part.trim());
    const reach = parts.find(part => part.startsWith('reach'))?.match(/[\d.]+/)?.[0];
    const interval = parts.find(part => part.startsWith('every'))?.match(/[\d.]+/)?.[0];
    details.attack = { type: parts[0], reach: reach ? Number(reach) : null, interval: interval ? Number(interval) : null };
  }
  const skills = [...html.matchAll(/<table[\s\S]*?<\/table>/g)].map(match => match[0])
    .find(table => /<th[^>]*>Skill<\/th>/.test(table));
  if (skills) {
    details.skills = [...skills.matchAll(/<tr[^>]*>([\s\S]*?)<\/tr>/g)]
      .map(row => [...row[1].matchAll(/<td[^>]*>([\s\S]*?)<\/td>/g)].map(cell => text(cell[1])))
      .filter(cells => cells.length >= 5)
      .map(([name, damage, cast, cooldown, range]) => ({ name, damage, cast, cooldown, range }));
  }
  return details;
}

let fetched = 0;
for (const boss of bossCatalog) {
  const url = `https://aion2data.com/monsters/${boss.id}-${boss.slug}`;
  try {
    const response = await fetch(url, { headers: { 'User-Agent': 'Mozilla/5.0 (AION2 damage calculator data refresh)' } });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    Object.assign(boss, parseDetails(await response.text()));
    fetched++;
  } catch (error) {
    console.error(`${boss.id} ${boss.name}: ${error.message}`);
  }
  await new Promise(resolve => setTimeout(resolve, 350));
}
await writeFile('boss-data.js', `const bossCatalog = ${JSON.stringify(bossCatalog)};\n${source.slice(source.lastIndexOf(']') + 2)}`);
console.log(`Updated ${fetched}/${bossCatalog.length} bosses.`);
