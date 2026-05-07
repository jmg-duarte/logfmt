use std::fs;
use std::path::PathBuf;
use std::process::Command;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

struct Logger {
    bin: &'static str,
    macro_path: &'static str,
}

const LOGGERS: &[Logger] = &[
    Logger { bin: "env_logger_default", macro_path: "log::info" },
    Logger { bin: "tracing_full",       macro_path: "tracing::info" },
    Logger { bin: "tracing_compact",    macro_path: "tracing::info" },
    Logger { bin: "tracing_pretty",     macro_path: "tracing::info" },
    Logger { bin: "tracing_json",       macro_path: "tracing::info" },
    Logger { bin: "tracing_log_bridge", macro_path: "log::info" },
];

struct Format {
    kind: &'static str,
    label: &'static str,
    snippet: &'static str,
}

const FORMATS: &[Format] = &[
    Format {
        kind: "display",
        label: "<code>{}</code> &mdash; <code>Display</code>",
        snippet: "<MACRO>!(\"hello {}, the answer is {}\", \"world\", 42);",
    },
    Format {
        kind: "debug",
        label: "<code>{:?}</code> &mdash; <code>Debug</code>",
        snippet: "let users = vec![\"alice\", \"bob\", \"charlie\"];\n<MACRO>!(\"users: {:?}\", users);",
    },
    Format {
        kind: "pretty_debug",
        label: "<code>{:#?}</code> &mdash; pretty <code>Debug</code>",
        snippet: "#[derive(Debug)]\nstruct User { name: &'static str, age: u32, roles: Vec<&'static str> }\n\nlet user = User { name: \"alice\", age: 30, roles: vec![\"admin\", \"editor\"] };\n<MACRO>!(\"user: {:#?}\", user);",
    },
    Format {
        kind: "named",
        label: "<code>{name}</code> &mdash; implicit named-argument capture (Rust 2021+)",
        snippet: "let name = \"alice\";\nlet age = 30;\n<MACRO>!(\"user {name} is {age} years old\");",
    },
    Format {
        kind: "other",
        label: "<code>{:x}</code>, <code>{:&gt;5}</code>, <code>{:.2}</code>, <code>{:08b}</code> &mdash; width, precision, hex, binary",
        snippet: "let n = 255u32;\n<MACRO>!(\"hex={:x}  padded={:>5}  precision={:.2}  binary={:08b}\", n, n, 3.14159, n);",
    },
];

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn highlight_rust(snippet: &str, ss: &SyntaxSet, syntax: &SyntaxReference, theme: &Theme) -> String {
    let mut h = HighlightLines::new(syntax, theme);
    let mut out = String::new();
    for line in LinesWithEndings::from(snippet) {
        let ranges = h.highlight_line(line, ss).expect("syntect highlight_line");
        out.push_str(
            &styled_line_to_highlighted_html(&ranges, IncludeBackground::No)
                .expect("syntect html"),
        );
    }
    out
}

fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() {
                let b = bytes[i];
                i += 1;
                if (0x40..=0x7e).contains(&b) {
                    break;
                }
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let template = fs::read_to_string(root.join("template.html"))
        .expect("read template.html");

    let mut build = Command::new("cargo");
    build.arg("build");
    for l in LOGGERS {
        build.arg("--bin").arg(l.bin);
    }
    let status = build.status().expect("spawn cargo build");
    assert!(status.success(), "cargo build failed");

    let bin_dir = root.join("target/debug");

    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let syntax = syntax_set
        .find_syntax_by_extension("rs")
        .expect("rust syntax")
        .clone();
    let theme = theme_set
        .themes
        .get("InspiredGitHub")
        .expect("InspiredGitHub theme")
        .clone();

    let mut cells = String::new();

    for (li, logger) in LOGGERS.iter().enumerate() {
        for (fi, fmt) in FORMATS.iter().enumerate() {
            let out = Command::new(bin_dir.join(logger.bin))
                .env("RUST_LOG", "info")
                .env("FMT_KIND", fmt.kind)
                .output()
                .expect("exec example bin");
            assert!(
                out.status.success(),
                "{} (FMT_KIND={}) exited non-zero",
                logger.bin, fmt.kind
            );

            let mut combined = Vec::new();
            combined.extend_from_slice(&out.stdout);
            combined.extend_from_slice(&out.stderr);
            let raw = String::from_utf8_lossy(&combined);
            let cleaned = strip_ansi(&raw);

            let snippet = fmt.snippet.replace("<MACRO>", logger.macro_path);
            let _ = (li, fi);
            // Initial state matches data-active-top=tracing-subscriber + data-active-sub=full.
            let hidden_attr = if logger.bin == "tracing_full" { "" } else { " hidden" };

            cells.push_str(&format!(
                "    <article class=\"cell\" data-logger=\"{logger}\" data-format=\"{format}\"{hidden}>\n",
                logger = logger.bin,
                format = fmt.kind,
                hidden = hidden_attr,
            ));
            cells.push_str(&format!("      <h3 class=\"cell-label\">{}</h3>\n", fmt.label));
            cells.push_str("      <div class=\"pair\">\n");
            cells.push_str(&format!(
                "        <pre class=\"rust\"><code>{}</code></pre>\n",
                highlight_rust(&snippet, &syntax_set, &syntax, &theme)
            ));
            cells.push_str(&format!(
                "        <pre class=\"output\"><code>{}</code></pre>\n",
                html_escape(cleaned.trim_end())
            ));
            cells.push_str("      </div>\n");
            cells.push_str("    </article>\n");
        }
    }

    let html = template.replace("{{cells}}", cells.trim_end());
    let out_path = root.join("index.html");
    fs::write(&out_path, html).expect("write index.html");
    println!("wrote {}", out_path.display());
}
