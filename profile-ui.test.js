const { test } = require('node:test');
const assert = require('node:assert/strict');
const { characterRankingRows, skillState } = require('./profile-ui');

test('formats only complete overall, class and server ranking rows', () => {
  assert.deepEqual(characterRankingRows({
    overall_rank: 320, overall_total: 5899, overall_percentile: 5.41,
    job_rank: 66, job_total: 866, job_percentile: 7.51,
    server_rank: 2, server_total: 55, server_percentile: 1.82
  }), [
    { scope: 'rankOverall', rank: 320, total: 5899, percent: '5.41' },
    { scope: 'rankClass', rank: 66, total: 866, percent: '7.51' },
    { scope: 'rankServer', rank: 2, total: 55, percent: '1.82' }
  ]);
  assert.deepEqual(characterRankingRows({ overall_rank: 1, overall_total: 10 }), []);
  assert.deepEqual(characterRankingRows(null), []);
});

test('separates equipped, learned and locked skills without treating zero-level skills as learned', () => {
  assert.equal(skillState({ acquired: 1, equip: 1, skillLevel: 17 }), 'equipped');
  assert.equal(skillState({ acquired: 1, equip: 0, skillLevel: 12 }), 'skillAcquired');
  assert.equal(skillState({ acquired: 0, equip: 0, skillLevel: 0 }), 'skillNotAcquired');
  assert.equal(skillState({ acquired: 1, equip: 1, skillLevel: 0 }), 'skillNotAcquired');
});
