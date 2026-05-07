fn main() {
    tracing_subscriber::fmt().compact().init();
    tracing::info!("the answer was: {}", 12);
}
