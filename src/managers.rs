use std::error::Error;
use mysql::Pool;
use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use bigdecimal::ToPrimitive;

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
    pub fn new<'b>(pool: &'b Pool, volume_decimals: u32, price_decimals: u32, on_trade: &'b dyn Fn(TradeEvent), on_cancel: &'b dyn Fn(u64)) -> OrderManager<'b>
    {
        let engine = Engine::new(on_trade, on_cancel);

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
        println!("{}", volume);

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

        if price != 0.0 {
            let side: Side = if side == 0 { Side::Sell } else { Side::Buy };
            let limit_order = LimitOrder::new(
                id,
                side,
                volume,
                price,
            );
            &(self.engine).cancel(limit_order);
        }
    }

    pub fn print_orderbook(&self) {
        println!("{:?}", self.engine.order_book_pair.sell_order_book);
        println!("--- ask: ↑ --- bid: ↓ ---");
        println!("{:?}", self.engine.order_book_pair.buy_order_book);
    }

    // tool
    fn round(value: f64, decimals: u32) -> f64 {
        let t = 10_u32.pow(decimals) as f64;
        let t_big = BigDecimal::from_f64(t).unwrap();

        let value_big = BigDecimal::from_f64(value).unwrap();
        let rounded = ( value_big * t_big.clone() ).to_f64().unwrap().round();
        let rounded_big = BigDecimal::from_f64(rounded).unwrap();
        (rounded_big / t_big).to_f64().unwrap()
    }

    fn floor(value: f64, decimals: u32) -> f64 {
        let t = 10_u32.pow(decimals) as f64;
        let t_big = BigDecimal::from_f64(t).unwrap();

        let value_big = BigDecimal::from_f64(value).unwrap();
        let floored = ( value_big * t_big.clone() ).to_f64().unwrap().floor();
        let floored_big = BigDecimal::from_f64(floored).unwrap();
        (floored_big / t_big).to_f64().unwrap()
    }
}
