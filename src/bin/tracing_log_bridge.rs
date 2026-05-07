fn main() {
    tracing_log::LogTracer::init().expect("LogTracer init");
    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber).expect("set global default");
    log::info!("the answer was: {}", 12);
}
