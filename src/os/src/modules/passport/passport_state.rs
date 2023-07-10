use crate::holon::OurPeer;

use super::contracts::IdentitySystem;

#[derive(Clone)]
pub struct PassportState {
    pub identity: Option<OurPeer>,
    pub identity_system: Option<IdentitySystem>,
}

impl PassportState {
    pub fn new() -> Self {
        PassportState {
            identity: None,
            identity_system: None,
        }
    }

    pub fn genesis(&mut self, identity: OurPeer, identity_system: IdentitySystem) {
        self.identity = Some(identity);
        self.identity_system = Some(identity_system);
    }
}
