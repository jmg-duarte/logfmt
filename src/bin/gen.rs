use std::fs;
use std::path::PathBuf;
use std::process::Command;

use syntect::highlighting::ThemeSet;
use syntect::html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

const SYN_CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "syn-" };

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
    Format {
        kind: "fields",
        label: "<code>info!(field = value)</code>, <code>?value</code> (Debug), <code>%value</code> (Display) &mdash; structured fields with sigils",
        // <KV_SEP> renders as ',' for tracing and ';' for log (kv requires a semicolon).
        // <F_DBG> / <F_DISP> render the field-with-sigil form for the active macro family
        // (tracing puts the sigil before the value; log puts it before the equals).
        snippet: "<MACRO>!(answer = 42<KV_SEP> \"plain value\");\n<MACRO>!(<F_DBG><KV_SEP> \"? sigil = Debug\");\n<MACRO>!(<F_DISP><KV_SEP> \"% sigil = Display\");",
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

fn highlight_rust(snippet: &str, ss: &SyntaxSet, syntax: &SyntaxReference) -> String {
    let mut g = ClassedHTMLGenerator::new_with_class_style(syntax, ss, SYN_CLASS_STYLE);
    for line in LinesWithEndings::from(snippet) {
        g.parse_html_for_line_which_includes_newline(line)
            .expect("syntect classed html");
    }
    g.finalize()
}

/// Prepend `parent` to every selector in `css` (handling grouped selectors).
/// Assumes well-formed CSS with top-level rules — adequate for syntect output.
fn scope_css(css: &str, parent: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(brace) = rest.find('{') {
        let (sel_part, after_brace) = rest.split_at(brace);
        let leading_ws_len = sel_part.len() - sel_part.trim_start().len();
        let leading = &sel_part[..leading_ws_len];
        let sel = sel_part.trim();

        out.push_str(leading);
        if sel.is_empty() {
            // Likely a stray brace; emit unchanged and stop.
            out.push_str(after_brace);
            return out;
        }
        let scoped: Vec<String> = sel
            .split(',')
            .map(|s| format!("{} {}", parent, s.trim()))
            .collect();
        out.push_str(&scoped.join(",\n"));
        out.push(' ');

        match after_brace.find('}') {
            Some(end) => {
                let (rule_body, after_rule) = after_brace.split_at(end + 1);
                out.push_str(rule_body);
                rest = after_rule;
            }
            None => {
                out.push_str(after_brace);
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

fn build_footer(root: &std::path::Path) -> String {
    if let Ok(owner) = std::env::var("GITHUB_REPOSITORY_OWNER") {
        if !owner.is_empty() {
            let owner = html_escape(&owner);
            return format!(
                "  <footer class=\"site-footer\">by <a href=\"https://github.com/{owner}\">{owner}</a></footer>\n"
            );
        }
    }
    if let Ok(out) = Command::new("git")
        .args(["log", "-1", "--pretty=format:%h\t%an"])
        .current_dir(root)
        .output()
    {
        if out.status.success() {
            if let Ok(line) = std::str::from_utf8(&out.stdout) {
                if let Some((hash, author)) = line.trim().split_once('\t') {
                    if !hash.is_empty() && !author.is_empty() {
                        return format!(
                            "  <footer class=\"site-footer\">by {author} &middot; <code>{hash}</code></footer>\n",
                            author = html_escape(author),
                            hash = html_escape(hash),
                        );
                    }
                }
            }
        }
    }
    String::new()
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
    let theme_light = theme_set
        .themes
        .get("InspiredGitHub")
        .expect("InspiredGitHub theme");
    let theme_dark = theme_set
        .themes
        .get("base16-ocean.dark")
        .expect("base16-ocean.dark theme");

    let css_light = css_for_theme_with_class_style(theme_light, SYN_CLASS_STYLE)
        .expect("light theme css");
    let css_dark_raw = css_for_theme_with_class_style(theme_dark, SYN_CLASS_STYLE)
        .expect("dark theme css");
    let css_dark = scope_css(&css_dark_raw, ":root.dark");

    let mut syntax_css = String::new();
    syntax_css.push_str("/* generated by gen.rs from syntect themes — do not hand-edit */\n\n");
    syntax_css.push_str("/* light theme: InspiredGitHub */\n");
    syntax_css.push_str(&css_light);
    syntax_css.push_str("\n\n/* dark theme: base16-ocean.dark, scoped under :root.dark */\n");
    syntax_css.push_str(&css_dark);
    fs::write(root.join("syntax.css"), &syntax_css).expect("write syntax.css");

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

            let is_log = logger.macro_path == "log::info";
            let kv_sep = if is_log { ";" } else { "," };
            let f_dbg = if is_log {
                "items:? = vec![1, 2, 3]"
            } else {
                "items = ?vec![1, 2, 3]"
            };
            let f_disp = if is_log {
                "name:% = \"world\""
            } else {
                "name = %\"world\""
            };
            let snippet = fmt.snippet
                .replace("<MACRO>", logger.macro_path)
                .replace("<KV_SEP>", kv_sep)
                .replace("<F_DBG>", f_dbg)
                .replace("<F_DISP>", f_disp);
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
                highlight_rust(&snippet, &syntax_set, &syntax)
            ));
            cells.push_str(&format!(
                "        <pre class=\"output\"><code>{}</code></pre>\n",
                html_escape(cleaned.trim_end())
            ));
            cells.push_str("      </div>\n");
            cells.push_str("    </article>\n");
        }
    }

    let mut html = template.replace("{{cells}}", cells.trim_end());
    html = html.replace("{{owner_footer}}\n", &build_footer(&root));

    let out_path = root.join("index.html");
    fs::write(&out_path, html).expect("write index.html");
    println!("wrote {}", out_path.display());
}
