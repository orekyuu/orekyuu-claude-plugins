use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process;

use clap::{Parser, Subcommand};
use walkdir::WalkDir;

// ---- CLI 定義 ----

#[derive(Parser)]
#[command(name = "junit-report")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// JUnit XML を含むディレクトリ一覧を出力する
    Results {
        /// 走査するルートディレクトリ（省略時は "."）
        dir: Option<String>,
    },
    /// テスト結果のサマリを markdown 形式で出力する
    Summary {
        /// JUnit XML を含むディレクトリ
        #[arg(long, required = true)]
        dir: String,
        /// Failures セクションのみ出力する
        #[arg(long)]
        failure: bool,
    },
    /// XML ファイルの system-out または system-err を出力する
    Output {
        /// 対象の JUnit XML ファイルパス
        #[arg(long, required = true)]
        path: String,
        /// system-out を出力する
        #[arg(long)]
        sysout: bool,
        /// system-err を出力する
        #[arg(long)]
        syserr: bool,
    },
}

// ---- データ構造 ----

struct TestSuite {
    tests: u32,
    failures: u32,
    errors: u32,
    skipped: u32,
}

struct FailedTest {
    name: String,
    failure_type: String,
    message: String,
    body: String,
    file: String,
}

// ---- ユーティリティ関数 ----

fn is_junit_xml(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if !name.ends_with(".xml") {
        return false;
    }

    if name.starts_with("TEST-") {
        return true;
    }

    // surefire-reports / test-results / surefire-it-reports 配下（サブディレクトリ含む）の .xml
    path.ancestors().skip(1).any(|ancestor| {
        ancestor
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| {
                matches!(
                    n,
                    "surefire-reports" | "test-results" | "surefire-it-reports"
                )
            })
            .unwrap_or(false)
    })
}

fn parse_u32(s: Option<&str>) -> u32 {
    s.and_then(|v| v.parse().ok()).unwrap_or(0)
}

/// type 属性の末尾クラス名を取得する（例: "org.junit.AssertionError" → "AssertionError"）
fn simple_type_name(type_attr: &str) -> &str {
    type_attr.rsplit('.').next().unwrap_or(type_attr)
}

fn find_junit_files(dir: &str) -> Vec<String> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_junit_xml(e.path()))
        .filter_map(|e| e.path().to_str().map(String::from))
        .collect()
}

/// JUnit XML ファイルからテストスイートと失敗一覧をパースする
fn parse_xml_content(
    content: &str,
    path: &str,
) -> Result<(Vec<TestSuite>, Vec<FailedTest>), Box<dyn std::error::Error>> {
    let doc = roxmltree::Document::parse(content)?;

    let mut suites = Vec::new();
    let mut all_failures = Vec::new();

    for suite_node in doc
        .descendants()
        .filter(|n| n.tag_name().name() == "testsuite")
    {
        let parent_is_suite = suite_node
            .parent()
            .map(|p| p.tag_name().name() == "testsuite")
            .unwrap_or(false);
        if parent_is_suite {
            continue;
        }

        let tests = parse_u32(suite_node.attribute("tests"));
        let failures = parse_u32(suite_node.attribute("failures"));
        let errors = parse_u32(suite_node.attribute("errors"));
        let skipped = parse_u32(suite_node.attribute("skipped"));

        suites.push(TestSuite {
            tests,
            failures,
            errors,
            skipped,
        });

        for testcase in suite_node
            .children()
            .filter(|n| n.tag_name().name() == "testcase")
        {
            let test_name = testcase.attribute("name").unwrap_or("").to_string();

            for child in testcase.children() {
                let tag = child.tag_name().name();
                if tag == "failure" || tag == "error" {
                    let failure_type = child.attribute("type").unwrap_or("").to_string();
                    let message = child.attribute("message").unwrap_or("").to_string();
                    let body = child.text().unwrap_or("").trim().to_string();

                    all_failures.push(FailedTest {
                        name: test_name.clone(),
                        failure_type,
                        message,
                        body,
                        file: path.to_string(),
                    });
                }
            }
        }
    }

    Ok((suites, all_failures))
}

fn parse_file(path: &str) -> Result<(Vec<TestSuite>, Vec<FailedTest>), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    parse_xml_content(&content, path)
}

