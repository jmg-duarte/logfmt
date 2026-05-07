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
        Ok("other") => {
            let n = 255u32;
            log::info!("hex={:x}  padded={:>5}  precision={:.2}  binary={:08b}", n, n, 3.14159, n);
        }
        _ => panic!("set FMT_KIND"),
    }
}
