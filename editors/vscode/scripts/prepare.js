// Copies what the extension ships from elsewhere in the repository: the
// grammar, which the website shares, and, when it has been built, the
// language server binary for this platform. A release packages one VSIX per
// target, each with that target's binary in server/.
const fs = require('node:fs');
const path = require('node:path');

const here = path.join(__dirname, '..');
const repo = path.join(here, '..', '..');
fs.mkdirSync(path.join(here, 'syntaxes'), { recursive: true });
fs.copyFileSync(path.join(repo, 'editors', 'sprig.tmLanguage.json'), path.join(here, 'syntaxes', 'sprig.tmLanguage.json'));
fs.copyFileSync(path.join(repo, 'LICENSE'), path.join(here, 'LICENSE'));

const exe = process.platform === 'win32' ? 'sprig.exe' : 'sprig';
// SPRIG_SERVER names the binary to bundle; otherwise the newest local build.
const built =
	process.env.SPRIG_SERVER ??
	['release', 'debug']
		.map((p) => path.join(repo, 'target', p, exe))
		.filter((p) => fs.existsSync(p))
		.sort((a, b) => fs.statSync(b).mtimeMs - fs.statSync(a).mtimeMs)[0];
if (built) {
	fs.mkdirSync(path.join(here, 'server'), { recursive: true });
	fs.copyFileSync(built, path.join(here, 'server', exe));
	fs.chmodSync(path.join(here, 'server', exe), 0o755);
	console.log(`prepare: grammar, and the server from ${path.relative(repo, built)}`);
} else {
	console.log('prepare: grammar; no built server, so the extension will look for sprig on PATH');
}
