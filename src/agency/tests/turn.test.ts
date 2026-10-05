// Run by Claude Code's own plugin test engine, not Node or a harness substitute.
import {test, expect} from 'claude-code/testing';

test('a fresh dispatch replaces a restored early-cancel draft', async ($, on) => {
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', text: 'NEXT_PROMPT'})}));
  on('prompt.edit', (_, e) => ({
    text: e.text.slice(0, e.start) + e.inputText + e.text.slice(e.end),
    cursor: e.start + e.inputText.length,
  }));
  const old = 'CANCELLED_PROMPT\nwith another line';
  const result = await $.prompt.edit({origin: {kind: 'composer'}, text: old,
    cursor: old.length, start: old.length, end: old.length, inputText: 'NEXT_PROMPT'});
  expect(result.text).toBe('NEXT_PROMPT');
});

test('successive paste chunks from one dispatch are retained', async ($, on) => {
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', text: 'FIRST_SECOND'})}));
  on('prompt.edit', (_, e) => ({
    text: e.text.slice(0, e.start) + e.inputText + e.text.slice(e.end),
    cursor: e.start + e.inputText.length,
  }));
  const first = await $.prompt.edit({origin: {kind: 'composer'}, text: '',
    cursor: 0, start: 0, end: 0, inputText: 'FIRST_'});
  const second = await $.prompt.edit({origin: {kind: 'composer'}, text: first.text,
    cursor: first.cursor, start: first.cursor, end: first.cursor, inputText: 'SECOND'});
  expect(second.text).toBe('FIRST_SECOND');
});

test('a mismatched submission is dropped before a model turn', async ($, on) => {
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', text: 'NEXT_PROMPT'})}));
  on('fs.write', () => ({value: undefined}));
  on('session.id', () => ({value: 'native-test'}));
  on('prompt.submit', (_, e) => ({text: e.text}));
  const result = await $.prompt.submit({origin: {kind: 'composer'}, text: 'OLD_PROMPTNEXT_PROMPT', wait: false});
  expect(result.drop).toBe('Agency prompt does not match this dispatch');
});
