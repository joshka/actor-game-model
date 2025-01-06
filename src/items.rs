use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::money::Gold;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(u64);

impl ItemId {
    pub fn new() -> ItemId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ItemId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Item#{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub price: Gold,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.name)?;
        write!(f, "({})", self.id)?;
        Ok(())
    }
}

impl Item {
    pub fn new(name: &str, price: Gold) -> Item {
        let id = ItemId::new();
        let name = name.to_string();
        Item { id, name, price }
    }
}
