---
name: get-pr-info
description: >
  Use this skill when the user asks to get PR information, "PRの情報を取得して",
  "PRの状態を確認して", "get PR info", "show PR details", "PRを確認して",
  "対応するPRを見せて", "PRのレビューコメントを見せて", "PRのCIの状態を教えて",
  or wants to see PR title, description, base branch, CI status, or review comments.
tools: Bash, AskUserQuestion
---

## PR 情報取得手順

`gh` コマンドを使って PR の情報を取得・表示する。

### 1. PR を特定する

ユーザーの発話から判断する：

- **PR 番号が明示されている** → そのまま使用する
- **指定なし（デフォルト）** → 現在のブランチの PR を取得する

```bash
# 現在のブランチの PR を取得（number, title, body, base/head, url, state, reviews を取得）
gh pr view --json number,title,body,baseRefName,headRefName,url,state,reviews
```

PR が見つからない場合は「現在のブランチに対応する PR が見つかりませんでした」とユーザーに伝えて終了する。

特定の PR 番号を指定する場合：

```bash
gh pr view <number> --json number,title,body,baseRefName,headRefName,url,state,reviews
```

### 2. CI チェックの詳細を取得する

```bash
gh pr checks <number>
```

各チェックの名前・状態・詳細 URL を取得する。

### 3. コメントを取得する

GitHub の PR コメントには3種類ある。`gh pr view --json comments` ではコード行への
レビューコメントが取得できないため、それぞれ `gh api` で取得する。

#### 3-1. コード行へのレビューコメント（インラインコメント）

```bash
gh api "repos/:owner/:repo/pulls/<number>/comments"
```

スレッド構造（返信関係）を把握するために `in_reply_to_id` フィールドを参照する。
`in_reply_to_id` が null のものがスレッドの起点、値があるものが返信。

#### 3-2. PR 全体へのコメント（Issue コメント）

```bash
gh api "repos/:owner/:repo/issues/<number>/comments"
```

#### 3-3. レビューの総括コメント（Review body）

`gh pr view --json reviews` で取得済みの `reviews[].body` を参照する。

### 4. 情報を整形して表示する

以下の形式でまとめる：

```
## PR #<number>: <title>

**URL:** <url>
**状態:** Open / Closed / Merged
**ベースブランチ:** <baseRefName> ← <headRefName>

### 説明

<body（空の場合は「説明なし」）>

### CI 状態

| チェック名 | 状態 | 詳細 |
|---|---|---|
| build | ✅ 成功 | — |
| test  | ❌ 失敗 | <url> |
| lint  | ⏳ 実行中 | — |

CI 全体: ✅ 全て成功 / ❌ 失敗あり / ⏳ 実行中 / ⚠️ 未実行

### レビュー状態

| レビュアー | 状態 |
|---|---|
| @alice | ✅ Approved |
| @bob   | ❌ Changes requested |

### コード行へのレビューコメント（インラインコメント）

📁 src/Foo.java:42
🟢 @alice: ここの変数名をわかりやすくしてください
 └─ 💬 @bob: 了解です、修正します

📁 src/Bar.java:10
🟢 @carol: このロジックは〜の方が良いと思います

### PR 全体へのコメント

💬 @dave: LGTM!
💬 @eve: 〜について確認させてください
```

#### CI 状態の絵文字マッピング

| `conclusion` / `status`  | 絵文字 |
| ------------------------ | ------ |
| `success`                | ✅     |
| `failure`                | ❌     |
| `in_progress` / `queued` | ⏳     |
| `cancelled`              | ⚠️     |
| `skipped`                | ⏭️     |
| `neutral`                | ➖     |

#### レビュー状態の絵文字マッピング

| `state`             | 絵文字 |
| ------------------- | ------ |
| `APPROVED`          | ✅     |
| `CHANGES_REQUESTED` | ❌     |
| `COMMENTED`         | 💬     |
| `PENDING`           | ⏳     |

各セクションにコメント・レビューがない場合は「なし」と表示する。

### 5. 追加案内

- CI が失敗している場合 → 「`gh-tools:investigate-ci-failure` で原因を調査できます」と案内する
- CI が実行中の場合 → 「`gh-tools:wait-ci` で完了を待機できます」と案内する
- Changes Requested がある場合 → レビュアーのコメント内容をもとに修正方針を提案する
