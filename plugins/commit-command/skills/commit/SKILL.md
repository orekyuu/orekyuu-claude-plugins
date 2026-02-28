---
name: commit
description: >
  Use this skill when the user says "/commit", "commit", "コミットして",
  or wants to create a git commit.
tools: Bash
---

以下の手順でgitコミットを作成する。

## 1. 変更内容を確認する

以下のコマンドを並列で実行して変更内容を把握する:

```bash
git status
git diff
git diff --cached
git log --oneline -5
```

## 2. コミットメッセージを作成する

- 変更内容を分析してコミットメッセージを考える
- `git log` の結果から、このリポジトリのコミットメッセージのスタイルに従う
- Conventional Commits形式（`feat:`, `fix:`, `chore:` など）が使われていれば従う

## 3. ファイルをステージングしてコミットする

```bash
git add <関係するファイル>
git commit -m "$(cat <<'EOF'
<コミットメッセージ>
EOF
)"
```

- `git add -A` や `git add .` は避け、関係するファイルを個別に指定する
- `.env` や認証情報を含むファイルは絶対にコミットしない

## 4. 結果を確認する

```bash
git status
```

コミットが成功したことをユーザーに伝える。
