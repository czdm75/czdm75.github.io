use std::{fs, path::Path, process::Command};

fn run(input: &str, output: &Path, mode: &str) -> serde_json::Value {
    let result = Command::new(env!("CARGO_BIN_EXE_typst-ssg"))
        .args([
            "--input",
            input,
            "--output",
            output.to_str().unwrap(),
            "--code-renderer",
            mode,
            "--report",
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&fs::read(output.join("ssg-report.json")).unwrap()).unwrap()
}

#[test]
fn bundle_metadata_headings_and_highlight_modes() {
    let temp = std::env::temp_dir().join(format!("typst-ssg-test-{}", std::process::id()));
    let compiled = run("content", &temp.join("compiled"), "typst");
    let client = run("content", &temp.join("client"), "highlight-js");
    assert!(compiled["pages"].as_array().unwrap().len() > 6);
    let page = compiled["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|page| page["path"] == "cs/io/java-nio-1/index.html")
        .unwrap();
    assert_eq!(page["metadata"][0]["year"], 2018);
    assert_eq!(page["toc"][0]["level"], 1);
    assert_eq!(page["toc"][0]["text"], "Java NIO: 应用");
    assert!(
        page["code"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["language"] == "java")
    );
    assert_eq!(compiled["pages"], client["pages"]);
    for page in compiled["pages"].as_array().unwrap() {
        let sidecar = Path::new(page["path"].as_str().unwrap()).with_extension("json");
        let saved: serde_json::Value =
            serde_json::from_slice(&fs::read(temp.join("compiled").join(sidecar)).unwrap())
                .unwrap();
        assert_eq!(&saved, page);
    }
    let route = "cs/io/java-nio-1/index.html";
    let compiled_html = fs::read_to_string(temp.join("compiled").join(route)).unwrap();
    let client_html = fs::read_to_string(temp.join("client").join(route)).unwrap();
    assert!(compiled_html.contains("color:"));
    assert!(!client_html.contains("color:"));
    assert!(client_html.contains("data-lang=\"java\""));

    let bare = run(
        "tests/fixtures/bundle.typ",
        &temp.join("bare"),
        "highlight-js",
    );
    let page = &bare["pages"][0];
    assert_eq!(page["type"], "page");
    assert_eq!(page["toc"].as_array().unwrap().len(), 2);
    assert_eq!(page["metadata"][0]["kind"], "example");
    assert_eq!(page["code"][0]["language"], "rust");
    assert!(
        page["code"][0]["text"]
            .as_str()
            .unwrap()
            .contains("fn main()")
    );
    let code = page["code"].as_array().unwrap();
    assert!(code[0]["block"].as_bool().unwrap());
    assert!(
        code.iter()
            .any(|c| c["text"] == "inline" && c["block"] == false)
    );
    assert!(
        code.iter()
            .any(|c| c["text"] == "first\nsecond" && c["block"] == true)
    );
    assert!(
        code.iter()
            .any(|c| c["text"] == "original source" && c["language"] == "custom")
    );
    let html = fs::read_to_string(temp.join("bare/bare/index.html")).unwrap();
    assert!(html.contains("Rendered replacement"));
    assert!(!html.contains("original source"));
    assert_eq!(page["metadata"][1].as_str().unwrap().split('-').count(), 3);
    let discovered = run(
        "tests/fixtures/articles",
        &temp.join("discovered"),
        "highlight-js",
    );
    let pages = discovered["pages"].as_array().unwrap();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0]["path"], "bare/index.html");
    assert_eq!(pages[1]["path"], "nested/other/index.html");
    assert_eq!(pages[1]["metadata"], serde_json::json!([{}]));
    assert_eq!(pages[1]["toc"][0]["text"], "Another bare article");
    assert!(!Path::new(".ssg-entry.typ").exists());
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn first_pass_exports_html_and_metadata_without_debug_report() {
    let output = std::env::temp_dir().join(format!("typst-ssg-first-pass-{}", std::process::id()));
    let result = Command::new(env!("CARGO_BIN_EXE_typst-ssg"))
        .args([
            "--input",
            "tests/fixtures/articles",
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    for route in ["bare/index", "nested/other/index"] {
        let html = fs::read_to_string(output.join(format!("{route}.html"))).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"));
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join(format!("{route}.json"))).unwrap())
                .unwrap();
        assert_eq!(metadata["path"], format!("{route}.html"));
        assert_eq!(metadata["type"], "page");
        assert!(!metadata["toc"].as_array().unwrap().is_empty());
    }
    assert!(!output.join("ssg-report.json").exists());
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn first_pass_serializes_native_metadata() {
    let output = std::env::temp_dir().join(format!("typst-ssg-native-{}", std::process::id()));
    let result = Command::new(env!("CARGO_BIN_EXE_typst-ssg"))
        .args([
            "--input",
            "tests/fixtures/non-json.typ",
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("index.json")).unwrap()).unwrap();
    assert_eq!(metadata["metadata"][0], "(..) => ..");
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn typst_rendering_options() {
    let root = std::env::temp_dir().join(format!("typst-ssg-render-{}", std::process::id()));
    let run = |name: &str, args: &[&str]| {
        let output = root.join(name);
        let result = Command::new(env!("CARGO_BIN_EXE_typst-ssg"))
            .args([
                "--input",
                "tests/fixtures/articles",
                "--output",
                output.to_str().unwrap(),
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        fs::read_to_string(output.join("bare/index.html")).unwrap()
    };
    let svg = run(
        "svg",
        &[
            "--math",
            "svg",
            "--code-theme",
            "tests/fixtures/test.tmTheme",
        ],
    );
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("<math"));
    assert!(svg.contains("color: #123456"));
    let client = run(
        "client",
        &[
            "--math",
            "mathml",
            "--code-renderer",
            "highlight-js",
            "--code-theme",
            "future.css",
        ],
    );
    assert!(client.contains("<math"));
    assert!(!client.contains("<svg"));
    assert!(!client.contains("color:"));
    assert!(client.contains("data-lang=\"rust\""));
    assert!(!client.contains("<script"));
    fs::remove_dir_all(root).unwrap();
}
