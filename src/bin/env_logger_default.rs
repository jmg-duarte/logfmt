#[derive(Debug)]
#[allow(dead_code)]
struct User {
    name: &'static str,
    age: u32,
    roles: Vec<&'static str>,
}

fn main() {
    env_logger::init();

    match std::env::var("FMT_KIND").as_deref() {
        Ok("display") => {
            log::info!("hello {}, the answer is {}", "world", 42);
        }
        Ok("debug") => {
            let users = vec!["alice", "bob", "charlie"];
            log::info!("users: {:?}", users);
        }
        Ok("pretty_debug") => {
            let user = User { name: "alice", age: 30, roles: vec!["admin", "editor"] };
            log::info!("user: {:#?}", user);
        }
        Ok("named") => {
            let name = "alice";
            let age = 30;
            log::info!("user {name} is {age} years old");
        }
        Ok("numeric_base") => {
            let n = 255u32;
            log::info!("hex={:x}  octal={:o}  binary={:b}", n, n, n);
        }
        Ok("width") => {
            let n = 255u32;
            log::info!("padded={:>5}", n);
        }
        Ok("precision") => {
            log::info!("precision={:.2}", 3.14159);
        }
        Ok("fields") => {
            log::info!(answer = 42; "plain value");
            log::info!(items:? = vec![1, 2, 3]; "? sigil = Debug");
            log::info!(name:% = "world"; "% sigil = Display");
        }
        _ => panic!("set FMT_KIND"),
    }
}
