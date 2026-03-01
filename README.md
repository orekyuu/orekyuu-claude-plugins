# orekyuu-claude-plugins

Claude Code 用のプラグインコレクションです。

## プラグイン一覧

| プラグイン名    | カテゴリ    | 説明                                                                 |
| --------------- | ----------- | -------------------------------------------------------------------- |
| junit-report    | testing     | JUnit XML テストレポートを収集して表示する                           |
| commit-command  | git         | 適切なコミットメッセージで git コミットを作成する                    |
| sdd             | development | 仕様書策定→テスト設計→実装の順で機能を開発する                       |
| mcp-ide-gateway | ide         | MCP-IDE-Gateway を MCP サーバーとして登録し JetBrains IDE と連携する |

## インストール方法

### 1. マーケットプレイスを追加する

Claude Code で以下のコマンドを実行してマーケットプレイスを登録します。

```
/plugin marketplace add orekyuu/orekyuu-claude-plugins
```

### 2. プラグインをインストールする

インストールしたいプラグインを個別に追加します。

```
/plugin install junit-report@orekyuu-plugins
/plugin install commit-command@orekyuu-plugins
/plugin install mcp-ide-gateway@orekyuu-plugins
```

または `/plugin` コマンドで対話式 UI を開き、**Discover** タブから選択してインストールすることもできます。

### アンインストール

```
/plugin uninstall junit-report@orekyuu-plugins
/plugin uninstall commit-command@orekyuu-plugins
/plugin uninstall mcp-ide-gateway@orekyuu-plugins
```

マーケットプレイスごと削除する場合（インストール済みプラグインも全て削除されます）:

```
/plugin marketplace remove orekyuu-plugins
```
