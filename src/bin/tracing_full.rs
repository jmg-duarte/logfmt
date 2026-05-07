fn main() {
    tracing_subscriber::fmt().init();
    tracing::info!("the answer was: {}", 12);
}
