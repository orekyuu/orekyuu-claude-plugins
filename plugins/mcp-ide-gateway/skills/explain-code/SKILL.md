---
name: explain-code
description: >
  Use this skill when the user asks to explain code, "コードを解説して", "コードを説明して",
  "explain this code", "explain the code", "処理の流れを解説して", "機能の流れを教えて",
  or wants inline explanations added to a file in the IDE.
tools: mcp__intellij-mcp__list_projects, mcp__intellij-mcp__find_class, mcp__intellij-mcp__find_file, Agent, AskUserQuestion
---

## モードの判定

ユーザーの発話から、以下の2つのモードのどちらかを判定して処理を分岐する。

| モード             | 発話の例                                                                             | 処理                                                                   |
| ------------------ | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------- |
| **ファイル解説**   | 「`McpServerImpl` を解説して」「このファイルを説明して」                             | 特定ファイルに inline 注釈を追加する                                   |
| **処理フロー解説** | 「リクエストからレスポンスまでの流れを解説して」「ログイン機能の処理の流れを教えて」 | フローに関わる複数ファイルを特定し、各ファイルに inline 注釈を追加する |

---

## ファイル解説モード

単一ファイルの内容を解説する。

### 1. プロジェクトを取得する

```
list_projects()
```

projectPath を取得する。複数プロジェクトがある場合は AskUserQuestion でどのプロジェクトを対象にするか確認する。

### 2. 対象ファイルを特定する

ユーザーの入力から以下を判断する：

- **ファイルパスが明示されている** → そのまま使用する
- **クラス名のみ** → `find_class(className, projectPath)` でファイルパスを特定する
- **ファイル名のみ** → `find_file(fileName, projectPath)` でファイルパスを特定する
- **不明・曖昧な場合** → AskUserQuestion でどのファイルを解説するか確認する

### 3. Agent でコード解説を実行する

projectPath と filePath が揃ったら、**Agent ツールを使って code-annotator に委譲する**。

```
Agent(
  subagent_type: "mcp-ide-gateway:code-annotator",
  prompt: "以下のファイルのコードを解説してください。\n\nprojectPath: ...\nfilePath: ..."
)
```

### 4. 結果を表示する

Agent から返ってきた報告をもとに、追加したコメントの一覧（行番号・対象・概要）を表示する。

---

## 処理フロー解説モード

特定の機能・リクエスト処理などの流れを、複数ファイルにまたがって解説する。

### 1. プロジェクトを取得する

```
list_projects()
```

### 2. エントリーポイントを特定する

ユーザーの発話から起点となるクラスまたはメソッドを判断する。

- **クラス名・メソッド名が明示されている** → そのまま使用する
- **機能名・画面名など抽象的な表現のみ** → まず `code-investigator` で探索する（ユーザーに確認する前に探索を試みること）

#### 抽象的な表現からの探索手順

ユーザーが「ログイン機能」「ツール実行」のような機能名・概念名を言った場合、以下の手順でエントリーポイントを探す。

```
Agent(
  subagent_type: "mcp-ide-gateway:code-investigator",
  prompt: """
以下の機能・概念に関連するエントリーポイントを探索してください。

projectPath: <projectPath>
探索キーワード: <ユーザーの発話から抽出したキーワード（日本語・英語両方）>

## 探索の進め方

1. キーワードの英語表記・略称・関連語を複数考える
   - 例: 「ログイン」→ login, auth, authenticate, sign_in
   - 例: 「ツール実行」→ execute, call, invoke, tool, handler
2. search_symbol() でクラス・メソッドを横断的に検索する
3. 候補が見つかったら get_class_structure() でシグネチャを確認し、エントリーポイントらしいものを絞り込む
   - Controller・Handler・Listener・Service・Endpoint などのクラス名
   - handle・execute・process・run・call・invoke などのメソッド名
4. 有力候補を判断できた場合は理由とともに報告する

## 報告フォーマット

### エントリーポイント候補

| クラス名 | メソッド名 | filePath | 有力度 | 理由 |
|---|---|---|---|---|
| LoginController | login | src/.../LoginController.java | 高 | HTTPエンドポイントのハンドラ、ログイン処理の起点と判断 |
| AuthService | authenticate | src/.../AuthService.java | 中 | ログイン処理の中核だが直接の起点ではない可能性 |
"""
)
```

探索結果に応じて以下のように判断する：

- **有力候補が1つに絞れた** → そのまま次のステップへ進む
- **候補が複数ある** → AskUserQuestion でどれを起点にするか確認する
- **候補が見つからなかった** → AskUserQuestion で起点となるクラス・メソッドを教えてもらう

### 3. code-investigator でフローに関わるファイルを調査する

エントリーポイントから処理の流れをたどり、関連ファイルの一覧を取得する。

```
Agent(
  subagent_type: "mcp-ide-gateway:code-investigator",
  prompt: """
以下のエントリーポイントから始まる処理フローを調査してください。

projectPath: <projectPath>
エントリーポイント: <className>#<methodName>（またはクラス名のみ）

## 調査してほしい内容

1. エントリーポイントから呼び出される処理を call_hierarchy・find_usages などで追跡する
2. フローに登場するすべてのクラス・メソッドを特定する
3. 各クラスが全体フローのどのフェーズを担当するかを整理する

## 報告フォーマット

### 全体フロー概要
（フロー図または箇条書きで流れを表す）

### 関与するファイル一覧

| filePath（プロジェクトルートからの相対パス） | クラス名 | フローでの役割 |
|---|---|---|
| src/main/java/.../Foo.java | Foo | リクエストの受け口、バリデーションを担当 |
| src/main/java/.../Bar.java | Bar | DBアクセスを担当 |
"""
)
```

### 4. 各ファイルに code-annotator で注釈を追加する

調査結果に含まれるファイル一覧をもとに、**ファイルごとに順番に** code-annotator を呼び出す（並列実行しない）。

```
Agent(
  subagent_type: "mcp-ide-gateway:code-annotator",
  prompt: """
以下のファイルのコードを解説してください。

projectPath: <projectPath>
filePath: <filePath>

## このファイルのフローでの役割

<code-investigator の調査結果から該当ファイルのフロー説明を抜粋して渡す>

## 全体フロー概要（参考）

<code-investigator が報告した全体フロー図>
"""
)
```

- フロー情報を渡すことで、各ファイルへの注釈に「全体フローでの位置づけ」が反映される
- ファイル数が多い場合（5件以上）は AskUserQuestion でどのファイルを対象にするか絞り込む

### 5. 結果を表示する

すべてのファイルへの注釈追加が完了したら、以下の形式でまとめて報告する。

**全体フロー:**
（code-investigator が報告したフロー図をそのまま表示）

**注釈を追加したファイル:**

| ファイル             | フローでの役割                 | 追加したコメント数 |
| -------------------- | ------------------------------ | ------------------ |
| `McpServerImpl.java` | HTTPサーバーの起動・ツール登録 | 5件                |
| `McpToolBean.java`   | リクエストのディスパッチ       | 3件                |
