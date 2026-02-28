---
name: junit-report-collector
description: >
  Use this skill when the user asks to "show test results", "collect JUnit reports",
  "summarize test results", mentions "JUnit", "surefire reports", or wants to
  understand why tests failed.
tools: Bash
---

Run the following commands to collect and display JUnit XML reports:

## テスト結果ディレクトリを探す

```
junit-report results [directory]
```

- ディレクトリを省略すると現在のディレクトリ `.` を走査する
- JUnit XML ファイル（`TEST-*.xml` または `surefire-reports`・`test-results`・`surefire-it-reports` 配下の `.xml`）を含むディレクトリを1行ずつ出力する

## サマリを見る

```
junit-report summary --dir <results で得たパス>
```

Markdown 形式で合格・失敗・スキップ数と、失敗したテストの詳細を出力する。

## 失敗のみ表示

```
junit-report summary --dir <path> --failure
```

`--failure` フラグを付けると `## Failures` セクションのみ出力される。

## 標準出力・標準エラーを見る

```
# system-out を表示
junit-report output --path <summary の path> --sysout

# system-err を表示
junit-report output --path <summary の path> --syserr
```

`--sysout` と `--syserr` はどちらか一方のみ指定する。

---

Once you have the output, display the results to the user.

## If junit-report is not found

If the command is not found, inform the user:

> `junit-report` is not installed. Please build it first:
>
> ```
> cargo build --release -p junit-report
> # Then add the binary to your PATH, or run with the full path:
> # ./target/release/junit-report
> ```

## If no XML files are found

Inform the user that no JUnit XML report files were found in the searched directory,
and suggest running the test suite first (e.g. `mvn test` or `./gradlew test`).
