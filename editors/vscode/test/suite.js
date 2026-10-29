// Runs inside VS Code. The workspace is a copy of examples/bakery/, which
// checks clean, so the expected count is known: none, until a reference is
// broken, and then exactly the one problem in the file that names it.
const assert = require('node:assert');
const path = require('node:path');
const vscode = require('vscode');

const until = async (what, test, ms = 20000) => {
	const end = Date.now() + ms;
	for (;;) {
		const got = test();
		if (got) return got;
		if (Date.now() > end) throw new Error(`timed out waiting for ${what}`);
		await new Promise((r) => setTimeout(r, 100));
	}
};

async function run() {
	const root = process.env.SPRIG_TEST_WORKSPACE;
	const uri = (name) => vscode.Uri.file(path.join(root, name));

	const lease = await vscode.workspace.openTextDocument(uri('lease.sprig'));
	assert.strictEqual(lease.languageId, 'sprig', '.sprig files are Sprig');
	await vscode.window.showTextDocument(lease);
	const ext = vscode.extensions.all.find((e) => e.packageJSON.name === 'sprig');
	await ext.activate();

	// The language server has read the folder; nothing in it is wrong.
	await new Promise((r) => setTimeout(r, 2000));
	const problems = () => vscode.languages.getDiagnostics().flatMap(([u, ds]) => ds.map((d) => [path.basename(u.fsPath), d]));
	assert.deepStrictEqual(problems(), [], 'examples/bakery checks clean');

	// Delete ^signed in lease.sprig: bakery.sprig and kitchen.sprig, which
	// name it, gain a problem each, without being opened.
	const at = lease.getText().indexOf('  ^signed');
	const edit = new vscode.WorkspaceEdit();
	edit.delete(lease.uri, new vscode.Range(lease.positionAt(at), lease.positionAt(at + '  ^signed'.length)));
	await vscode.workspace.applyEdit(edit);
	const found = await until('diagnostics in the files that name ^signed', () => {
		const p = problems();
		return p.length === 2 ? p : null;
	});
	const where = found.map(([file, d]) => `${file}:${d.range.start.line + 1} ${d.code}`).sort();
	assert.deepStrictEqual(where, ['bakery.sprig:15 S0203', 'kitchen.sprig:18 S0203']);
	console.log(`vscode: ${where.join(', ')}`);
}

module.exports = { run };
