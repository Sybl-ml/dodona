use config::Environment;
use std::env;
use std::str::FromStr;

pub struct Params {
    pub conn_str: String,
    pub node_socket: u16,
    pub interface_socket: u16,
}

pub fn initialise() -> Params {
    let config = config::ConfigFile::from_filesystem();
    let resolved = config.resolve(Environment::Testing);
    resolved.populate_environment();
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let node_socket =
        u16::from_str(&env::var("NODE_SOCKET").expect("NODE_SOCKET must be set")).unwrap();
    let interface_socket =
        u16::from_str(&env::var("INTERFACE_SOCKET").expect("INTERFACE_SOCKET must be set"))
            .unwrap();
    Params {
        conn_str,
        node_socket,
        interface_socket,
    }
}
