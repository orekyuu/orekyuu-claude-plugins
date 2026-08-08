---
name: adversarial-verify
description: >
  Use this skill when the user says "/adversarial-verify", "敵対的検証して",
  "反証を試みて", "サブエージェントを立てて敵対的に検証して", "レビューして",
  "確認して", "チェックして",
  or wants to adversarially verify a document, specification, code, or other artifact.
tools: Agent, AskUserQuestion, Read, Glob, Grep, Write
---

検証対象が引数で指定されていない場合は AskUserQuestion で確認する（対象・重点観点）。
ファイルパスが渡された場合は Read / Glob / Grep で内容を取得する。

取得した内容を渡して `adversarial-verification:skeptic` エージェントを起動する。

エージェントの結果が返ったら AskUserQuestion で保存先パスを確認。
