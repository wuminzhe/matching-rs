use std::io::{Write, BufReader, BufRead, Error};
use std::fs::File;

extern crate intrusive_collections;
mod side;
mod limit_order;
mod order_book;
mod order_book_pair;
mod engine;

use crate::side::Side;
use crate::limit_order::LimitOrder;
use crate::engine::Engine;

use chrono::prelude::*;

mod data;
mod manager;

use manager::OrderManager;

#[derive(Debug, PartialEq, Eq)]
struct Member {
    id: u32,
    email: Option<String>,
    phone: Option<String>,
    created_at: Option<NaiveDateTime>,
}

fn main() -> Result<(), Error>{

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
    

    let pool = mysql::Pool::new("mysql://root:123456@localhost:3306/matching").unwrap();
    let on_trade = |trade_price: f64, trade_volume: f64, trade_funds: f64| {
        println!("price: {}, volume: {}", trade_price, trade_volume);
        println!("{:?}", data::Order::find(&pool));
    };

    let mut order_manager = OrderManager::new(&pool, &on_trade);
    order_manager.submit(0.13, 1299.0, 0, "123456");
    order_manager.submit(0.12, 10.0, 1, "123456");
    Ok(())

}
