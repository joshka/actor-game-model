use std::{
    collections::HashMap,
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info};

use crate::{
    items::{Item, ItemId},
    money::Gold,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShopId(u64);

impl fmt::Display for ShopId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shop #{}", self.0)
    }
}

impl ShopId {
    fn new() -> ShopId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ShopId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug)]
pub struct Shop {
    id: ShopId,
    inventory: HashMap<ItemId, Item>,
    receiver: mpsc::Receiver<ShopMessage>,
}

impl Shop {
    pub fn new(items: impl IntoIterator<Item = Item>) -> ShopHandle {
        let (sender, receiver) = mpsc::channel(100);
        let id = ShopId::new();
        let shop = Shop {
            id,
            inventory: items.into_iter().map(|item| (item.id, item)).collect(),
            receiver,
        };
        tokio::spawn(shop.run());
        ShopHandle { id, sender }
    }

    #[tracing::instrument(skip(self), fields(shop_id = %self.id))]
    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            match message {
                ShopMessage::ListItems { response } => self.list_items(response),
                ShopMessage::CheckPrice { item_id, response } => {
                    self.check_price(item_id, response)
                }
                ShopMessage::BuyItem {
                    item_id,
                    payment,
                    response,
                } => self.buy_item(item_id, payment, response),
            }
        }
    }

    fn list_items(&self, response: oneshot::Sender<Result<Vec<Item>>>) {
        debug!("Listing items in the shop");
        let items = self.inventory.values().cloned().collect();
        let _ = response.send(Ok(items));
    }

    fn check_price(&self, item_id: ItemId, response: oneshot::Sender<Result<Gold>>) {
        if let Some(item) = self.inventory.get(&item_id) {
            info!("Checking price of item {item}: {price}", price = item.price);
            let _ = response.send(Ok(item.price));
        } else {
            info!("Item {item_id} is not available in the shop");
            let _ = response.send(Err(Error::ItemNotAvailable));
        }
    }

    fn buy_item(
        &mut self,
        item_id: ItemId,
        payment: Gold,
        response: oneshot::Sender<Result<Item, Error>>,
    ) {
        let Some(item) = self.inventory.remove(&item_id) else {
            info!("Item {item_id} is not available in the shop");
            let _ = response.send(Err(Error::ItemNotAvailable));
            return;
        };
        if payment < item.price {
            info!(
                "Not enough gold to purchase item {item_id}: {payment} < {price}",
                price = item.price
            );
            let buy_error = Error::NotEnoughGold {
                payment,
                price: item.price,
            };
            let _ = response.send(Err(buy_error));
            self.inventory.insert(item_id, item);
            return;
        }
        info!("{item} purchased for {payment}");
        let _ = response.send(Ok(item));
    }
}

#[derive(Debug, Clone)]
pub struct ShopHandle {
    id: ShopId,
    sender: mpsc::Sender<ShopMessage>,
}

impl fmt::Display for ShopHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl ShopHandle {
    pub async fn list_items(&self) -> Result<Vec<Item>> {
        let (response, receiver) = oneshot::channel();
        let message = ShopMessage::ListItems { response };
        let _ = self.sender.send(message).await?;
        receiver.await?
    }

    pub async fn buy_item(&self, item_id: ItemId, payment: Gold) -> Result<Item> {
        let (response, receiver) = oneshot::channel();
        let message = ShopMessage::BuyItem {
            item_id,
            payment,
            response,
        };
        self.sender.send(message).await?;
        receiver.await?
    }

    pub async fn check_price(&self, item_id: ItemId) -> Result<Gold> {
        let (response, receiver) = oneshot::channel();
        let message = ShopMessage::CheckPrice { item_id, response };
        self.sender.send(message).await?;
        receiver.await?
    }
}

#[derive(Debug)]
pub enum ShopMessage {
    /// List all items available in the shop
    ListItems {
        /// The response channel to send the list of items
        response: oneshot::Sender<Result<Vec<Item>>>,
    },
    /// Buy an item from the shop
    BuyItem {
        /// The item to buy
        item_id: ItemId,
        /// The amount of gold to pay for the item
        payment: Gold,
        /// The response channel to send the result of the purchase
        response: oneshot::Sender<Result<Item, Error>>,
    },
    /// Check the price of an item
    CheckPrice {
        item_id: ItemId,
        response: oneshot::Sender<Result<Gold>>,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The item is not available in the shop")]
    ItemNotAvailable,
    #[error("Not enough gold to purchase the item: {payment} < {price}")]
    NotEnoughGold { payment: Gold, price: Gold },
    #[error("Shop is closed")]
    ShopClosed(#[from] mpsc::error::SendError<ShopMessage>),
    #[error("No response from the shop")]
    NoResponse(#[from] oneshot::error::RecvError),
}
