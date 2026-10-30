// Markdown subset for assistant answers. Output is a plain node tree; rendering builds DOM nodes
// with createElement/createTextNode only, so model text is never parsed as HTML.
const TABLE_SEPARATOR = /^\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?$/;
const BULLET = /^[-*+]\s+(.*)$/;
const ORDERED = /^\d+[.)]\s+(.*)$/;
const INLINE = /`([^`]+)`|\*\*([^*]+)\*\*|\*([^*\s][^*]*)\*|\[([^\]]+)\]\(([^)\s]+)\)/g;

function parseInline(text) {
  const nodes = [];
  const pattern = new RegExp(INLINE.source, 'g');
  let last = 0;
  let match;
  while ((match = pattern.exec(text))) {
    if (match.index > last) nodes.push({ text: text.slice(last, match.index) });
    if (match[1] !== undefined) nodes.push({ tag: 'code', children: [{ text: match[1] }] });
    else if (match[2] !== undefined) nodes.push({ tag: 'strong', children: parseInline(match[2]) });
    else if (match[3] !== undefined) nodes.push({ tag: 'em', children: parseInline(match[3]) });
    else nodes.push({ text: `${match[4]} (${match[5]})` });
    last = pattern.lastIndex;
  }
  if (last < text.length) nodes.push({ text: text.slice(last) });
  return nodes;
}

function splitRow(line) {
  return line.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map(cell => cell.trim());
}

function isTableStart(lines, index) {
  return lines[index].includes('|') && index + 1 < lines.length && TABLE_SEPARATOR.test(lines[index + 1].trim());
}

function startsBlock(lines, index) {
  const trimmed = lines[index].trim();
  return /^```/.test(trimmed) || /^#{1,6}\s/.test(trimmed) || /^(-{3,}|\*{3,}|_{3,})$/.test(trimmed)
    || BULLET.test(trimmed) || ORDERED.test(trimmed) || trimmed.startsWith('>') || isTableStart(lines, index);
}

function parseBlocks(lines) {
  const blocks = [];
  let i = 0;
  while (i < lines.length) {
    const trimmed = lines[i].trim();
    if (!trimmed) {
      i++;
    } else if (trimmed.startsWith('```')) {
      const body = [];
      i++;
      while (i < lines.length && !lines[i].trim().startsWith('```')) body.push(lines[i++]);
      i++;
      blocks.push({ tag: 'pre', children: [{ tag: 'code', children: [{ text: body.join('\n') }] }] });
    } else if (/^#{1,6}\s/.test(trimmed)) {
      const [, hashes, title] = /^(#{1,6})\s+(.*)$/.exec(trimmed);
      blocks.push({ tag: `h${Math.min(hashes.length + 2, 6)}`, children: parseInline(title) });
      i++;
    } else if (/^(-{3,}|\*{3,}|_{3,})$/.test(trimmed)) {
      blocks.push({ tag: 'hr' });
      i++;
    } else if (isTableStart(lines, i)) {
      const rows = [{ tag: 'tr', children: splitRow(lines[i]).map(cell => ({ tag: 'th', children: parseInline(cell) })) }];
      i += 2;
      while (i < lines.length && lines[i].trim() && lines[i].includes('|')) {
        rows.push({ tag: 'tr', children: splitRow(lines[i]).map(cell => ({ tag: 'td', children: parseInline(cell) })) });
        i++;
      }
      blocks.push({ tag: 'table', children: rows });
    } else if (BULLET.test(trimmed) || ORDERED.test(trimmed)) {
      const pattern = BULLET.test(trimmed) ? BULLET : ORDERED;
      const items = [];
      while (i < lines.length && pattern.test(lines[i].trim())) {
        items.push({ tag: 'li', children: parseInline(pattern.exec(lines[i].trim())[1]) });
        i++;
      }
      blocks.push({ tag: pattern === BULLET ? 'ul' : 'ol', children: items });
    } else if (trimmed.startsWith('>')) {
      const quoted = [];
      while (i < lines.length && lines[i].trim().startsWith('>')) quoted.push(lines[i++].trim().replace(/^>\s?/, ''));
      blocks.push({ tag: 'blockquote', children: parseBlocks(quoted) });
    } else {
      const paragraph = [trimmed];
      i++;
      while (i < lines.length && lines[i].trim() && !startsBlock(lines, i)) paragraph.push(lines[i++].trim());
      blocks.push({ tag: 'p', children: parseInline(paragraph.join(' ')) });
    }
  }
  return blocks;
}

function parseMarkdown(source) {
  return parseBlocks(String(source ?? '').replace(/\r\n?/g, '\n').split('\n'));
}

function toDom(node, doc) {
  if (node.text !== undefined) return doc.createTextNode(node.text);
  const element = doc.createElement(node.tag);
  for (const child of node.children || []) element.append(toDom(child, doc));
  return element;
}

function renderMarkdown(source, doc = document) {
  const fragment = doc.createDocumentFragment();
  for (const node of parseMarkdown(source)) fragment.append(toDom(node, doc));
  return fragment;
}

if (typeof module !== 'undefined') module.exports = { parseMarkdown, renderMarkdown };
