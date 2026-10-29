// Every built page at a phone's width and a tablet's, and fail on what makes a
// page hard to use by touch:
//
// 1. The page scrolls sideways. Wide content (a terminal, a table) may scroll
//    inside its own box; the page itself may not.
// 2. A control smaller than its target size: at a phone's width, 44 by 44 CSS
//    pixels, Apple's and WCAG 2.5.5's; at a tablet's, WCAG 2.5.8's 24. A link
//    inside a sentence is exempt, as WCAG exempts it: the sentence is what
//    sets its size.
// 3. A text field whose text is under 16px, which iOS zooms the page into on
//    focus and leaves zoomed.
//
// It needs Chrome, as scripts/a11y.mjs does.
import { pages as built, run } from './harness.mjs';

const pages = built();
// Width, and the smallest a control may be at it.
const widths = [
	[360, 44],
	[768, 24],
];

const harness = `<!doctype html><meta charset="utf-8"><body><pre id="out">pending</pre><script>
const pages = ${JSON.stringify(pages)};
const widths = ${JSON.stringify(widths)};
const results = [];
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
const CONTROLS = 'a[href], button, input:not([type=hidden]), select, textarea, summary, [role=button], [role=tab], [role=checkbox], [role=switch]';
const TEXT = 'p, li, td, th, dd, dt, figcaption, blockquote, label, small';
function name(el) {
  const t = (el.getAttribute('aria-label') || el.textContent || el.value || '').trim().replace(/\\s+/g, ' ').slice(0, 30);
  return el.tagName.toLowerCase() + (el.id ? '#' + el.id : '') + (el.classList.length ? '.' + [...el.classList].slice(0, 2).join('.') : '') + (t ? ' "' + t + '"' : '');
}
async function check(page, [width, least]) {
  const f = document.createElement('iframe');
  f.style.cssText = 'width:' + width + 'px;height:800px;border:0';
  document.body.appendChild(f);
  await new Promise((r) => { f.onload = r; f.src = page; });
  await wait(400);
  const w = f.contentWindow;
  const d = f.contentDocument;
  const style = (el) => w.getComputedStyle(el);
  const shown = (el) => {
    for (let a = el; a && a !== d.body; a = a.parentElement) {
      const s = style(a);
      if (s.display === 'none' || s.visibility === 'hidden' || a.hidden || a.inert) return false;
      if (a.tagName === 'DETAILS' && !a.open && a !== el.closest('summary')?.parentElement) return false;
      if (a.tagName === 'DIALOG' && !a.open) return false;
      // Visually hidden until focused, like a skip link.
      if (s.clip === 'rect(0px, 0px, 0px, 0px)' || /inset\(50%\)/.test(s.clipPath)) return false;
    }
    const b = el.getBoundingClientRect();
    // Skip links and the like sit off the page until focused.
    return b.width > 0 && b.height > 0 && b.right > 0 && b.left < width && b.bottom > 0;
  };
  const scroll = d.documentElement.scrollWidth;
  if (scroll > width) results.push({ page, width, rule: 'scrolls sideways', what: scroll + 'px wide' });
  // WCAG 2.5.8's spacing exception, for the AA size only: a small target is
  // fine when a circle of the target size centred on it touches no other
  // target. 2.5.5, the phone's size, has no such exception.
  const spaced = least < 44;
  const targets = [...d.querySelectorAll(CONTROLS)].filter(shown).map((el) => [el, el.getBoundingClientRect()]);
  const apart = (el, b) => {
    const cx = b.left + b.width / 2, cy = b.top + b.height / 2, r = least / 2;
    return targets.every(([o, ob]) => {
      if (o === el || o.contains(el) || el.contains(o)) return true;
      const dx = Math.max(ob.left - cx, 0, cx - ob.right), dy = Math.max(ob.top - cy, 0, cy - ob.bottom);
      return dx * dx + dy * dy >= r * r;
    });
  };
  for (const el of d.querySelectorAll(CONTROLS)) {
    if (!shown(el)) continue;
    const s = style(el);
    // A link in running text: inline, inside a block that says more than it does.
    const block = el.closest(TEXT);
    const inline = s.display === 'inline' || s.display === 'inline-block' && el.tagName === 'A';
    if (inline && block && block.textContent.trim().length > el.textContent.trim().length + 1) continue;
    // A field's label is part of its target: tapping either one works.
    const b = (el.matches('input, select, textarea') && el.closest('label') || el).getBoundingClientRect();
    if ((b.width < least || b.height < least) && !(spaced && apart(el, b))) results.push({ page, width, rule: 'control under ' + least + 'px', what: name(el) + ' ' + Math.round(b.width) + 'x' + Math.round(b.height) });
  }
  for (const el of d.querySelectorAll('textarea, input:not([type]), input[type=text], input[type=search], input[type=email], input[type=url], [contenteditable=true]')) {
    if (!shown(el)) continue;
    const size = parseFloat(style(el).fontSize);
    if (size < 16) results.push({ page, width, rule: 'text field under 16px', what: name(el) + ' ' + size + 'px' });
  }
  f.remove();
}
(async () => {
  for (const p of pages) for (const w of widths) await check(p, w);
  document.getElementById('out').textContent = JSON.stringify(results);
})().catch((e) => { document.getElementById('out').textContent = 'ERROR ' + e.stack; });
</script>`;

run({ harness, budget: pages.length * widths.length * 4000 })
	.then((out) => {
		const problems = JSON.parse(out);
		if (problems.length) {
			console.error(`mobile: ${problems.length} problem(s)`);
			let last = '';
			for (const p of problems) {
				const head = `  ${p.page} at ${p.width}px`;
				if (head !== last) console.error(head);
				last = head;
				console.error(`    ${p.rule}: ${p.what}`);
			}
			process.exit(1);
		}
		console.log(`mobile: ${pages.length} pages at ${widths.map(([w, n]) => `${w}px (controls ${n}px)`).join(' and ')}: nothing scrolls sideways, no control too small`);
	})
	.catch((e) => {
		console.error(`mobile: ${e.message}`);
		process.exit(1);
	});
