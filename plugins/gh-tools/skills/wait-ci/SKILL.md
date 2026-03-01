---
name: wait-ci
description: >
  Use this skill when the user asks to wait for CI, watch CI results,
  "CIを待って", "CIが終わるまで待って", "wait for CI", "CIの結果を待って",
  "watch CI", "CIが終わったら教えて", or wants to monitor CI completion on the current branch or PR.
tools: Bash, AskUserQuestion
---

## CI 待機手順

`gh` コマンドを使って現在のブランチの CI 完了を待機する。

### 1. PR を特定する

現在のブランチから PR を取得する。

```bash
gh pr view --json number,headRefName,url
```

- PR が見つかった場合 → PR 番号・ブランチ名を確認する
- PR が見つからない場合（ブランチが push されていない、PR が未作成など）→ エラー内容をユーザーに伝えて終了する

### 2. CI の実行状況を確認する

ブランチの最新 run を取得する。

```bash
gh run list --branch <headRefName> --limit 5
```

- CI run がない場合 → 「CI が見つかりませんでした」と伝えて終了する
- 既に完了している（`completed`）場合 → 結果を報告してステップ 4 へ
- 実行中（`in_progress` / `queued`）の場合 → ステップ 3 へ

### 3. CI の完了を待機する

最新の run を `gh run watch` で待機する。

```bash
gh run watch <run-id> --exit-status
```

コマンドが完了したら run の最終状態を取得する。

```bash
gh run view <run-id> --json status,conclusion,name,databaseId,url
```

### 4. 結果を報告する

以下の形式でまとめる：

```
## CI 結果

**Run:** <name> (#<run-id>)
**ブランチ:** <headRefName>
**結果:** ✅ 成功 / ❌ 失敗 / ⚠️ キャンセル
```

| 結論 (`conclusion`) | 対応                                                                                  |
| ------------------- | ------------------------------------------------------------------------------------- |
| `success`           | 「CI が成功しました ✅」と報告して終了する                                            |
| `failure`           | 「CI が失敗しました ❌。失敗したジョブを調査しますか？」と AskUserQuestion で確認する |
| `cancelled`         | 「CI がキャンセルされました」と報告して終了する                                       |
| `skipped`           | 「CI がスキップされました」と報告して終了する                                         |

失敗の場合にユーザーが調査を希望したら、`gh-tools:investigate-ci-failure` スキルと同様の手順で調査を行う。
