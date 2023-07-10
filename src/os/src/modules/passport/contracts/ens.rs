#[derive(Clone, Debug)]
pub struct ENS {
    pub contract: String,
    pub network: String,
}

impl ENS {
    pub fn describe(&self) -> String {
        "ens".to_string()
    }
}
