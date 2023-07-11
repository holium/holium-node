pub struct DbState {
    pool: Option<Pool<SqliteConnectionManager>>,
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
