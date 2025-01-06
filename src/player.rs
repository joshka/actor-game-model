use std::{
    collections::HashMap,
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tracing::info;

use super::money::Gold;
use crate::{
    items::{Item, ItemId},
    shop::ShopHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(u64);

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player{}", self.0)
    }
}

impl PlayerId {
    fn new() -> PlayerId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        PlayerId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Player {
    id: PlayerId,
    wallet: Gold,
    inventory: HashMap<ItemId, Item>,
    receiver: mpsc::Receiver<PlayerMessage>,
}

impl Player {
    pub fn new(gold: Gold, items: impl IntoIterator<Item = Item>) -> PlayerHandle {
        let (sender, receiver) = mpsc::channel(100);
        let id = PlayerId::new();
        let items = items.into_iter().map(|item| (item.id, item)).collect();
        let player = Player {
            id,
            wallet: gold,
            inventory: items,
            receiver,
        };
        tokio::spawn(player.run());
        PlayerHandle { id, sender }
    }

    #[tracing::instrument(skip(self), fields(player_id = %self.id))]
    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            match message {
                PlayerMessage::BuyItem {
                    shop,
                    item_id,
                    response,
                } => self.buy(shop, item_id, response).await,
            }
        }
    }

    #[tracing::instrument(skip_all)]
    async fn buy(
        &mut self,
        shop: ShopHandle,
        item_id: ItemId,
        response: oneshot::Sender<Result<Item, BuyError>>,
    ) {
        let Ok(price) = shop.check_price(item_id).await else {
            info!("Item {item_id} is not available in the shop");
            let _ = response.send(Err(BuyError::NotAvailable));
            return;
        };
        let available = self.wallet;
        if available < price {
            info!(
                "Not enough gold to buy item {item_id}. You have {available:?} but the price is {price:?}",
                price = price
            );
            let buy_error = BuyError::NotEnoughGold { available, price };
            let _ = response.send(Err(buy_error));
            return;
        }
        self.wallet -= price;
        let Ok(item) = shop.buy_item(item_id, price).await else {
            info!("Item {item_id} is not available in the shop");
            self.wallet += price;
            let _ = response.send(Err(BuyError::NotAvailable));
            return;
        };
        info!("Successfully bought item {item} for {price}");
        let _ = response.send(Ok(item.clone()));
        self.inventory.insert(item.id, item);
    }
}

#[derive(Debug, Clone)]
pub struct PlayerHandle {
    id: PlayerId,
    sender: mpsc::Sender<PlayerMessage>,
}

impl fmt::Display for PlayerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl PlayerHandle {
    pub async fn buy(&mut self, shop: ShopHandle, item_id: ItemId) -> Result<Item, BuyError> {
        let (response_tx, response_rx) = oneshot::channel();
        let message = PlayerMessage::BuyItem {
            shop,
            item_id,
            response: response_tx,
        };
        self.sender.send(message).await?;
        response_rx.await?
    }
}

#[derive(Debug)]
pub enum PlayerMessage {
    /// The player wants to buy an item.
    BuyItem {
        /// The player selling the item.
        shop: ShopHandle,
        /// The item to buy.
        item_id: ItemId,
        /// The response channel to send the result of the buy operation.
        response: oneshot::Sender<Result<Item, BuyError>>,
    },
}

#[derive(Debug, Error)]
pub enum BuyError {
    #[error("The item is not available")]
    NotAvailable,
    #[error("Not enough gold to buy the item. You have {available:?} but the price is {price:?}")]
    NotEnoughGold { available: Gold, price: Gold },
    #[error("Player Disconnected")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<PlayerMessage>),
    #[error("Failed to receive response")]
    ReceiveError(#[from] tokio::sync::oneshot::error::RecvError),
}
