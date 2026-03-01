---
name: run-test
description: >
  Use this skill when the user asks to run tests, "テストを実行して", "テストを走らせて",
  "run tests", "execute tests", or wants to verify test results for a specific file or test method.
tools: mcp__intellij-mcp__list_projects, mcp__intellij-mcp__find_file, mcp__intellij-mcp__list_test_configurations, mcp__intellij-mcp__run_test, Agent
---

## テスト実行手順

テスト実行はトークン消費が大きいため、**必ず Agent ツールでサブエージェントに委譲して実行する**。
メインのコンテキストではプロジェクト情報の収集と結果の表示のみを行う。

### 1. プロジェクトを取得する

```
list_projects()
```

projectPath を取得する。複数プロジェクトがある場合は AskUserQuestion でどのプロジェクトを対象にするか確認する。

### 2. 対象ファイルを特定する

ユーザーの入力から以下を判断する：

- **ファイルパスが明示されている** → そのまま使用する
- **クラス名やファイル名のみ** → `find_file(fileName, projectPath)` でファイルパスを特定する
- **テスト全体を実行** → ユーザーに対象ファイルを確認する

### 3. テスト設定を確認する

**Agent に委譲する前に**、メインのコンテキストで `list_test_configurations()` を呼び出して設定を確認する。

```
list_test_configurations(filePath, projectPath)
```

- 設定が1つだけ → そのまま使用する
- 設定が複数ある（Gradle のマルチモジュールなど）→ AskUserQuestion でどの設定を使うか確認する
- 設定が取得できない → `configurationName` なしで実行する

### 4. Agent でテストを実行する

projectPath・filePath 一覧・configurationName が揃ったら、**Agent ツールを使ってサブエージェントにテスト実行を委譲する**。

```
Agent(
  subagent_type: "run-test-executor",
  prompt: "以下のテストを順番に実行して結果を報告してください。\n\nprojectPath: ...\nconfigurationName: ...\n実行対象:\n- filePath: ...\n- filePath: ..."
)
```

- `configurationName` は手順3で確定したものを必ず含める（未確定の場合は「設定なし」と明記する）
- テスト実行に必要な情報はすべて prompt に含める
- 複数ファイルがある場合も **1つの Agent 呼び出し**にまとめる
- Agent 内では `run_test()` を1ファイルずつ順番に実行する（並列不可）

### 4. 結果を表示する

Agent から返ってきた結果をもとに、メインのコンテキストでユーザーに報告する。

**複数ファイルの場合はテーブル形式でまとめる：**

| ファイル       | 結果        | 詳細                        |
| -------------- | ----------- | --------------------------- |
| `FooTest.java` | ✅ 全件成功 | 3件                         |
| `BarTest.java` | ❌ 失敗あり | 1件失敗（エラー内容を表示） |

- **成功** → 合格したテスト数を表示する
- **失敗** → 失敗したテスト名・エラーメッセージ・スタックトレースをわかりやすく表示する
- **エラー** → テスト実行自体が失敗した場合はその理由を伝える

失敗があった場合は、原因の調査や修正方法の提案を行う。
