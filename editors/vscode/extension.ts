import { workspace, ExtensionContext, commands, window } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext) {
  // Start the `fav lsp` server as the language server
  const serverOptions: ServerOptions = {
    command: 'fav',
    args: ['lsp'],
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    // Activate for .fav files
    documentSelector: [{ scheme: 'file', language: 'favnir' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.fav'),
    },
  };

  client = new LanguageClient(
    'favnir',
    'Favnir Language Server',
    serverOptions,
    clientOptions,
  );

  // Register commands — quote fileName to prevent shell injection
  context.subscriptions.push(
    commands.registerCommand('favnir.run', () => {
      const terminal = window.createTerminal('Favnir');
      const file = JSON.stringify(window.activeTextEditor?.document.fileName ?? '');
      terminal.sendText(`fav run ${file}`);
      terminal.show();
    }),
    commands.registerCommand('favnir.check', () => {
      const terminal = window.createTerminal('Favnir Check');
      const file = JSON.stringify(window.activeTextEditor?.document.fileName ?? '');
      terminal.sendText(`fav check ${file}`);
      terminal.show();
    }),
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