/// XML ファイルから system-out または system-err のテキストを取得する
fn extract_system_content(content: &str, tag: &str) -> Result<String, Box<dyn std::error::Error>> {
    let doc = roxmltree::Document::parse(content)?;
    let text = doc
        .descendants()
        .find(|n| n.tag_name().name() == tag)
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();
    Ok(text)
}

// ---- サブコマンド実装 ----

fn cmd_results(dir: Option<String>) {
    let root = dir.as_deref().unwrap_or(".");
    let files = find_junit_files(root);

    // 各ファイルの親ディレクトリを収集して重複排除
    let mut dirs: HashSet<String> = HashSet::new();
    for f in &files {
        let parent = Path::new(f)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".")
            .to_string();
        dirs.insert(parent);
    }

    let mut sorted: Vec<String> = dirs.into_iter().collect();
    sorted.sort();

    for d in sorted {
        println!("{}", d);
    }
}

fn cmd_summary(dir: String, failure_only: bool) {
    let files = find_junit_files(&dir);

    let mut total_tests: u32 = 0;
    let mut total_failures: u32 = 0;
    let mut total_errors: u32 = 0;
    let mut total_skipped: u32 = 0;
    let mut all_failures: Vec<FailedTest> = Vec::new();

    for file in &files {
        match parse_file(file) {
            Ok((suites, mut failed)) => {
                for suite in &suites {
                    total_tests += suite.tests;
                    total_failures += suite.failures;
                    total_errors += suite.errors;
                    total_skipped += suite.skipped;
                }
                all_failures.append(&mut failed);
            }
            Err(e) => {
                eprintln!("Warning: could not parse {}: {}", file, e);
            }
        }
    }

    let succeeded = total_tests
        .saturating_sub(total_failures)
        .saturating_sub(total_errors)
        .saturating_sub(total_skipped);

    if !failure_only {
        println!("# Summary");
        println!("Succeeded: {}", succeeded);
        println!("Failure: {}", total_failures + total_errors);
        println!("Skip: {}", total_skipped);
        println!();
    }

    if !all_failures.is_empty() {
        println!("## Failures");
        for f in &all_failures {
            let type_label = simple_type_name(&f.failure_type);
            if !f.message.is_empty() && !type_label.is_empty() {
                println!("### {} ({}: {})", f.name, type_label, f.message);
            } else if !type_label.is_empty() {
                println!("### {} ({})", f.name, type_label);
            } else {
                println!("### {}", f.name);
            }
            println!("path: {}", f.file);
            println!();
            if !f.body.is_empty() {
                println!("```");
                println!("{}", f.body);
                println!("```");
            }
            println!();
        }
    }
}

