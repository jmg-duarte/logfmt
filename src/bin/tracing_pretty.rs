fn main() {
    tracing_subscriber::fmt().pretty().init();
    tracing::info!("the answer was: {}", 12);
}
