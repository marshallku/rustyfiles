use dotenv::dotenv;

use super::app::Env;

#[derive(Clone)]
pub struct AppState {
    pub host: String,
    pub port: u16,
    pub address: String,
    pub allowed_hosts: Vec<String>,
}

impl AppState {
    pub fn from_env() -> Self {
        dotenv().ok();

        let env = Env::new();

        Self {
            host: env.host.into_owned(),
            port: env.port,
            address: env.address.into_owned(),
            allowed_hosts: env.allowed_hosts,
        }
    }

    pub fn is_host_allowed(&self, host: &str) -> bool {
        if self.allowed_hosts.is_empty() {
            true
        } else {
            self.allowed_hosts.iter().any(|allowed| allowed == host)
        }
    }
}
