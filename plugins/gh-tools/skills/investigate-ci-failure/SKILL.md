---
name: investigate-ci-failure
description: >
  Use this skill when the user asks to investigate CI failures, "CIの失敗を調査して",
  "failed CI logs", "なぜCIが失敗したか調べて", "CI failure analysis",
  "CIのエラーログを見せて", "CIが落ちた原因を教えて", or wants to understand why CI failed.
tools: Bash, Agent, AskUserQuestion
---

## CI 失敗調査手順

ログ取得・分析はコンテキストを大量に消費するため、**必ず Agent ツールでサブエージェントに委譲して実行する**。
メインのコンテキストでは run の特定・結果の表示のみを行う。

### 1. 対象の run を特定する

ユーザーの発話から判断する：

- **run ID が明示されている** → そのまま使用する
- **PR 番号が明示されている** → `gh pr view <number> --json headRefName` でブランチ名を取得し、そのブランチの最新失敗 run を取得する
- **指定なし（デフォルト）** → 現在のブランチの最新失敗 run を取得する

```bash
# 現在のブランチを確認
git rev-parse --abbrev-ref HEAD

# 現在のブランチの失敗した run を最新順に取得
gh run list --branch <current-branch> --status failure --limit 5
```

複数の失敗 run がある場合は最新のものを対象にする。
run が見つからない場合はユーザーに伝えて終了する。

### 2. Agent でログ調査を実行する

run ID とブランチが確定したら、**Agent ツールを使ってサブエージェントに調査を委譲する**。

```
Agent(
  subagent_type: "gh-tools:ci-failure-investigator",
  prompt: """
以下の run の失敗を調査してください。

runId: <run-id>
branch: <branch>
"""
)
```

### 3. 結果を表示する

Agent から返ってきた調査結果をそのままユーザーに報告する。

修正方針が明確な場合は、修正に取りかかるかどうかをユーザーに確認する。
