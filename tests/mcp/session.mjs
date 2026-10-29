// A scripted session with `sprig mcp`, through the official TypeScript MCP
// client rather than this repository's own: what an agent's client sees.
// It reads, picks the next item, ticks it, adds steps, answers a question,
// and then holds the result to `sprig check --strict`.
//
//   node session.mjs [path/to/sprig]
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { cpSync, mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '../..');
const sprig = resolve(process.argv[2] ?? join(root, 'target/debug/sprig'));
const dir = mkdtempSync(join(tmpdir(), 'sprig-mcp-'));
cpSync(join(root, 'examples/bakery'), dir, { recursive: true });
const env = { ...process.env, SPRIG_TODAY: '2026-10-01' };

const client = new Client({ name: 'sprig-ci', version: '0' });
await client.connect(new StdioClientTransport({ command: sprig, args: ['mcp', '--in', dir], env }));

const call = async (name, args = {}) => {
  const r = await client.callTool({ name, arguments: args });
  assert.equal(r.isError, false, `${name}: ${r.content?.[0]?.text}`);
  return r.structuredContent;
};
const step = (what) => console.log(`ok - ${what}`);

const { tools } = await client.listTools();
assert.deepEqual(
  tools.map((t) => t.name),
  ['list_plans', 'show_tree', 'next_items', 'find_items', 'check_plans',
   'tick_items', 'mark_item', 'add_items', 'answer_question', 'set_field'],
);
step(`${tools.length} tools listed`);

const plans = (await call('list_plans')).items;
assert.ok(plans.some((p) => p.file === 'bakery.sprig' && p.done === 9 && p.total === 40));
step('read: list_plans');

const tree = await call('show_tree', { file: 'lease' });
assert.equal(tree.file, 'lease.sprig');
step('read: show_tree');

const next = (await call('next_items', { who: 'dana', limit: 3 })).items;
assert.ok(next.length > 0);
step(`next: ${next[0].ref} ${next[0].node.label}`);

const ticked = await call('tick_items', { refs: ['^signed'] });
assert.match(ticked.diff, /\+x Sign the lease/);
assert.ok(ticked.no_longer_waiting.some((f) => f.text === 'Supplier credit line'));
step('tick: ^signed, which unblocked Supplier credit line');

const added = await call('add_items', { under: '^design', items: ['Call the plumber due:fri', 'Pick tiles  @teo'] });
assert.deepEqual(added.added.map((a) => a.text), ['- Call the plumber due:2026-10-02', '- Pick tiles  @teo']);
step('add: two steps under ^design');

const answered = await call('answer_question', { ref: '^name', text: 'Kiln & Crumb' });
assert.match(answered.diff, /\+ {2}= Kiln & Crumb/);
step('answer: ^name');

const refused = await client.callTool({ name: 'set_field', arguments: { ref: '^logo', key: 'after', value: '^nowhere' } });
assert.equal(refused.isError, true);
assert.match(refused.content[0].text, /error\[S0201\]/);
step('a write that breaks the plan is refused');

assert.deepEqual((await call('check_plans')).items, []);
await client.close();

// The files, not the server's word for them.
execFileSync(sprig, ['check', '--strict', dir], { env, stdio: 'inherit' });
assert.match(readFileSync(join(dir, 'kitchen.sprig'), 'utf8'), /^ {2}- Pick tiles {2}@teo$/m);
step('sprig check --strict passes on the result');
