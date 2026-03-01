---
name: sdd
description: >
  Use this skill when the user says "/sdd", "SDDで実装して", "仕様書を書いてから実装して",
  or wants to implement a feature following Specification-Driven Development.
tools: Bash, Agent
---

SDD（Specification-Driven Development）に従い、仕様策定 → テスト設計 → 実装 の順で機能を開発する。
各フェーズは専用エージェントが担当し、ファイルパスを通じて連携する。

## フェーズ1: 仕様策定

`sdd:sdd-spec` エージェントを起動して仕様書を作成する。
ユーザーの要件をそのままプロンプトに含める。

```
Agent: sdd:sdd-spec
prompt: <ユーザーの要件をそのまま含める>
```

エージェントが返すのは仕様書のファイルパス。このパスを変数として保持する。

## フェーズ2: テスト設計

フェーズ1で取得した仕様書パスを `sdd:sdd-test` エージェントに渡して起動する。

```
Agent: sdd:sdd-test
prompt: 仕様書のパス: <フェーズ1で取得したパス>
```

エージェントが返すのは作成したテストファイルのパス一覧。このパス一覧を保持する。

## フェーズ3: 実装

フェーズ1の仕様書パスとフェーズ2のテストファイルパスを `sdd:sdd-impl` エージェントに渡して起動する。

```
Agent: sdd:sdd-impl
prompt: |
  仕様書のパス: <フェーズ1で取得したパス>
  テストファイルのパス:
  - <フェーズ2で取得したパス1>
  - <フェーズ2で取得したパス2>
  ...
```

## 完了

すべてのフェーズが完了したらユーザーに実装完了を報告する。
