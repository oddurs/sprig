// Downloads VS Code, starts it on a copy of examples/bakery/ with only this
// extension installed and no network extensions, and runs test/suite.js
// inside it.
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { runTests } = require('@vscode/test-electron');

async function main() {
	const here = path.join(__dirname, '..');
	const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'sprig-vscode-'));
	fs.cpSync(path.join(here, '..', '..', 'examples', 'bakery'), workspace, { recursive: true });
	try {
		await runTests({
			version: process.env.VSCODE_VERSION ?? 'stable',
			extensionDevelopmentPath: here,
			extensionTestsPath: path.join(__dirname, 'suite.js'),
			launchArgs: [workspace, '--disable-extensions', '--skip-welcome', '--skip-release-notes'],
			extensionTestsEnv: { SPRIG_TEST_WORKSPACE: workspace },
		});
	} catch (e) {
		console.error('vscode: tests failed', e);
		process.exit(1);
	} finally {
		fs.rmSync(workspace, { recursive: true, force: true });
	}
}

main();
