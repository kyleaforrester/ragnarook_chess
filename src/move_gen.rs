use crate::search::{Node};
use std::sync::{Arc, RwLockWriteGuard};

pub fn bloom(_leaf: &Arc<Node>, _guard: RwLockWriteGuard<Vec<Arc<Node>>>) {

}
