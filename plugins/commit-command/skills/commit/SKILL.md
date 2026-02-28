---
name: commit
description: >
  Use this skill when the user says "/commit", "commit", "コミットして",
  or wants to create a git commit.
tools: Bash
---

以下の手順でgitコミットを作成する。

## 1. コミット計画の作成

### 1-1. 変更内容の確認
以下のコマンドを順番に実行して変更内容を把握する:

```bash
git status
git diff
git diff --cached
git log --oneline -5
```

### 1-2. コミット計画の作成
変更内容を作業の単位に分割し、コミットの内容を計画してユーザーに提案する。
Conventional Commits形式（`feat:`, `fix:`, `chore:` など）で一行で書くこと

```md
# commit1: xxxx
feat: xxxx
変更ファイル:
- file1
- file2

# commit2: xxx
test: xxxx
変更ファイル:
- file1
- file2
```
AskUserQuestionでコミット計画の修正が必要ないかを確認し、修正の必要があれば修正する。

## 2. コミットの実行
ユーザーに承認してもらった手順でファイルをステージングしてコミットする

- `git add -A` や `git add .` は避け、関係するファイルを個別に指定する
- `.env` や認証情報を含むファイルは絶対にコミットしない

```bash
git add --renormalize <関係するファイル>
git commit -m "<コミットメッセージ>"
```

## 3. 結果を確認する

```bash
git status
```
コミットが成功したことをユーザーに伝える。
