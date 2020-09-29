use crate::search::{Node};
use std::sync::{RwLock, Mutex, Arc, RwLockWriteGuard};

pub fn bloom(leaf: &Arc<Node>, guard: RwLockWriteGuard<Vec<Arc<Node>>>) {

}
