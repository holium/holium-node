// use serde::Deserialize;
use std::collections::HashSet;

use serde::Deserialize;

pub type Rid = String;
pub type Capacity = u64;
pub type Title = String;

#[derive(Deserialize, Debug)]
pub struct Room {
    pub rid: Rid,
    pub provider: String,
    pub creator: String,
    pub access: String,
    pub title: Title,
    pub present: HashSet<String>,
    pub whitelist: HashSet<String>,
    pub capacity: Capacity,
    pub path: Option<String>,
}

// impl Room {
//     fn new(
//         rid: Rid,
//         provider: String,
//         creator: String,
//         access: String,
//         title: Title,
//         capacity: Capacity,
//         path: Option<String>,
//     ) -> Self {
//         Room {
//             rid,
//             provider,
//             creator,
//             access,
//             title,
//             present: HashSet::new(),
//             whitelist: HashSet::new(),
//             capacity,
//             path,
//         }
//     }
//     fn add_present(&mut self, ship: String) {
//         self.present.insert(ship);
//     }
//     fn remove_present(&mut self, ship: String) {
//         self.present.remove(&ship);
//     }
//     fn add_whitelist(&mut self, ship: String) {
//         self.whitelist.insert(ship);
//     }
//     fn remove_whitelist(&mut self, ship: String) {
//         self.whitelist.remove(&ship);
//     }
//     fn set_capacity(&mut self, capacity: Capacity) {
//         self.capacity = capacity;
//     }
//     fn set_path(&mut self, path: Option<String>) {
//         self.path = path;
//     }
// }
