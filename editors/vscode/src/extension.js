// The Chea extension: the TextMate grammar does the colouring, and `chea
// lsp` does everything else (problems, completion, definition, hover, code
// actions and formatting). The server is the binary bundled with the
// extension, so it works offline with nothing else installed; a setting or
// `chea` on PATH can stand in for it.
const fs = require('node:fs');
const path = require('node:path');
const vscode = require('vscode');
const { LanguageClient } = require('vscode-languageclient/node');

let client;

function server(context) {
	const configured = vscode.workspace.getConfiguration('chea').get('server.path');
	if (configured) return configured;
	const exe = process.platform === 'win32' ? 'chea.exe' : 'chea';
	const bundled = context.asAbsolutePath(path.join('server', exe));
	return fs.existsSync(bundled) ? bundled : 'chea';
}

async function activate(context) {
	const command = server(context);
	client = new LanguageClient(
		'chea',
		'Chea',
		{ run: { command, args: ['lsp'] }, debug: { command, args: ['lsp'] } },
		{
			documentSelector: [{ scheme: 'file', language: 'chea' }],
			synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher('**/*.chea') },
		},
	);
	await client.start();
	context.subscriptions.push({ dispose: () => client && client.stop() });
}

function deactivate() {
	return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
