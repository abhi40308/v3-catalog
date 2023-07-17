use std::env;

pub struct Configuration {
    pub port: u32,
    pub default_database_url: Option<String>
}

fn get_port() -> u32 {
    let port_raw = env::var("PORT");
    match port_raw {
        Ok(port_string) => port_string.parse().unwrap(),
        Err(_) => 3000,
    }
}

pub fn get_default_db_url() -> Option<String> {
    env::var("DEFAULT_DB_URL").map(|url|Some(url)).unwrap_or(None)
}

pub fn get_configuration() -> Configuration {
    Configuration { port: get_port(), default_database_url: get_default_db_url() }
}
