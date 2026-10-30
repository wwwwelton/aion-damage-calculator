const { test } = require('node:test');
const assert = require('node:assert/strict');
const { parseMarkdown } = require('./markdown');

const text = value => ({ text: value });

test('headings are shifted below the panel title and keep inline emphasis', () => {
  assert.deepEqual(parseMarkdown('# Report\n## Summary\n### Detail'), [
    { tag: 'h3', children: [text('Report')] },
    { tag: 'h4', children: [text('Summary')] },
    { tag: 'h5', children: [text('Detail')] },
  ]);
});

test('consecutive lines form one paragraph and unmatched asterisks stay literal', () => {
  assert.deepEqual(parseMarkdown('Hit is **high**\nwith 2 * 3 and *ok*'), [
    { tag: 'p', children: [
      text('Hit is '), { tag: 'strong', children: [text('high')] }, text(' with 2 * 3 and '),
      { tag: 'em', children: [text('ok')] },
    ] },
  ]);
});

test('pipe tables become header and body rows, and every row keeps its cells', () => {
  const [table] = parseMarkdown('| Atributo | Você |\n| --- | --- |\n| Dano | `100%` |');
  assert.equal(table.tag, 'table');
  assert.deepEqual(table.children[0].children.map(cell => cell.tag), ['th', 'th']);
  assert.deepEqual(table.children[1].children[1], { tag: 'td', children: [{ tag: 'code', children: [text('100%')] }] });
});

test('bullet and numbered lists group consecutive items', () => {
  assert.deepEqual(parseMarkdown('- one\n- two\n1. first'), [
    { tag: 'ul', children: [{ tag: 'li', children: [text('one')] }, { tag: 'li', children: [text('two')] }] },
    { tag: 'ol', children: [{ tag: 'li', children: [text('first')] }] },
  ]);
});

test('fenced code keeps its content verbatim without inline parsing', () => {
  assert.deepEqual(parseMarkdown('```\n**not bold**\n```'), [
    { tag: 'pre', children: [{ tag: 'code', children: [text('**not bold**')] }] },
  ]);
});

test('model text containing HTML never produces elements', () => {
  const [paragraph] = parseMarkdown('<img src=x onerror=alert(1)> and <b>bold</b>');
  assert.deepEqual(paragraph.children, [text('<img src=x onerror=alert(1)> and <b>bold</b>')]);
});

test('links keep their label and visible URL as plain text', () => {
  assert.deepEqual(parseMarkdown('[guia](https://example.com)'), [
    { tag: 'p', children: [text('guia (https://example.com)')] },
  ]);
});
