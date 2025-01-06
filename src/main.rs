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
    let bow_id = bow.id;

    let player1 = Player::new(Gold::new(1000), []);
    let player2 = Player::new(Gold::new(1000), []);

    let shop1 = shop::Shop::new(player1.clone(), [sword, shield]);
    let shop2 = shop::Shop::new(player2.clone(), [axe, bow]);

    let _ = join!(
        buy(player1.clone(), shop2, bow_id),
        buy(player2.clone(), shop1, sword_id)
    );

    let player1_info = player1.info().await;
    let player2_info = player2.info().await;

    info!("Player1: {:?}", player1_info);
    info!("Player2: {:?}", player2_info);

    Ok(())
}

async fn buy(
    mut player: PlayerHandle,
    shop: ShopHandle,
    item_id: ItemId,
) -> Result<(), player::PlayerError> {
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
