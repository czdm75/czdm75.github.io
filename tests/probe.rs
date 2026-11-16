use std::{fs, path::Path, process::Command};

#[test]
fn custom_theme_loads_and_updates_without_rebuilding_the_binary() {
    let root =
        std::env::temp_dir().join(format!("typst-ssg-runtime-package-{}", std::process::id()));
    let package = root.join("theme/custom");
    fs::create_dir_all(package.join("templates")).unwrap();
    fs::create_dir_all(root.join("content")).unwrap();
    fs::write(root.join("content/post.typ"), "= Runtime article").unwrap();
    fs::write(
        package.join("theme.toml"),
        r#"
template-dirs = ["templates"]
stylesheets = ["_theme/custom/main.css"]
[assets]
"_theme/custom/main.css" = "main.css"
"_theme/custom/image.bin" = "image.bin"
[defaults.toolbar]
enabled = true
title = "Default toolbar"
"#,
    )
    .unwrap();
    let template = "<!doctype html><html><head>{{{head_html}}}{{#each theme.stylesheets}}<link rel=\"stylesheet\" href=\"{{this}}\">{{/each}}</head><body>Version 1 {{#if theme.options.toolbar.enabled}}<aside>{{theme.options.toolbar.title}}</aside>{{/if}} {{{body_html}}}</body></html>";
    fs::write(package.join("templates/page.hbs"), template).unwrap();
    fs::write(package.join("main.css"), "/* version 1 */").unwrap();
    fs::write(package.join("image.bin"), [0, 255, 128]).unwrap();
    let run = |args: &[&str]| {
        let result = Command::new(env!("CARGO_BIN_EXE_typst-ssg"))
            .current_dir(&root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    };
    run(&[
        "--theme=custom",
        "--output=first",
        "--theme-option=toolbar.title=\"<Custom>\"",
    ]);
    let first = fs::read_to_string(root.join("first/post/index.html")).unwrap();
    assert!(first.contains("Version 1"));
    assert!(first.contains("<aside>&lt;Custom&gt;</aside>"));
    assert!(first.contains("Runtime article"));
    assert!(first.contains("../_theme/custom/main.css"));
    assert_eq!(
        fs::read(root.join("first/_theme/custom/image.bin")).unwrap(),
        [0, 255, 128]
    );

    // The exact same binary reads an explicit package directory and its latest files.
    fs::rename(&package, root.join("skin")).unwrap();
    fs::write(
        root.join("skin/templates/page.hbs"),
        template.replace("Version 1", "Version 2"),
    )
    .unwrap();
    fs::write(root.join("skin/main.css"), "/* version 2 */").unwrap();
    run(&[
        "--theme=unregistered",
        "--theme-dir=skin",
        "--output=second",
        "--theme-option=toolbar.enabled=false",
    ]);
    let second = fs::read_to_string(root.join("second/post/index.html")).unwrap();
    assert!(second.contains("Version 2"));
    assert!(!second.contains("<aside>"));
    assert!(!second.contains("Version 1"));
    assert_eq!(
        fs::read_to_string(root.join("second/_theme/custom/main.css")).unwrap(),
        "/* version 2 */"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn toml_config_builds_with_relative_paths_and_cli_theme_overrides() {
    let root = std::env::temp_dir().join(format!("typst-ssg-toml-build-{}", std::process::id()));
    fs::create_dir_all(root.join("settings/articles")).unwrap();
    fs::write(
        root.join("settings/articles/post.typ"),
        "= Config article\n\n== Child\n\nContent.",
    )
    .unwrap();
    fs::write(
        root.join("settings/site.toml"),
        r#"
input = "articles"
output = "build"
report = true
[theme]
name = "mdbook"
[theme.options.toc]
enabled = false
mode = "embedded"
"#,
    )
    .unwrap();
    let run = |args: &[&str]| {
        let result = Command::new(env!("CARGO_BIN_EXE_typst-ssg"))
            .current_dir(&root)
            .arg("--config=settings/site.toml")
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    };
    run(&[]);
    let hidden = fs::read_to_string(root.join("settings/build/post/index.html")).unwrap();
    assert!(hidden.contains("_theme/mdbook/variables.css"));
    assert!(hidden.contains("id=\"ssg-heading-1\""));
    assert!(!hidden.contains("class=\"on-this-page\""));
    assert!(root.join("settings/build/ssg-report.json").exists());
    assert!(!root.join("build").exists());
    run(&["--output=override", "--toc=true", "--report=false"]);
    let visible = fs::read_to_string(root.join("override/post/index.html")).unwrap();
    assert!(visible.contains("class=\"on-this-page\""));
    assert!(!visible.contains("id=\"toc\""));
    assert!(!root.join("override/ssg-report.json").exists());
    fs::remove_dir_all(root).unwrap();
}

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
    let untitled = compiled["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|page| page["path"] == "etc/tiktok-in-my-memory/index.html")
        .unwrap();
    assert_eq!(untitled["title"], "关于 TikTok 案的几句杂谈");
    let html =
        fs::read_to_string(temp.join("compiled/etc/tiktok-in-my-memory/index.html")).unwrap();
    assert!(html.contains("<title>关于 TikTok 案的几句杂谈</title>"));
    for name in ["计算机系统", "读书笔记"] {
        assert!(html.contains(&format!("<details open><summary>{name}</summary>")));
    }
    for name in [
        "I/O 与网络",
        "编程语言",
        "分布式系统与大数据",
        "数学与统计",
        "算法导论",
        "写给大忙人看的 Java 核心技术",
        "深入理解 Java 虚拟机",
        "Scala 编程",
        "设计数据密集型应用",
        "杂谈",
    ] {
        assert!(
            html.contains(&format!("<details><summary>{name}</summary>")),
            "{name}"
        );
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
    assert_eq!(pages[0]["title"], "Bare document");
    assert_eq!(pages[1]["title"], "Another bare article");
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
        assert!(html.contains("id=\"grid-wrapper\""));
        assert!(html.contains("id=\"toc\""));
        let title = if route == "bare/index" {
            "Bare document"
        } else {
            "Another bare article"
        };
        assert!(html.contains(&format!("<title>{title}</title>")));
        assert_eq!(html.matches("<title>").count(), 1);
        assert!(!output.join(format!("{route}.json")).exists());
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
            "--report",
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
        serde_json::from_slice(&fs::read(output.join("ssg-report.json")).unwrap()).unwrap();
    assert_eq!(metadata["pages"][0]["metadata"][0], "(..) => ..");
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn typst_rendering_options() {
    let root = std::env::temp_dir().join(format!("typst-ssg-render-{}", std::process::id()));
    let run = |name: &str, args: &[&str]| {
        let output = root.join(name);
        let result = Command::new(env!("CARGO_BIN_EXE_typst-ssg"))
            .args([
                "--theme",
                "default",
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
    assert!(client.contains("_theme/default/main.js"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn raw_mode_exports_html_and_page_metadata() {
    let output = std::env::temp_dir().join(format!("typst-ssg-raw-{}", std::process::id()));
    let result = Command::new(env!("CARGO_BIN_EXE_typst-ssg"))
        .args([
            "--input",
            "tests/fixtures/articles",
            "--mode",
            "raw",
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
        assert!(!html.contains("_theme/default"));
        assert!(!html.contains("id=\"grid-wrapper\""));
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join(format!("{route}.json"))).unwrap())
                .unwrap();
        assert_eq!(metadata["path"], format!("{route}.html"));
        assert!(!metadata["toc"].as_array().unwrap().is_empty());
    }
    assert!(!output.join("index.html").exists());
    assert!(!output.join("_theme").exists());
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn summary_selects_orders_and_renames_navigation() {
    let root = std::env::temp_dir().join(format!("typst-ssg-summary-{}", std::process::id()));
    fs::create_dir_all(root.join("content/auto/nested")).unwrap();
    fs::write(
        root.join("content/a.typ"),
        "#let meta = (title: \"Article A\")\n= Body A",
    )
    .unwrap();
    fs::write(
        root.join("content/b.typ"),
        "#let meta = (title: \"Article B\")\n= Body B",
    )
    .unwrap();
    fs::write(root.join("content/auto/z.typst"), "= Z").unwrap();
    fs::write(
        root.join("content/auto/a.typ"),
        "#let meta = (title: \"Auto A\")\n= A",
    )
    .unwrap();
    fs::write(root.join("content/auto/nested/c.typ"), "= C").unwrap();
    fs::write(root.join("content/c.typ"), "= Separate group").unwrap();
    // Unselected files must never be evaluated.
    fs::write(
        root.join("content/ignored.typ"),
        "#panic(\"excluded article\")",
    )
    .unwrap();
    fs::write(
        root.join("content/summary.typ"),
        r#"
#let summary = (
  (title: "Manual <group>", children: (
    (path: "b.typ", title: "Navigation B"),
    (path: "auto/", title: "Automatic", collapsed: true),
    "a.typ",
  )),
  (title: "Manual <group>", collapsed: true, children: ("c.typ",)),
)
"#,
    )
    .unwrap();
    let build = || {
        Command::new(env!("CARGO_BIN_EXE_typst-ssg"))
            .current_dir(&root)
            .args(["--report"])
            .output()
            .unwrap()
    };
    let result = build();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("public/ssg-report.json")).unwrap()).unwrap();
    let pages = report["pages"].as_array().unwrap();
    assert_eq!(pages.len(), 6);
    let page = pages.iter().find(|p| p["path"] == "b/index.html").unwrap();
    assert_eq!(page["title"], "Article B");
    assert_eq!(page["navigation"]["title"], "Navigation B");
    assert_eq!(
        page["metadata"],
        serde_json::json!([{ "title": "Article B" }])
    );
    for route in ["index.html", "b/index.html", "auto/nested/c/index.html"] {
        let html = fs::read_to_string(root.join("public").join(route)).unwrap();
        let start = html.find("<ul class=\"article-tree\">").unwrap();
        let tree = &html[start..html[start..].find("</nav>").unwrap() + start];
        let labels = [
            "Manual &lt;group&gt;",
            "Navigation B",
            "Automatic",
            "Auto A",
            "nested",
            ">C</a>",
            ">Z</a>",
            "Article A",
        ];
        let mut previous = 0;
        for label in labels {
            let index = tree
                .find(label)
                .unwrap_or_else(|| panic!("missing {label}: {tree}"));
            assert!(index >= previous, "wrong order for {label}: {tree}");
            previous = index;
        }
        assert!(!tree.contains("ignored"));
        assert!(tree.contains("<details><summary>Automatic</summary>"));
        assert!(tree.contains("<details open><summary>nested</summary>"));
        assert!(tree.contains("<details open><summary>Manual &lt;group&gt;</summary>"));
        assert!(tree.contains("<details><summary>Manual &lt;group&gt;</summary>"));
        assert_eq!(
            tree.matches("<summary>Manual &lt;group&gt;</summary>")
                .count(),
            2
        );
    }
    let index = fs::read_to_string(root.join("public/index.html")).unwrap();
    let listing = &index[index.find("class=\"index-group\"").unwrap()..];
    assert!(listing.find("Navigation B").unwrap() < listing.find("Automatic").unwrap());
    assert!(listing.find("Automatic").unwrap() < listing.rfind("Article A").unwrap());
    assert!(!root.join("public/summary").exists());
    assert!(!root.join("public/ignored").exists());

    for (summary, expected) in [
        (
            "#let summary = (\"a.typ\", \"b.typ\", \"a.typ\")",
            "more than once",
        ),
        ("#let summary = (\"missing.typ\",)", "path not found"),
        (
            "#let summary = ((path: \"auto\", collapsed: \"yes\"),)",
            "collapsed must be a boolean",
        ),
        (
            "#let summary = ((title: 123, children: (\"a.typ\",)),)",
            "title must be a string",
        ),
    ] {
        fs::write(root.join("content/summary.typ"), summary).unwrap();
        let result = build();
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    // The root shorthand includes all files while reserving summary.typ.
    fs::remove_file(root.join("content/ignored.typ")).unwrap();
    fs::write(root.join("content/summary.typ"), "#let summary = (\".\",)").unwrap();
    let result = build();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    // Without a summary, discovery still works with the original metadata.
    fs::remove_file(root.join("content/summary.typ")).unwrap();
    let result = build();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("public/ssg-report.json")).unwrap()).unwrap();
    assert_eq!(report["pages"].as_array().unwrap().len(), 6);
    assert!(
        report["pages"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p.get("navigation").is_none())
    );
    fs::remove_dir_all(root).unwrap();
}
