const { test } = require('node:test');
const assert = require('node:assert/strict');
const { MAP_METADATA, TIMER_SCHEDULES, formatDuration, nextScheduledEvent } = require('./game-companion');

test('map metadata defines non-empty bounds and tiles for starter and world maps', () => {
  assert.equal(MAP_METADATA.length, 7);
  for (const map of MAP_METADATA) {
    assert.ok(map.width > 0);
    assert.ok(map.height > 0);
    assert.ok(map.tiles[0] > 0);
    assert.ok(map.tiles[1] > 0);
  }
});

test('duration formats hours, minutes and seconds cleanly', () => {
  assert.equal(formatDuration(0), '00:00');
  assert.equal(formatDuration(45), '00:45');
  assert.equal(formatDuration(3665), '1h 01m');
});

test('computes next hourly and interval-based recurring event occurrences', () => {
  const fixed = new Date('2026-10-30T10:15:00Z');
  const hourly = nextScheduledEvent(TIMER_SCHEDULES.shugoFesta, fixed);
  assert.equal(hourly.next.getMinutes(), 0);
  assert.equal(hourly.next.getHours(), fixed.getHours() + 1);

  const cyclical = nextScheduledEvent(TIMER_SCHEDULES.kairah, fixed);

  assert.ok(cyclical.next > fixed);
  assert.equal(cyclical.next.getMinutes(), 0);
});
test('class builds define both pve and pvp recommendations', () => {
  const { CLASS_BUILDS } = require('./game-companion');
  assert.ok(CLASS_BUILDS.Gladiator.pve.weapon);
  assert.ok(CLASS_BUILDS.Gladiator.pvp.weapon);
  assert.ok(CLASS_BUILDS.Gladiator.pve.stigmas.length >= 4);
  assert.ok(CLASS_BUILDS.Gladiator.pvp.stigmas.length >= 4);
});