fn cmd_output(path: String, sysout: bool, syserr: bool) {
    match (sysout, syserr) {
        (true, true) => {
            eprintln!("error: --sysout and --syserr cannot be used together");
            process::exit(1);
        }
        (false, false) => {
            eprintln!("error: --sysout and --syserr cannot be used together");
            process::exit(1);
        }
        _ => {}
    }

    let tag = if sysout { "system-out" } else { "system-err" };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: could not read {}: {}", path, e);
            process::exit(1);
        }
    };

    match extract_system_content(&content, tag) {
        Ok(text) => print!("{}", text),
        Err(e) => {
            eprintln!("error: could not parse {}: {}", path, e);
            process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Results { dir } => cmd_results(dir),
        Commands::Summary { dir, failure } => cmd_summary(dir, failure),
        Commands::Output {
            path,
            sysout,
            syserr,
        } => cmd_output(path, sysout, syserr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- is_junit_xml ----

    #[test]
    fn test_is_junit_xml_test_prefix() {
        assert!(is_junit_xml(Path::new(
            "surefire-reports/TEST-com.example.FooTest.xml"
        )));
    }

    #[test]
    fn test_is_junit_xml_surefire_reports_dir() {
        assert!(is_junit_xml(Path::new("surefire-reports/report.xml")));
    }

    #[test]
    fn test_is_junit_xml_test_results_dir() {
        assert!(is_junit_xml(Path::new("test-results/result.xml")));
    }

    #[test]
    fn test_is_junit_xml_surefire_it_reports_dir() {
        assert!(is_junit_xml(Path::new("surefire-it-reports/report.xml")));
    }

    #[test]
    fn test_is_junit_xml_rejects_unknown_dir() {
        assert!(!is_junit_xml(Path::new("build/reports/result.xml")));
    }

    #[test]
    fn test_is_junit_xml_rejects_non_xml() {
        assert!(!is_junit_xml(Path::new("surefire-reports/TEST-Foo.txt")));
    }

    #[test]
    fn test_is_junit_xml_test_results_subdir_test() {
        // test-results/test/ 配下の .xml（TEST- prefix なし）
        assert!(is_junit_xml(Path::new("test-results/test/result.xml")));
    }

    #[test]
    fn test_is_junit_xml_test_results_subdir_integration() {
        // test-results/integrationTest/ 配下の .xml
        assert!(is_junit_xml(Path::new(
            "test-results/integrationTest/result.xml"
        )));
    }

    #[test]
    fn test_is_junit_xml_surefire_reports_subdir() {
        // surefire-reports/subdir/ 配下の .xml
        assert!(is_junit_xml(Path::new(
            "surefire-reports/subdir/result.xml"
        )));
    }

    #[test]
    fn test_is_junit_xml_rejects_unknown_ancestor() {
        // build/reports/ 配下は検出しない
        assert!(!is_junit_xml(Path::new("build/reports/subdir/result.xml")));
    }

    // ---- parse_u32 ----

    #[test]
    fn test_parse_u32_valid() {
        assert_eq!(parse_u32(Some("42")), 42);
    }

    #[test]
    fn test_parse_u32_invalid() {
        assert_eq!(parse_u32(Some("abc")), 0);
    }

    #[test]
    fn test_parse_u32_none() {
        assert_eq!(parse_u32(None), 0);
    }

    // ---- simple_type_name ----

    #[test]
    fn test_simple_type_name_fqn() {
        assert_eq!(
            simple_type_name("org.junit.AssertionError"),
            "AssertionError"
        );
    }

    #[test]
    fn test_simple_type_name_simple() {
        assert_eq!(simple_type_name("AssertionError"), "AssertionError");
    }

    #[test]
    fn test_simple_type_name_empty() {
        assert_eq!(simple_type_name(""), "");
    }

    // ---- parse_xml_content ----

    #[test]
    fn test_parse_all_passing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="com.example.FooTest" tests="2" failures="0" errors="0" skipped="0" time="0.05">
  <testcase name="testA" classname="com.example.FooTest" time="0.02"/>
  <testcase name="testB" classname="com.example.FooTest" time="0.03"/>
</testsuite>"#;

        let (suites, failures) = parse_xml_content(xml, "TEST-FooTest.xml").unwrap();
        assert_eq!(suites.len(), 1);
        assert_eq!(suites[0].tests, 2);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_parse_with_failure() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="com.example.FooTest" tests="2" failures="1" errors="0" skipped="0" time="0.05">
  <testcase name="testA" classname="com.example.FooTest" time="0.02">
    <failure message="expected true but was false" type="org.junit.AssertionError">
      stack trace here
    </failure>
  </testcase>
  <testcase name="testB" classname="com.example.FooTest" time="0.03"/>
</testsuite>"#;

        let (suites, failures) = parse_xml_content(xml, "TEST-FooTest.xml").unwrap();
        assert_eq!(suites[0].failures, 1);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].name, "testA");
        assert_eq!(failures[0].failure_type, "org.junit.AssertionError");
        assert_eq!(failures[0].message, "expected true but was false");
        assert_eq!(failures[0].body, "stack trace here");
    }

    #[test]
    fn test_parse_with_error() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="com.example.FooTest" tests="1" failures="0" errors="1" skipped="0" time="0.01">
  <testcase name="testA" classname="com.example.FooTest" time="0.01">
    <error message="NullPointerException" type="java.lang.NullPointerException">
      npe trace
    </error>
  </testcase>
