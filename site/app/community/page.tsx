export const metadata = {
  title: 'Community — Favnir',
  description: 'Join the Favnir community on GitHub Discussions and Discord.',
};

export default function CommunityPage() {
  return (
    <main>
      <h1>Favnir Community</h1>
      <p>Join the Favnir community. We discuss ideas, share Rune recipes, and support each other.</p>
      <ul>
        <li>
          <a href="https://github.com/favnir/favnir/discussions">GitHub Discussions</a>
          {' '}— Q&amp;A, feature requests, and announcements
        </li>
        <li>
          <span>Discord（coming soon）</span>
          {' '}— real-time chat
        </li>
        <li>
          <a href="https://github.com/favnir/favnir/blob/main/CONTRIBUTING.md">Contributing Guide</a>
          {' '}— how to contribute Runes and fixes
        </li>
      </ul>

      <section>
        <h2>第 1 回 Favnir Rune コンテスト（2026-07）</h2>
        <p>
          コミュニティ Rune を作って公開しよう！優秀な Rune は公式 README に掲載されます。
        </p>
        <h3>募集要件（5 条件）</h3>
        <table>
          <thead>
            <tr><th>条件</th><th>内容</th></tr>
          </thead>
          <tbody>
            <tr><td>connect</td><td>環境変数でサービスに接続できる</td></tr>
            <tr><td>read</td><td>データを取得する関数が 1 つ以上</td></tr>
            <tr><td>write</td><td>データを書き込む関数が 1 つ以上</td></tr>
            <tr><td>error</td><td>エラーを Result.err で返す（クラッシュしない）</td></tr>
            <tr><td>test</td><td>cargo test で 3 件以上 PASS する</td></tr>
          </tbody>
        </table>
        <h3>応募方法</h3>
        <ol>
          <li><code>runes/&lt;rune-name&gt;/</code> に <code>rune.toml</code> と <code>.fav</code> を追加</li>
          <li>PR を開く（タイトルに <code>[コンテスト]</code> を付ける）</li>
          <li>レビュー後、優秀作品を発表</li>
        </ol>
        <h3>特典</h3>
        <ul>
          <li>公式 README への掲載</li>
          <li>Favnir グッズ（ステッカー等）</li>
          <li>コントリビューターバッジ</li>
        </ul>
        <p>
          詳細は{' '}
          <a href="https://github.com/favnir/favnir/blob/main/CONTRIBUTING.md">CONTRIBUTING.md</a>
          {' '}を参照。
        </p>
      </section>
    </main>
  );
}
