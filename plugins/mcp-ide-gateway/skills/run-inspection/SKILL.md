---
name: run-inspection
description: >
  Use this skill when the user asks to run inspections, "inspectionを実行して", "静的解析して",
  "コードの問題を調べて", "コード品質をチェックして", "run inspection", "analyze code quality",
  or wants to find and plan fixes for code issues detected by static analysis.
tools: mcp__intellij-mcp__list_projects, mcp__intellij-mcp__find_file, mcp__intellij-mcp__find_class, Agent, AskUserQuestion
---

## 静的解析・修正計画の手順

解析の実行と計画立案は `inspection-planner` に委譲する。
メインのコンテキストではスコープの確定・結果の提示・修正方針の確認のみを行う。

### 1. プロジェクトを取得する

```
list_projects()
```

複数プロジェクトがある場合は AskUserQuestion で対象を確認する。

### 2. 解析スコープを確定する

ユーザーの発話から対象を判断する：

- **ファイル・クラス名が明示されている** → `find_file()` または `find_class()` でパスを取得して `filePaths` として使う
- **「プロジェクト全体」「全部」など** → `filePaths` なし（プロジェクト全体）
- **スコープの指定がない（デフォルト）** → `git-scope-resolver` で git 変更ファイルを収集する（下記参照）
- **不明・曖昧な場合** → AskUserQuestion でスコープを確認する

重要度の指定がある場合（「エラーだけ」「警告以上」など）は minSeverity に変換する：

| ユーザーの表現                           | minSeverity |
| ---------------------------------------- | ----------- |
| 「エラーだけ」「コンパイルエラー」       | `ERROR`     |
| 「警告以上」「バグリスク」（デフォルト） | `WARNING`   |
| 「すべて」「スタイルも含めて」           | `INFO`      |

#### スコープ未指定時: git-scope-resolver で対象を収集する

ファイル・クラス・プロジェクト全体の指定がない場合は、**Agent ツールで git-scope-resolver に委譲**して対象ファイル一覧を取得する。

```
Agent(
  subagent_type: "mcp-ide-gateway:git-scope-resolver",
  prompt: """
inspection対象のファイルリストを収集してください。

projectPath: <projectPath>
"""
)
```

git-scope-resolver が返す「inspection 対象ファイル一覧」を `filePaths` として使う。
ファイルが1件も見つからなかった場合は AskUserQuestion で対象を確認する。

### 3. inspection-planner で解析・計画立案を実行する

スコープが確定したら **Agent ツールで inspection-planner に委譲する**。

```
Agent(
  subagent_type: "mcp-ide-gateway:inspection-planner",
  prompt: """
以下のスコープで静的解析を実行し、修正計画を立案してください。

projectPath: <projectPath>
filePaths:          # 省略時はプロジェクト全体を対象にする
  - src/main/java/.../Foo.java
  - src/main/java/.../Bar.java
minSeverity: <minSeverity>  # 省略時は WARNING
"""
)
```

- `filePaths` に複数ファイルを列挙するとファイルごとに解析が実行される
- `filePaths` を省略するとプロジェクト全体を対象にする

### 4. AskUserQuestion で修正方針を提案する

inspection-planner の報告をもとに、AskUserQuestion でユーザーに修正方針を選んでもらう。

選択肢は **inspection-planner が提案した戦略** をそのまま使う。
以下は選択肢の構成例：

```
AskUserQuestion(
  question: "解析が完了しました。どの修正方針で進めますか？",
  options: [
    {
      label: "戦略A: クイックウィン（推奨）",
      description: "未使用 import・変数など自動修正可能な N 件。リスク低。"
    },
    {
      label: "戦略B: 重要度優先",
      description: "ERROR・WARNING 全 N 件。バグリスクを大幅に低減。"
    },
    {
      label: "戦略C: ファイル単位",
      description: "最も問題が多い McpServerImpl.java に集中して N 件修正。"
    },
    {
      label: "計画だけ確認する（修正はしない）",
      description: "修正は行わず、問題の詳細と計画を確認するだけにする。"
    }
  ]
)
```

選択肢の内容・数は inspection-planner の報告に合わせて柔軟に構成すること。
必ず「計画だけ確認する（修正はしない）」を最後の選択肢として含める。

### 5. 選択された方針を報告する

ユーザーが選んだ方針をもとに、以下のように応答する。

- **修正戦略を選んだ場合** — 「`<戦略名>` で進めます。修正を開始してよいですか？」と確認してから、対応する修正エージェントまたは手順を案内する
- **「計画だけ確認する」を選んだ場合** — inspection-planner の問題詳細・修正計画をそのまま表示してセッションを終了する
