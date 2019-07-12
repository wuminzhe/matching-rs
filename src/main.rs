use mysql::*;

// extern crate intrusive_collections;

mod engine;
mod models;
mod managers;
mod errors;

use engine::*;
use models::*;
use managers::OrderManager;

fn main(){

    // let path = "./src/orders.csv";
    // let input = File::open(path)?;
    // let buffered = BufReader::new(input);
    // for line in buffered.lines() {
        // let row = line?;
        // let mut split = row.split(",");
        // let id: u32 = split.next().unwrap().parse().unwrap();
        // let price: f64 = split.next().unwrap().parse().unwrap();
        // let volume: f64 = split.next().unwrap().parse().unwrap();
        // let order_type = split.next().unwrap();
        // let side = if order_type == "OrderAsk" {
            // Side::Sell
        // } else {
            // Side::Buy
        // };
        // let limit_order = LimitOrder::new(
            // id,
            // side,
            // volume,
            // price,
        // );
        // engine.submit(limit_order);
    // }
    

    let pool = Pool::new("mysql://root:123456@localhost:3306/matching").unwrap();

    let on_trade = |event: TradeEvent| {
        println!("price: {}, volume: {}", event.price, event.volume);
        &pool
            .start_transaction(false, Some(IsolationLevel::RepeatableRead), Some(false))
            .and_then(|mut tx| {
                Trade::create(&mut tx, event.price, event.volume, event.ask_order_id, event.bid_order_id);

                Order::sub_volume(&mut tx, event.ask_order_id, event.volume, event.ask_order_filled);
                Order::add_volume(&mut tx, event.bid_order_id, event.volume, event.bid_order_filled);
                tx.commit()
            });
    };

    let mut order_manager = OrderManager::new(&pool, &on_trade, 8, 8);
    // order_manager.submit(0.12, 11.000000035, 0, "u123456");
    // order_manager.submit(0.13, 11.00000003, 1, "u123456");

    let id = order_manager.submit(0.000003456, 11.000000035, 0, "u123456").unwrap();
    order_manager.print_orderbook();
    order_manager.cancel(id, 0.000003456, 11.000000035, 0, "u123456");
    order_manager.print_orderbook();
    order_manager.submit(0.000004,    11.0000000315, 1, "u123456");
}
