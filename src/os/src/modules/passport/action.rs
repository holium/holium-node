#[derive(Clone, Debug)]
pub enum IdentityAction {
    Alive { path: String, data: AliveData },
}

#[derive(Clone, Debug)]
pub struct AliveData {
    pub hid: String,
    pub boot_key: String,
}
