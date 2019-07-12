use std::error::Error;
use mysql::Pool;

use crate::models::Order;
use crate::engine::Side;
use crate::engine::LimitOrder;
use crate::engine::Engine;
use crate::engine::TradeEvent;

use crate::errors::TinyError;

// 每个成员的生命周期小于等于'a
pub struct OrderManager<'a> 
{
    pool: &'a Pool,
    engine: Engine<'a>,

    volume_decimals: u32,
    price_decimals: u32,
}

impl<'a> OrderManager<'a> {
    // 返回的对象的成员的生命周期小于等于'b, 'b的实际值是pool和on_trade两者生命周期的小值
    pub fn new<'b>(pool: &'b Pool, on_trade: &'b dyn Fn(TradeEvent), volume_decimals: u32, price_decimals: u32) -> OrderManager<'b>
    {
        let engine = Engine::new(on_trade);

        OrderManager {
            engine: engine,
            pool: pool,
            volume_decimals: volume_decimals,
            price_decimals: price_decimals,
        }
    }

    pub fn submit(&mut self, price: f64, volume: f64, side: u8, created_by: &str) -> Result<u64, Box<Error>>{
        // price 采用四舍五入
        let price = OrderManager::round(price, self.price_decimals);
        // volume 采用截断
        let volume = OrderManager::floor(volume, self.volume_decimals);

        if price != 0.0 && volume != 0.0 {
            // 创建订单
            let id: u64 = Order::create(self.pool, price, volume, side, created_by);

            // 入撮合引擎
            let side: Side = if side == 0 { Side::Sell } else { Side::Buy };
            let limit_order = LimitOrder::new(
                id,
                side,
                volume,
                price,
            );
            &(self.engine).submit(limit_order);
            Ok(id)
        } else {
            Err(Box::new(TinyError::new("")))
        }
    }

    pub fn cancel(&mut self, id: u64, price: f64, volume: f64, side: u8, created_by: &str) {
        let price = OrderManager::round(price, self.price_decimals);

        let side: Side = if side == 0 { Side::Sell } else { Side::Buy };
        let limit_order = LimitOrder::new(
            id,
            side,
            volume,
            price,
        );
        &(self.engine).cancel(limit_order);
    }

    pub fn print_orderbook(&self) {
        println!("+------------------------------------------------------------------------------");
        println!("|{:?}", self.engine.order_book_pair.sell_order_book);
        println!("+--- ask: ↑ --- bid: ↓ ---");
        println!("|{:?}", self.engine.order_book_pair.buy_order_book);
        println!("+------------------------------------------------------------------------------");
    }

    // tool
    fn round(value: f64, decimals: u32) -> f64 {
        let t = 10_u32.pow(decimals) as f64;
        (value * t).round() / t
    }

    fn floor(value: f64, decimals: u32) -> f64 {
        let t = 10_u32.pow(decimals) as f64;
        (value * t).floor() / t
    }
}
