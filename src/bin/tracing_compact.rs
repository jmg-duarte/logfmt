#[derive(Debug)]
#[allow(dead_code)]
struct User {
    name: &'static str,
    age: u32,
    roles: Vec<&'static str>,
}

fn main() {
    tracing_subscriber::fmt().compact().init();

    match std::env::var("FMT_KIND").as_deref() {
        Ok("display") => {
            tracing::info!("hello {}, the answer is {}", "world", 42);
        }
        Ok("debug") => {
            let users = vec!["alice", "bob", "charlie"];
            tracing::info!("users: {:?}", users);
        }
        Ok("pretty_debug") => {
            let user = User { name: "alice", age: 30, roles: vec!["admin", "editor"] };
            tracing::info!("user: {:#?}", user);
        }
        Ok("named") => {
            let name = "alice";
            let age = 30;
            tracing::info!("user {name} is {age} years old");
        }
        Ok("numeric_base") => {
            let n = 255u32;
            tracing::info!("hex={:x}  octal={:o}  binary={:b}", n, n, n);
        }
        Ok("width") => {
            let n = 255u32;
            tracing::info!("padded={:>5}", n);
        }
        Ok("precision") => {
            tracing::info!("precision={:.2}", 3.14159);
        }
        Ok("fields") => {
            tracing::info!(answer = 42, "plain value");
            tracing::info!(items = ?vec![1, 2, 3], "? sigil = Debug");
            tracing::info!(name = %"world", "% sigil = Display");
        }
        _ => panic!("set FMT_KIND"),
    }
}
