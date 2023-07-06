use std::env;

pub struct Configuration {
    pub port: u32,
}

fn get_port() -> u32 {
	let port_raw = env::var("PORT");
	match port_raw {
		Ok(port_string) => port_string.parse().unwrap(),
		Err(_) => 3000
	}
}

pub fn get_configuration() -> Configuration {
	Configuration {
		port: get_port()
	}
}