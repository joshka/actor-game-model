use items::{Item, ItemId};
use money::Gold;
use player::{Player, PlayerHandle};
use shop::ShopHandle;
use tokio::join;
use tracing::info;

mod items;
mod money;
mod player;
mod shop;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let sword = Item::new("Sword", Gold::new(100));
    let shield = Item::new("Shield", Gold::new(150));
    let axe = Item::new("Axe", Gold::new(200));
    let bow = Item::new("Bow", Gold::new(250));

    let sword_id = sword.id;
    let shield_id = shield.id;

    let shop = shop::Shop::new([sword, shield, axe, bow]);

    let player1 = Player::new(Gold::new(200), []);
    let player2 = Player::new(Gold::new(300), []);

    let _ = join!(
        buy(player1, shop.clone(), sword_id),
        buy(player2, shop.clone(), shield_id)
    );
    Ok(())
}

async fn buy(
    mut player: PlayerHandle,
    shop: ShopHandle,
    item_id: ItemId,
) -> Result<(), player::BuyError> {
    match player.buy(shop.clone(), item_id).await {
        Ok(item) => {
            info!("{player} bought {item} from {shop}");
        }
        Err(error) => {
            info!("{player} failed to buy {item_id} from {shop}: {error:?}");
        }
    }
    Ok(())
}
