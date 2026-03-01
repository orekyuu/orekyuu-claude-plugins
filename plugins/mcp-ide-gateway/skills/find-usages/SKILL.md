---
name: find-usages
description: >
  Use this skill when the user asks to find usages of a class or method,
  "利用箇所を探して", "どこで使われているか", "find usages", "who calls this",
  or wants to know where a specific class or method is referenced.
tools: mcp__intellij-mcp__list_projects, mcp__intellij-mcp__find_usages, mcp__intellij-mcp__get_call_hierarchy, mcp__intellij-mcp__find_class
---

## 利用箇所の検索手順

### 1. プロジェクトを取得する

```
list_projects()
```

projectPath を取得する。複数プロジェクトがある場合は AskUserQuestion でどのプロジェクトを対象にするか確認する。

### 2. 検索対象を確認する

ユーザーの入力から以下を判断する：

- **クラス名のみ** → `find_usages(className, projectPath)`
- **クラス名 + メソッド/フィールド名** → `find_usages(className, projectPath, memberName)`
- クラス名が不明・曖昧な場合 → `find_class(className, projectPath)` で候補を絞り込み、AskUserQuestion で確認する

### 3. 利用箇所を検索する

```
find_usages(className, projectPath, memberName?)
```

- `className`: 完全修飾クラス名（例: `com.example.MyClass`）または単純名
- `memberName`: メソッド名またはフィールド名（省略時はクラス自体の利用箇所）

### 4. 結果を表示する

検索結果をファイルパス・行番号・コードスニペット付きで表示する。

結果が多い場合（20件以上）はファイル別にグループ化して表示する。

結果が0件の場合は「利用箇所が見つかりませんでした」と伝える。

---

## 呼び出し階層が必要な場合

メソッドの呼び出し元を再帰的に辿りたい場合は `get_call_hierarchy` を使う。

```
get_call_hierarchy(className, memberName, projectPath, depth?)
```

- `depth` のデフォルトは 3。深く辿りたい場合は depth を増やす（最大 10）。

ユーザーが「どこから呼ばれているか全部たどって」「呼び出し元を再帰的に」と言った場合に使用する。
