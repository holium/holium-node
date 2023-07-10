use crate::holon::OurPeer;

use self::contracts::IdentitySystem;

pub mod contracts;
pub mod passport_state;

pub struct PassportModule {
    pub identity: Option<OurPeer>,
    pub identity_system: Option<IdentitySystem>,
}

impl PassportModule {
    pub fn new() -> Self {
        return PassportModule {
            identity: None,
            identity_system: None,
        };
    }

    pub fn genesis(&mut self, identity: OurPeer, identity_system: IdentitySystem) {
        self.identity = Some(identity);
        self.identity_system = Some(identity_system);
    }
}
