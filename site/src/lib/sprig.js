// The draft 0.2 reference parser, extracted unchanged in behaviour from
// design/sprig-draft-0.2.html so the site can run it at build time and in the
// browser. It is the behavioural oracle CLAUDE.md names until sprig-core
// exists, and cairn 0109 replaces it with sprig-core compiled to WebAssembly:
// the site must not ship with two parsers that could disagree.
//
// The design page kept its workspace in module state, and so does this. A
// render is: useWorkspace(files), beginRender(), build and render, endRender().

const TODAY = (() => { const d = new Date(); d.setHours(0,0,0,0); return d; })();
const addDays = (d, n) => new Date(d.getFullYear(), d.getMonth(), d.getDate() + n);
const iso = d => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;

/* ---------- language ---------- */
const MARKS = {'-':'todo','~':'doing','x':'done','/':'dropped','>':'later','?':'ask','+':'graft','#':'group','=':'answer'};
const LINE_RE = /^([-~x\/>?+#=])(?:\s+(.*))?$/;
const TOKEN_RE = /[^\s"]*"[^"]*"\S*|\S+/g;
const FIELD_RE = /^([a-z][\w-]*):(.+)$/i;
const DATE_KEYS = new Set(['due','start','target']);
const OPEN = new Set(['todo','doing','ask']);
const SETTLING = new Set(['done','dropped','later']);
const TICKABLE = '-~x/>?';
const MONTHS = ['january','february','march','april','may','june','july','august','september','october','november','december'];
const DAYS = ['sunday','monday','tuesday','wednesday','thursday','friday','saturday'];

const esc = s => String(s).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
const unquote = v => v.replace(/^"(.*)"$/, '$1');

function monthIndex(v) {
  v = String(v).toLowerCase();
  if (v.length < 3 || !/^[a-z]+$/.test(v)) return -1;
  return MONTHS.findIndex(m => m.startsWith(v));
}
function weekdayIndex(v) {
  v = String(v).toLowerCase();
  return v.length >= 3 ? DAYS.findIndex(d => d.startsWith(v)) : -1;
}

function parseDate(v) {
  if (!v) return null;
  v = String(v).toLowerCase().trim();
  let m;
  if ((m = v.match(/^(\d{4})-(\d{1,2})-(\d{1,2})$/))) return new Date(+m[1], m[2] - 1, +m[3]);
  if (v === 'today') return TODAY;
  if (v === 'tomorrow') return addDays(TODAY, 1);
  if ((m = v.match(/^\+(\d+)([dw])$/))) return addDays(TODAY, +m[1] * (m[2] === 'w' ? 7 : 1));
  if ((m = v.match(/^q([1-4])$/))) {
    let d = new Date(TODAY.getFullYear(), +m[1] * 3, 0);
    if (d < TODAY) d = new Date(TODAY.getFullYear() + 1, +m[1] * 3, 0);
    return d;
  }
  let mi = -1, day = 0;
  if ((m = v.match(/^([a-z]+)-?(\d{1,2})$/))) { mi = monthIndex(m[1]); day = +m[2]; }
  else if ((m = v.match(/^(\d{1,2})-?([a-z]+)$/))) { mi = monthIndex(m[2]); day = +m[1]; }
  if (mi >= 0) {
    const d = new Date(TODAY.getFullYear(), mi, day);
    // A bare month-day more than four months back means next year.
    if ((d - TODAY) / 864e5 < -120) d.setFullYear(d.getFullYear() + 1);
    return d;
  }
  const wd = weekdayIndex(v);
  return wd >= 0 ? addDays(TODAY, (wd - TODAY.getDay() + 7) % 7) : null;
}

function nextDue(base, every) {
  every = String(every).toLowerCase();
  let m;
  if (/^(day|daily)$/.test(every)) return addDays(base, 1);
  if (/^(week|weekly)$/.test(every)) return addDays(base, 7);
  if (/^(month|monthly)$/.test(every)) return new Date(base.getFullYear(), base.getMonth() + 1, base.getDate());
  if ((m = every.match(/^(\d+)([dw])$/))) return addDays(base, +m[1] * (m[2] === 'w' ? 7 : 1));
  const wd = weekdayIndex(every);
  return wd >= 0 ? addDays(base, ((wd - base.getDay() + 7) % 7) || 7) : null;
}

// Typed shortcuts become ISO so the file means the same thing tomorrow.
function expandDates(line) {
  return line.replace(/(^|\s)(due|start|target):([^\s"]+)/gi, (all, pre, k, v) => {
    if (/^\d{4}-\d{2}-\d{2}$/.test(v)) return all;
    const d = parseDate(v);
    return d ? `${pre}${k}:${iso(d)}` : all;
  });
}

function estHours(v) {
  const r = v && String(v).match(/^(\d+(?:\.\d+)?)([hdw])$/i);
  return r ? +r[1] * ({h:1, d:8, w:40}[r[2].toLowerCase()]) : 0;
}
const fmtEst = h => h < 8 ? `${+h.toFixed(1)}h` : `${+(h / 8).toFixed(1)}d`;

function parseInline(text) {
  const m = {who:[], tags:[], prio:0, id:null, fields:{}, after:[], words:[]};
  for (const t of text.match(TOKEN_RE) || []) {
    let r;
    if ((r = t.match(/^@([\w.\-]+)$/))) m.who.push(r[1]);
    else if ((r = t.match(/^#([\w\-]+)$/))) m.tags.push(r[1]);
    else if (/^!{1,3}$/.test(t)) m.prio = Math.max(m.prio, t.length);
    else if ((r = t.match(/^\^([\w\-]+)$/))) m.id = r[1];
    else if ((r = t.match(FIELD_RE)) && !r[2].startsWith('//')) {
      const key = r[1].toLowerCase(), val = unquote(r[2]);
      if (key === 'after') m.after.push(val); else m.fields[key] = val;
    }
    else m.words.push(t);
  }
  m.text = m.words.join(' ').replace(/\[\[([^\]]+)\]\]/g, '$1');
  return m;
}

function mkNode(state, text, indent, line, file) {
  return {state, meta: parseInline(text), children: [], notes: [], answers: [], indent, line, file};
}

function titleIndex(lines) {
  let i = 0;
  while (i < lines.length && (!lines[i].trim() || lines[i].trim().startsWith('//'))) i++;
  return i < lines.length && !/^\s/.test(lines[i]) && !LINE_RE.test(lines[i]) ? i : -1;
}

function parse(src, file) {
  const lines = src.replace(/\r/g, '').split('\n');
  const root = mkNode('root', '', -1, -1, file);
  const doc = {file, title: null, root, anchors: {}};
  const ti = titleIndex(lines);
  if (ti >= 0) { root.meta = parseInline(lines[ti].trim()); doc.title = root.meta.text || null; }
  const stack = [root];
  const top = () => stack[stack.length - 1];
  let gap = false;
  for (let i = ti + 1; i < lines.length; i++) {
    const raw = lines[i].replace(/^\t+/, t => '  '.repeat(t.length));
    const text = raw.trim();
    if (!text) { gap = true; continue; }
    if (text.startsWith('//')) continue;
    const indent = raw.length - raw.trimStart().length;
    while (stack.length > 1 && top().indent >= indent) stack.pop();
    const escaped = text.startsWith('\\');
    const m = !escaped && text.match(LINE_RE);
    if (m && m[1] === '=') top().answers.push(parseInline(m[2] || ''));
    else if (m) {
      const n = mkNode(MARKS[m[1]], m[2] || '', indent, i, file);
      top().children.push(n); stack.push(n);
    } else {
      const t = top();
      if (gap && t.notes.length) t.notes.push('');
      t.notes.push(escaped ? text.slice(1) : text);
    }
    gap = false;
  }
  return doc;
}

/* ---------- workspace: grafts, rollups, references ---------- */
let files = {};
let CACHE = {};
const BUILDING = new Set();

const fileKey = n => { n = n.trim().replace(/^\.\//, ''); return /\.sprig$/.test(n) ? n : n + '.sprig'; };
function linkParts(tok) {
  const r = tok.match(/\[\[([^\]\^]+)(?:\^([\w-]+))?\]\]/);
  return r ? {file: fileKey(r[1]), id: r[2] || null} : null;
}
const clone = n => ({...n, children: n.children.map(clone), cloned: true});
const labelOf = n => n.state === 'graft' ? (n.graftTitle || n.target || 'graft') : (n.meta.text || 'untitled');

function buildDoc(name) {
  if (name in CACHE) return CACHE[name];
  if (BUILDING.has(name)) return 'cycle';
  if (!(name in files)) return null;
  BUILDING.add(name);
  const doc = parse(files[name], name);
  graftAll(doc.root);
  BUILDING.delete(name);
  indexAnchors(doc.root, doc.anchors);
  roll(doc.root, null, doc.root.meta.who);
  CACHE[name] = doc;
  return doc;
}

function graftAll(n) {
  for (const c of n.children) {
    if (c.cloned) continue;
    if (c.state === 'graft') {
      const tok = c.meta.words.find(w => w.includes('[['));
      const lp = tok && linkParts(tok);
      if (!lp) c.error = 'A graft needs a link, like + [[kitchen]]';
      else {
        c.target = lp.file; c.targetId = lp.id;
        const t = buildDoc(lp.file);
        const src = t && t !== 'cycle' ? (lp.id ? t.anchors[lp.id] : t.root) : null;
        if (t === 'cycle') c.error = `${lp.file} is already above this line, so grafting it would loop`;
        else if (!t) c.error = `No file named ${lp.file} yet`;
        else if (!src) c.error = `No ^${lp.id} in ${lp.file}`;
        else {
          c.graftTitle = lp.id ? labelOf(src) : (t.title || lp.file);
          c.graftOwner = (src.who || [])[0] || null;
          c.graftNotes = src.notes;
          c.graftAnswers = lp.id ? src.answers : [];
          c.children = c.children.concat(src.children.map(clone));
        }
      }
    }
    graftAll(c);
  }
}

function indexAnchors(n, map) {
  for (const c of n.children) {
    if (c.cloned) continue;
    if (c.meta.id && !(c.meta.id in map)) map[c.meta.id] = c;
    indexAnchors(c, map);
  }
}

function roll(n, inh, who) {
  n.answered = n.state === 'ask' && n.answers.length > 0;
  const own = n.answered ? 'done' : n.state;
  const settled = inh || (SETTLING.has(own) ? own : null);
  n.inh = settled;
  n.who = n.meta.who.length ? n.meta.who : (n.state === 'graft' && n.graftOwner ? [n.graftOwner] : who);
  const s = {total:0, done:0, doing:0, est:0};
  const eh = estHours(n.meta.fields.est);
  if (!n.children.length) {
    const st = settled || own;
    const upkeep = !!n.meta.fields.every;
    if (!upkeep && (st === 'todo' || st === 'doing' || st === 'done' || st === 'ask')) {
      s.total = 1;
      if (st === 'done') s.done = 1;
      if (st === 'doing') s.doing = 1;
      if (st !== 'done') s.est = eh;
    }
  } else {
    for (const c of n.children) {
      const cs = roll(c, settled, n.who);
      s.total += cs.total; s.done += cs.done; s.doing += cs.doing; s.est += cs.est;
    }
    if (!s.est && eh && !settled) s.est = eh;
  }
  n.stats = s;
  const derived = s.total === 0 ? 'empty' : s.done === s.total ? 'done' : (s.doing || s.done) ? 'doing' : 'todo';
  n.view = settled || ((own === 'group' || own === 'graft' || own === 'root') ? derived : own);
  return s;
}

function resolveRef(ref, from) {
  let r;
  if ((r = ref.match(/^\^([\w-]+)$/))) {
    const d = buildDoc(from);
    const nd = d && d !== 'cycle' ? d.anchors[r[1]] : null;
    return {node: nd, label: nd ? labelOf(nd) : ref};
  }
  const lp = linkParts(ref);
  if (lp) {
    const d = buildDoc(lp.file);
    if (!d || d === 'cycle') return {node: null, label: ref};
    if (lp.id) { const nd = d.anchors[lp.id]; return {node: nd, label: nd ? labelOf(nd) : ref}; }
    return {node: d.root, label: d.title || lp.file};
  }
  return {node: null, label: ref};
}

function blockers(n) {
  if (!OPEN.has(n.view) || !n.meta.after.length) return null;
  const out = [];
  for (const ref of n.meta.after) {
    const t = resolveRef(ref, n.file);
    if (!t.node) out.push({label: ref, unknown: true});
    else if (t.node.view !== 'done') out.push({label: t.label});
  }
  return out.length ? out : null;
}

/* ---------- highlighting ---------- */
const linkSpans = s => esc(s).replace(/\[\[[^\]]+\]\]/g, x => `<span class="k-link">${x}</span>`);
function hlToken(t, note) {
  let r;
  if (note) return /^@[\w.\-]+[.,;:!?)]*$/.test(t) ? `<span class="k-who">${esc(t)}</span>` : linkSpans(t);
  if (/^@[\w.\-]+$/.test(t)) return `<span class="k-who">${esc(t)}</span>`;
  if (/^#[\w\-]+$/.test(t)) return `<span class="k-tag">${esc(t)}</span>`;
  if (/^!{1,3}$/.test(t)) return `<span class="k-prio">${t}</span>`;
  if (/^\^[\w\-]+$/.test(t)) return `<span class="k-id">${esc(t)}</span>`;
  if ((r = t.match(FIELD_RE)) && !r[2].startsWith('//')) {
    const v = r[2];
    const cls = v.startsWith('[[') ? 'k-link' : v.startsWith('^') ? 'k-id' : 'k-val';
    return `<span class="k-key">${esc(r[1])}:</span><span class="${cls}">${esc(v)}</span>`;
  }
  return linkSpans(t);
}
function hlInline(s, note) {
  return (s.match(/\s+|[^\s"]*"[^"]*"\S*|\S+/g) || []).map(t => /^\s/.test(t) ? t : hlToken(t, note)).join('');
}
function hlLine(ln) {
  const ind = ln.match(/^\s*/)[0];
  const body = ln.slice(ind.length);
  if (!body) return esc(ln);
  if (body.startsWith('//')) return ind + `<span class="k-com">${esc(body)}</span>`;
  if (body.startsWith('\\')) return ind + `<span class="k-esc">\\</span><span class="k-note">${hlInline(body.slice(1), true)}</span>`;
  const m = body.match(LINE_RE);
  if (!m) return ind + `<span class="k-note">${hlInline(body, true)}</span>`;
  const rest = hlInline(body.slice(1));
  if (m[1] === '#') return ind + `<span class="k-gm">#</span><span class="k-group">${rest}</span>`;
  if (m[1] === '=') return ind + `<span class="k-ans">=</span><span class="k-anst">${rest}</span>`;
  const st = MARKS[m[1]];
  return ind + `<span class="k-${st}">${esc(m[1])}</span><span class="kl-${st}">${rest}</span>`;
}
function highlight(src) {
  const lines = src.split('\n');
  const ti = titleIndex(lines);
  return lines.map((ln, i) => i === ti ? `<span class="k-title">${hlInline(ln)}</span>` : hlLine(ln)).join('\n');
}

/* ---------- rendering ---------- */
let REF = [];
let FP = null, HIDE = false, VM = 'tree';
const COLL = new Set();

function relDays(d) {
  if (d === 0) return 'today';
  if (d === 1) return 'tomorrow';
  if (d < 0) return `${-d}d ago`;
  if (d < 14) return `in ${d}d`;
  if (d < 60) return `in ${Math.round(d / 7)}w`;
  return `in ${Math.round(d / 30)}mo`;
}
function fmtDate(d) {
  const s = d.toLocaleDateString('en-US', {month: 'short', day: 'numeric'});
  return d.getFullYear() !== TODAY.getFullYear() ? `${s}, ${d.getFullYear()}` : s;
}
const daysFrom = d => Math.round((d - TODAY) / 864e5);
function dateChip(k, v, view) {
  const d = parseDate(v);
  const pre = k === 'due' ? '' : k + ' ';
  if (!d) return `<span class="chip">${pre}${esc(v)}</span>`;
  const diff = daysFrom(d);
  let cls = 'chip date', rel = '';
  if (!SETTLING.has(view)) {
    if (diff < 0 && k !== 'start') { cls += ' late'; rel = `${-diff}d late`; }
    else rel = relDays(diff);
  }
  return `<span class="${cls}">${pre}${fmtDate(d)}${rel ? ' · ' + rel : ''}</span>`;
}

function linkHTML(inner) {
  const lp = linkParts('[[' + inner + ']]');
  if (!lp) return esc(inner);
  return `<a href="#" class="flink" data-go="${esc(lp.file)}"${lp.id ? ` data-id="${esc(lp.id)}"` : ''}>${esc(inner)}</a>`;
}
function wordHTML(w) {
  const r = w.match(/^(.*?)\[\[([^\]]+)\]\](.*)$/);
  return r ? esc(r[1]) + linkHTML(r[2]) + esc(r[3]) : esc(w);
}
function inlineHTML(n) {
  if (n.state === 'graft') {
    const where = n.target ? n.target + (n.targetId ? '^' + n.targetId : '') : '';
    return `<span class="gt">${esc(n.graftTitle || n.target || 'Graft')}</span>` +
      (where ? ` <a href="#" class="fchip" data-go="${esc(n.target)}"${n.targetId ? ` data-id="${esc(n.targetId)}"` : ''}>${esc(where)}</a>` : '');
  }
  return n.meta.words.map(wordHTML).join(' ') || '<span class="untitled">untitled</span>';
}
function mdLite(s) {
  let h = esc(s);
  h = h.replace(/`([^`]+)`/g, '<code>$1</code>');
  h = h.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  h = h.replace(/(^|[\s(])\*([^*\s][^*]*)\*/g, '$1<em>$2</em>');
  h = h.replace(/(^|\s)(@[\w.\-]*\w)/g, '$1<span class="nwho">$2</span>');
  h = h.replace(/\[\[([^\]]+)\]\]/g, (_, x) => linkHTML(x.replace(/&amp;/g, '&')));
  h = h.replace(/(^|\s)(https?:\/\/[^\s<]+)/g, '$1<a href="$2" target="_blank" rel="noopener">$2</a>');
  return h;
}
function notesHTML(notes) {
  const paras = [];
  let cur = [];
  for (const l of notes) { if (l === '') { if (cur.length) paras.push(cur); cur = []; } else cur.push(l); }
  if (cur.length) paras.push(cur);
  return paras.map(p => `<p>${mdLite(p.join(' '))}</p>`).join('');
}
function answersHTML(list) {
  return list.map(a => {
    const chips = a.who.map(w => `<span class="chip who">@${esc(w)}</span>`).join('') +
      Object.entries(a.fields).map(([k, v]) => { const d = parseDate(v); return `<span class="chip kv">${esc(k)} ${d ? fmtDate(d) : esc(v)}</span>`; }).join('');
    return `<p class="ans"><span class="eq">=</span><span>${a.words.map(wordHTML).join(' ')}</span>${chips}</p>`;
  }).join('');
}

function chipsHTML(n, blk) {
  const f = n.meta.fields, c = [];
  if (n.meta.prio) c.push(`<span class="chip prio" title="Priority ${n.meta.prio}">${'!'.repeat(n.meta.prio)}</span>`);
  for (const w of n.meta.who) c.push(`<span class="chip who">@${esc(w)}</span>`);
  if (n.state === 'graft' && !n.meta.who.length && n.graftOwner) c.push(`<span class="chip who soft">@${esc(n.graftOwner)}</span>`);
  for (const k of ['start', 'due', 'target']) if (f[k]) c.push(dateChip(k, f[k], n.view));
  if (f.est && !n.children.length) c.push(`<span class="chip">${esc(f.est)}</span>`);
  if (f.every) c.push(`<span class="chip">every ${esc(f.every)}</span>`);
  for (const t of n.meta.tags) c.push(`<span class="chip tag">#${esc(t)}</span>`);
  for (const [k, v] of Object.entries(f)) {
    if (!['start', 'due', 'target', 'est', 'every'].includes(k)) c.push(`<span class="chip kv">${esc(k)} ${esc(v)}</span>`);
  }
  if (blk) for (const b of blk) {
    c.push(b.unknown ? `<span class="chip err">can't find ${esc(b.label)}</span>` : `<span class="chip block">waits on ${esc(b.label)}</span>`);
  }
  const s = n.stats;
  if (n.children.length && OPEN.has(n.state) && !n.inh && !n.answered && s.total && s.done === s.total) c.push('<span class="chip ready">all done · tick it</span>');
  if (n.meta.id) c.push(`<span class="chip id">^${esc(n.meta.id)}</span>`);
  if (n.error) c.push(`<span class="chip err">${esc(n.error)}</span>`);
  return c.join('');
}

function progHTML(n) {
  const s = n.stats;
  if (!n.children.length || !s.total) return '<span></span>';
  const pct = s.done / s.total * 100, dp = s.doing / s.total * 100;
  return `<span class="prog" title="${s.done} of ${s.total} done"><span class="bar"><i style="width:${pct}%"></i><b style="width:${dp}%"></b></span>${s.done}/${s.total}${s.est ? ' · ' + fmtEst(s.est) : ''}</span>`;
}

function visible(n) {
  if (HIDE && (n.view === 'done' || n.view === 'dropped')) return false;
  if (!FP) return true;
  if (!n.children.length) return n.who.includes(FP);
  return n.meta.who.includes(FP) || n.children.some(visible);
}

function markHTML(n, k) {
  if (n.state === 'group' || n.state === 'graft') return `<span class="mark m-${n.state} v-${n.view}" aria-hidden="true"></span>`;
  const inherited = n.inh && n.inh !== n.state && !n.answered;
  const disp = n.answered ? 'answered' : inherited ? n.inh : n.state;
  const label = n.state === 'done' ? 'Reopen' : n.meta.fields.every ? 'Done for now, move to next date' : 'Mark done';
  return `<button type="button" class="mark m-${disp}${inherited ? ' inherited' : ''}" data-act="toggle" data-k="${k}" aria-label="${label}: ${esc(labelOf(n))}" title="${label}"></button>`;
}

function nodeHTML(n, inhBlk) {
  if (!visible(n)) return '';
  const k = REF.push(n) - 1;
  const own = blockers(n);
  const blocked = OPEN.has(n.view) && !!(own || inhBlk);
  const kids = n.children.length > 0;
  const ck = `${n.file}:${n.line}`;
  const open = !COLL.has(ck);
  const caret = kids ? `<button type="button" class="caret" data-act="fold" data-ck="${esc(ck)}" aria-expanded="${open}" aria-label="${open ? 'Collapse' : 'Expand'}"></button>` : '<span></span>';
  let extra = n.notes.length ? `<div class="notes">${notesHTML(n.notes)}</div>` : '';
  if (open && n.graftNotes && n.graftNotes.length) extra += `<div class="notes">${notesHTML(n.graftNotes)}</div>`;
  const answers = n.answers.concat(n.graftAnswers || []);
  if (answers.length) extra += `<div class="answers">${answersHTML(answers)}</div>`;
  const inner = kids && open ? n.children.map(c => nodeHTML(c, blocked ? (own || inhBlk) : null)).join('') : '';
  return `<li class="node v-${n.view} s-${n.state}${blocked ? ' is-blocked' : ''}${kids ? ' has-kids' : ''}" data-src="${esc(ck)}">` +
    `<div class="row">${caret}${markHTML(n, k)}<div class="body"><span class="txt" data-act="src" data-k="${k}">${inlineHTML(n)}</span>${chipsHTML(n, own)}</div>${progHTML(n)}</div>` +
    extra + (inner ? `<ul>${inner}</ul>` : '') + '</li>';
}

function walkLeaves(root, fn) {
  (function w(n, path, ib, due, prio) {
    for (const c of n.children) {
      const own = blockers(c);
      const b = OPEN.has(c.view) && (own || ib) ? (own || ib) : null;
      const d = c.meta.fields.due || due;
      const p = Math.max(prio, c.meta.prio);
      if (!c.children.length) fn(c, path, b, d, p);
      else w(c, path.concat(labelOf(c)), b, d, p);
    }
  })(root, [], null, root.meta.fields.due || null, 0);
}

function nextHTML(doc) {
  const ready = [], waiting = [];
  walkLeaves(doc.root, (n, path, b, due, prio) => {
    if (!OPEN.has(n.view) || (FP && !n.who.includes(FP))) return;
    const d = parseDate(due);
    (b ? waiting : ready).push({n, path, b, t: d ? d.getTime() : Infinity, prio});
  });
  ready.sort((a, b) => (b.n.view === 'doing') - (a.n.view === 'doing') || b.prio - a.prio || a.t - b.t);
  const li = x => {
    const k = REF.push(x.n) - 1;
    const path = x.path.length ? x.path.map(esc).join(' <span>›</span> ') : esc(doc.title || doc.file);
    return `<li class="v-${x.n.view}${x.b ? ' is-blocked' : ''}">${markHTML(x.n, k)}<div><div class="path">${path}</div>` +
      `<div class="line"><span class="txt" data-act="src" data-k="${k}">${inlineHTML(x.n)}</span>${chipsHTML(x.n, x.b)}</div></div></li>`;
  };
  return `<div class="nx-h">Ready now · ${ready.length}</div>` +
    (ready.length ? `<ul class="nx">${ready.map(li).join('')}</ul>` : '<p class="empty">Nothing is ready. Everything open is waiting on something.</p>') +
    `<div class="nx-h">Waiting · ${waiting.length}</div>` +
    (waiting.length ? `<ul class="nx">${waiting.map(li).join('')}</ul>` : '<p class="empty">Nothing is blocked.</p>');
}

function propsHTML(doc) {
  const m = doc.root.meta, out = [];
  if (m.who.length) out.push(['owner', m.who.map(w => `<span class="chip who">@${esc(w)}</span>`).join(' ')]);
  for (const [k, v] of Object.entries(m.fields)) {
    const d = DATE_KEYS.has(k) ? parseDate(v) : null;
    out.push([k, d ? `${fmtDate(d)} <span class="rel">${relDays(daysFrom(d))}</span>` : esc(v)]);
  }
  for (const t of m.tags) out.push(['tag', '#' + esc(t)]);
  return out.map(([k, v]) => `<div><dt>${esc(k)}</dt><dd>${v}</dd></div>`).join('');
}

/* ---------- module interface ---------- */

export function useWorkspace(next) {
  files = next;
  CACHE = {};
}
export function setView({fp = null, hide = false, vm = 'tree', collapsed = []} = {}) {
  FP = fp; HIDE = hide; VM = vm;
  COLL.clear();
  for (const c of collapsed) COLL.add(c);
}
export function beginRender() { CACHE = {}; REF = []; }
export function endRender() { return REF.slice(); }

export {
  TODAY, addDays, iso, parseDate, nextDue, expandDates, fmtEst, esc,
  MARKS, LINE_RE, OPEN, TICKABLE, titleIndex, parse, buildDoc, blockers, labelOf,
  highlight, walkLeaves, nodeHTML, nextHTML, propsHTML, notesHTML,
};
