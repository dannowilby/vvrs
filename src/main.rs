fn main() {
    env_logger::init();

    log::info!("Hello, world!");

    match vvrs::run() {
        Ok(()) => {}
        Err(e) => {
            log::error!("Event loop error: {}", e.to_string())
        }
    }
}