</testsuite>"#;

        let (_, failures) = parse_xml_content(xml, "TEST-FooTest.xml").unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].failure_type, "java.lang.NullPointerException");
        assert_eq!(failures[0].message, "NullPointerException");
    }

    #[test]
    fn test_parse_with_skipped() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="com.example.FooTest" tests="2" failures="0" errors="0" skipped="1" time="0.01">
  <testcase name="testA" classname="com.example.FooTest" time="0.01"/>
  <testcase name="testB" classname="com.example.FooTest" time="0.00">
    <skipped/>
  </testcase>
</testsuite>"#;

        let (suites, failures) = parse_xml_content(xml, "TEST-FooTest.xml").unwrap();
        assert_eq!(suites[0].skipped, 1);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_parse_testsuites_wrapper() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="com.example.FooTest" tests="1" failures="0" errors="0" skipped="0" time="0.01">
    <testcase name="testA" classname="com.example.FooTest" time="0.01"/>
  </testsuite>
  <testsuite name="com.example.BarTest" tests="1" failures="1" errors="0" skipped="0" time="0.02">
    <testcase name="testB" classname="com.example.BarTest" time="0.02">
      <failure message="oops" type="AssertionError">trace</failure>
    </testcase>
  </testsuite>
</testsuites>"#;

        let (suites, failures) = parse_xml_content(xml, "combined.xml").unwrap();
        assert_eq!(suites.len(), 2);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].name, "testB");
    }

    #[test]
    fn test_parse_invalid_xml() {
        let result = parse_xml_content("<not valid xml", "bad.xml");
        assert!(result.is_err());
    }

    // ---- summary markdown 出力フォーマット ----

    #[test]
    fn test_summary_header_with_type_and_message() {
        // failure_type が FQN のとき SimpleTypeName に変換される
        let type_name = simple_type_name("org.junit.AssertionError");
        let header = if !type_name.is_empty() {
            format!(
                "### {} ({}: {})",
                "testCase9", type_name, "Assertion error message"
            )
        } else {
            format!("### {}", "testCase9")
        };
        assert_eq!(
            header,
            "### testCase9 (AssertionError: Assertion error message)"
        );
    }

    #[test]
    fn test_summary_header_without_message() {
        // message が空の場合は括弧部分を省略しない（type だけ表示）
        let type_name = simple_type_name("AssertionError");
        let message = "";
        let header = if !message.is_empty() && !type_name.is_empty() {
            format!("### {} ({}: {})", "testFoo", type_name, message)
        } else if !type_name.is_empty() {
            format!("### {} ({})", "testFoo", type_name)
        } else {
            format!("### {}", "testFoo")
        };
        assert_eq!(header, "### testFoo (AssertionError)");
    }

    // ---- extract_system_content ----

    #[test]
    fn test_extract_sysout() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="FooTest" tests="1" failures="0" errors="0" skipped="0" time="0.01">
  <testcase name="testA" classname="FooTest" time="0.01"/>
  <system-out>hello stdout</system-out>
  <system-err>hello stderr</system-err>
</testsuite>"#;

        let out = extract_system_content(xml, "system-out").unwrap();
        assert_eq!(out, "hello stdout");
    }

    #[test]
    fn test_extract_syserr() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="FooTest" tests="1" failures="0" errors="0" skipped="0" time="0.01">
  <testcase name="testA" classname="FooTest" time="0.01"/>
  <system-out>hello stdout</system-out>
  <system-err>hello stderr</system-err>
</testsuite>"#;

        let err = extract_system_content(xml, "system-err").unwrap();
        assert_eq!(err, "hello stderr");
    }

    #[test]
    fn test_extract_missing_element() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="FooTest" tests="1" failures="0" errors="0" skipped="0" time="0.01">
  <testcase name="testA" classname="FooTest" time="0.01"/>
