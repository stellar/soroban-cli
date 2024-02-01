use std::collections::HashMap;

use testcontainers::{core::WaitFor, Image};

const NAME: &str = "stellar/quickstart";
// const TAG: &str = "testing";
const TAG: &str =
    "soroban-dev@sha256:0ad51035cf7caba2fd99c7c1fad0945df6932be7d5c893e1520ccdef7d6a6ffe";

#[derive(Debug, Default)]
pub struct Soroban(HashMap<String, String>, HashMap<String, String>);

impl Soroban {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("ENABLE_SOROBAN_RPC".to_string(), "true".to_string());
        map.insert(
            "ENABLE_SOROBAN_DIAGNOSTIC_EVENTS".to_string(),
            "true".to_string(),
        );
        map.insert("ENABLE_LOGS".to_string(), "true".to_string());
        map.insert("NETWORK".to_string(), "local".to_string());
        map.insert("POSTGRES_PASSWORD".to_string(), "p".to_string());
        #[allow(unused_mut)]
        let mut volumes = HashMap::new();
        // volumes.insert("/home/willem/c/s/soroban-tools/opt/stellar".to_string(), "/opt/stellar".to_string());
        Soroban(map, volumes)
    }
}

impl Image for Soroban {
    type Args = ();

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![8000, 11626]
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        // vec![WaitFor::seconds(30)]

        vec![WaitFor::message_on_stdout("friendbot: started")]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.0.iter())
    }

    fn volumes(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.1.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::Soroban;
    use soroban_cli::rpc::Client;
    use testcontainers::clients;

    #[tokio::test]
    async fn testcontainers_work() {
        let _ = pretty_env_logger::try_init();
        let docker = clients::Cli::default();
        let node = docker.run(Soroban::new());
        let host_port = node.get_host_port_ipv4(8000);
        let url: String = format!("http://localhost:{host_port}/soroban/rpc");
        println!("{url}");
        let client = Client::new(&url).unwrap();
        let res = client.get_network().await;
        println!("{res:#?}");
        assert!(res.is_ok());
    }
}
