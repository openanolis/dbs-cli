use dragonball::api::v1::NetworkInterfaceConfig;
use slog::Drain;
use slog::*;
use slog_scope::set_global_logger;

use std::str::FromStr;
use std::sync::Mutex;

pub fn setup_db_log(log_file_path: &String, log_level: &str) {
    let log_level = Level::from_str(log_level).unwrap();

    let file = std::fs::OpenOptions::new()
        .truncate(true)
        .read(true)
        .create(true)
        .write(true)
        .open(log_file_path)
        .expect("Cannot write to the log file.");

    let root = slog::Logger::root(
        Mutex::new(slog_json::Json::default(file).filter_level(log_level)).map(slog::Fuse),
        o!("version" => env!("CARGO_PKG_VERSION")),
    );

    let guard = set_global_logger(root);
    guard.cancel_reset();
    slog_stdlog::init().unwrap();
}

#[inline]
/// Get net device name from `NetworkInterfaceConfig`.
pub(crate) fn net_device_name(config: &NetworkInterfaceConfig) -> String {
    match &config.backend {
        dragonball::api::v1::Backend::Virtio(config) => {
            format!("virtio-net({})", config.host_dev_name)
        }
        dragonball::api::v1::Backend::Vhost(config) => {
            format!("vhost-net({})", config.host_dev_name)
        }
        dragonball::api::v1::Backend::VhostUser(config) => {
            format!("vhost-user-net({})", config.sock_path)
        }
    }
}
