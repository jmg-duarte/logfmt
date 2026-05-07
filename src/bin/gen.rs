use std::fs;
use std::path::PathBuf;
use std::process::Command;

const EXAMPLES: &[&str] = &[
    "env_logger_default",
    "tracing_full",
    "tracing_compact",
    "tracing_pretty",
    "tracing_json",
    "tracing_log_bridge",
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
    for name in EXAMPLES {
        build.arg("--bin").arg(name);
    }
    let status = build.status().expect("spawn cargo build");
    assert!(status.success(), "cargo build failed");

    let bin_dir = root.join("target/debug");
    let mut html = template;

    for name in EXAMPLES {
        let source = fs::read_to_string(root.join(format!("src/bin/{name}.rs")))
            .expect("read example source");

        let out = Command::new(bin_dir.join(name))
            .env("RUST_LOG", "info")
            .output()
            .expect("exec example bin");

        let mut combined = Vec::new();
        combined.extend_from_slice(&out.stdout);
        combined.extend_from_slice(&out.stderr);
        let raw = String::from_utf8_lossy(&combined);
        let cleaned = strip_ansi(&raw);

        let src_placeholder = format!("{{{{{}.source}}}}", name);
        let out_placeholder = format!("{{{{{}.output}}}}", name);
        html = html.replace(&src_placeholder, &html_escape(source.trim_end()));
        html = html.replace(&out_placeholder, &html_escape(cleaned.trim_end()));
    }

    let out_path = root.join("index.html");
    fs::write(&out_path, html).expect("write index.html");
    println!("wrote {}", out_path.display());
}