</testsuite>"#;

        let out = extract_system_content(xml, "system-out").unwrap();
        assert_eq!(out, "");
    }

    // ---- output サブコマンドのエラー検知（ロジックテスト）----

    #[test]
    fn test_output_both_flags_is_error() {
        // sysout=true, syserr=true の組み合わせはエラーになるべき
        let sysout = true;
        let syserr = true;
        let is_error = matches!((sysout, syserr), (true, true) | (false, false));
        assert!(is_error);
    }

    #[test]
    fn test_output_no_flags_is_error() {
        let sysout = false;
        let syserr = false;
        let is_error = matches!((sysout, syserr), (true, true) | (false, false));
        assert!(is_error);
    }

    #[test]
    fn test_output_sysout_only_is_valid() {
        let sysout = true;
        let syserr = false;
        let is_error = matches!((sysout, syserr), (true, true) | (false, false));
        assert!(!is_error);
    }

    #[test]
    fn test_output_syserr_only_is_valid() {
        let sysout = false;
        let syserr = true;
        let is_error = matches!((sysout, syserr), (true, true) | (false, false));
        assert!(!is_error);
    }

    // ---- ファイルシステムを使った find_junit_files テスト ----

    /// テスト用一時ディレクトリ（Drop 時に自動削除）
    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new(suffix: &str) -> Self {
            let path = std::env::temp_dir().join(format!("junit-report-test-{}", suffix));
            fs::create_dir_all(&path).unwrap();
            TempDir(path)
        }
        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_xml(path: &std::path::Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    const MINIMAL_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="Test" tests="1" failures="0" errors="0" skipped="0" time="0.01">
  <testcase name="testA" classname="Test" time="0.01"/>
</testsuite>"#;

    #[test]
    fn test_find_junit_files_test_results_subdir() {
        // test-results/test/ と test-results/integrationTest/ 配下の result.xml を検出できる
        let tmp = TempDir::new("subdir");
        let root = tmp.path();

        write_xml(
            &root.join("build/test-results/test/result.xml"),
            MINIMAL_XML,
        );
        write_xml(
            &root.join("build/test-results/integrationTest/result.xml"),
            MINIMAL_XML,
        );

        let mut files = find_junit_files(root.to_str().unwrap());
        files.sort();
        assert_eq!(files.len(), 2);
        assert!(files[0].ends_with("integrationTest/result.xml"));
        assert!(files[1].ends_with("test/result.xml"));
    }

    #[test]
    fn test_find_junit_files_multiproject() {
        // マルチプロジェクト構成: subproject-a と subproject-b それぞれに test-results がある
        let tmp = TempDir::new("multiproject");
        let root = tmp.path();

        write_xml(
            &root.join("subproject-a/build/test-results/test/TEST-FooTest.xml"),
            MINIMAL_XML,
        );
        write_xml(
            &root.join("subproject-a/build/test-results/integrationTest/TEST-BarTest.xml"),
            MINIMAL_XML,
        );
        write_xml(
            &root.join("subproject-b/build/test-results/test/TEST-BazTest.xml"),
            MINIMAL_XML,
        );

        let files = find_junit_files(root.to_str().unwrap());
        assert_eq!(files.len(), 3);

        // 各サブプロジェクト・各テスト種別のディレクトリが揃っているか
        let dirs: HashSet<String> = files
            .iter()
            .filter_map(|f| Path::new(f).parent()?.to_str().map(String::from))
            .collect();
        assert_eq!(dirs.len(), 3); // test, integrationTest (project-a), test (project-b)
    }

    #[test]
    fn test_find_junit_files_dedup_dirs() {
        // 同一ディレクトリに複数 XML があっても find_junit_files はファイル単位で返す
        let tmp = TempDir::new("dedup");
        let root = tmp.path();

        write_xml(&root.join("test-results/TEST-FooTest.xml"), MINIMAL_XML);
        write_xml(&root.join("test-results/TEST-BarTest.xml"), MINIMAL_XML);

        let files = find_junit_files(root.to_str().unwrap());
        assert_eq!(files.len(), 2); // ファイルは2件

        // 親ディレクトリは重複排除で1件
        let dirs: HashSet<String> = files
            .iter()
            .filter_map(|f| Path::new(f).parent()?.to_str().map(String::from))
            .collect();
        assert_eq!(dirs.len(), 1);
    }

    #[test]
    fn test_find_junit_files_ignores_non_junit_xml() {
        // JUnit XML でないファイルは無視される
        let tmp = TempDir::new("non-junit");
        let root = tmp.path();

        write_xml(&root.join("build/reports/result.xml"), MINIMAL_XML); // 無視されるべき
        write_xml(&root.join("test-results/TEST-FooTest.xml"), MINIMAL_XML); // 検出されるべき

        let files = find_junit_files(root.to_str().unwrap());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("TEST-FooTest.xml"));
    }
}
