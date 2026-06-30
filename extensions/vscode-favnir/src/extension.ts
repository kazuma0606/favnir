import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  const serverCommand = vscode.workspace
    .getConfiguration('favnir')
    .get<string>('serverPath', 'fav');

  // fav lsp サブコマンドで LSP サーバーを起動（stdio transport）
  const serverOptions: ServerOptions = {
    run:   { command: serverCommand, args: ['lsp'], transport: TransportKind.stdio },
    debug: { command: serverCommand, args: ['lsp'], transport: TransportKind.stdio },
  };

  // FileSystemWatcher を明示的に管理してリソースリークを防ぐ
  const watcher = vscode.workspace.createFileSystemWatcher('**/*.fav');
  context.subscriptions.push(watcher);

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'favnir' }],
    synchronize: { fileEvents: watcher },
  };

  client = new LanguageClient(
    'favnir',
    'Favnir Language Server',
    serverOptions,
    clientOptions
  );
  client.start();

  // fav run / fav check をターミナルから実行するコマンドを登録
  context.subscriptions.push(
    vscode.commands.registerCommand('favnir.run', () => {
      const terminal = vscode.window.createTerminal('fav run');
      terminal.sendText('fav run');
      terminal.show();
    }),
    vscode.commands.registerCommand('favnir.check', () => {
      const terminal = vscode.window.createTerminal('fav check');
      terminal.sendText('fav check');
      terminal.show();
    })
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) return undefined;
  return client.stop();
}
