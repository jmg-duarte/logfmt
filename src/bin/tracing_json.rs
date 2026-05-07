fn main() {
    tracing_subscriber::fmt().json().init();
    tracing::info!("the answer was: {}", 12);
}
