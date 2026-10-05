// Run by Claude Code's own plugin test engine, not Node or a harness substitute.
import {test, expect} from 'claude-code/testing';

const marker = 'HCTL2_DISPATCH_d1';

test('a fresh dispatch replaces a restored early-cancel draft', async ($, on) => {
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', marker, text: 'NEXT_PROMPT\n'})}));
  on('prompt.edit', (_, e) => ({
    text: e.text.slice(0, e.start) + e.inputText + e.text.slice(e.end),
    cursor: e.start + e.inputText.length,
  }));
  const old = 'CANCELLED_PROMPT\nwith another line';
  const result = await $.prompt.edit({origin: {kind: 'composer'}, text: old,
    cursor: old.length, start: old.length, end: old.length, inputText: marker});
  expect(result.text).toBe(marker);
});

test('successive paste chunks from one dispatch are retained', async ($, on) => {
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', marker, text: 'TASK\n'})}));
  on('prompt.edit', (_, e) => ({
    text: e.text.slice(0, e.start) + e.inputText + e.text.slice(e.end),
    cursor: e.start + e.inputText.length,
  }));
  const first = await $.prompt.edit({origin: {kind: 'composer'}, text: '',
    cursor: 0, start: 0, end: 0, inputText: 'HCTL2_'});
  const second = await $.prompt.edit({origin: {kind: 'composer'}, text: first.text,
    cursor: first.cursor, start: first.cursor, end: first.cursor, inputText: 'DISPATCH_d1'});
  expect(second.text).toBe(marker);
});

test('a marker injects the entire frozen task without trimming or attachments', async ($, on) => {
  const text = '  Summarize these notes.\r\n' + '\tNotes with trailing spaces.  \r\n'.repeat(22) + '\n';
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', marker, text})}));
  on('fs.write', () => ({value: undefined}));
  on('session.id', () => ({value: 'native-test'}));
  on('prompt.submit', (_, e) => ({text: e.text}));
  const result = await $.prompt.submit({origin: {kind: 'composer'}, text: marker, wait: false});
  expect(result.text).toBe(text);
});

test('several KB on one line reach the native turn as task text', async ($, on) => {
  const text = 'Summarize: ' + 'a long note '.repeat(700) + '  \n';
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', marker, text})}));
  on('fs.write', () => ({value: undefined}));
  on('session.id', () => ({value: 'native-test'}));
  on('prompt.submit', (_, e) => ({text: e.text}));
  const result = await $.prompt.submit({origin: {kind: 'composer'}, text: marker, wait: false});
  expect(result.text).toBe(text);
});

test('command-shaped task text is injected without passing through the composer', async ($, on) => {
  let text = '!touch /a-test-only-path\n';
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', marker, text})}));
  on('fs.write', () => ({value: undefined}));
  on('session.id', () => ({value: 'native-test'}));
  on('prompt.submit', (_, e) => ({text: e.text}));
  const bash = await $.prompt.submit({origin: {kind: 'composer'}, text: marker, wait: false});
  expect(bash.text).toBe(text);
  text = '/clear\n';
  const clear = await $.prompt.submit({origin: {kind: 'composer'}, text: marker, wait: false});
  expect(clear.text).toBe(text);
});

test('task text itself is not accepted instead of the dispatch marker', async ($, on) => {
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', marker, text: 'TASK'})}));
  on('fs.write', () => ({value: undefined}));
  on('session.id', () => ({value: 'native-test'}));
  on('prompt.submit', (_, e) => ({text: e.text}));
  const result = await $.prompt.submit({origin: {kind: 'composer'}, text: 'TASK', wait: false});
  expect(result.drop).toBe('Agency prompt does not match this dispatch');
});

test('a mismatched marker is dropped before a model turn', async ($, on) => {
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', marker, text: 'NEXT_PROMPT\n'})}));
  on('fs.write', () => ({value: undefined}));
  on('session.id', () => ({value: 'native-test'}));
  on('prompt.submit', (_, e) => ({text: e.text}));
  const result = await $.prompt.submit({origin: {kind: 'composer'}, text: 'HCTL2_DISPATCH_OLD', wait: false});
  expect(result.drop).toBe('Agency prompt does not match this dispatch');
});

test('the same dispatch retry replaces a restored marker draft', async ($, on) => {
  on('fs.read', () => ({value: JSON.stringify({id: 'fresh', digest: 'd1', marker, text: 'TASK\n'})}));
  on('fs.write', () => ({value: undefined}));
  on('session.id', () => ({value: 'native-test'}));
  on('prompt.edit', (_, e) => ({
    text: e.text.slice(0, e.start) + e.inputText + e.text.slice(e.end),
    cursor: e.start + e.inputText.length,
  }));
  on('prompt.submit', (_, e) => ({text: e.text}));
  await $.prompt.edit({origin: {kind: 'composer'}, text: '', cursor: 0, start: 0, end: 0, inputText: marker});
  await $.prompt.submit({origin: {kind: 'composer'}, text: marker, wait: false});
  const result = await $.prompt.edit({origin: {kind: 'composer'}, text: marker,
    cursor: marker.length, start: marker.length, end: marker.length, inputText: marker});
  expect(result.text).toBe(marker);
});
