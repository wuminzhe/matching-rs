use mysql::Pool;
use crate::models::Order;
use crate::engine::Side;
use crate::engine::LimitOrder;
use crate::engine::Engine;
use crate::engine::TradeEvent;

// 每个成员的生命周期小于等于'a
pub struct OrderManager<'a> 
{
    pool: &'a Pool,
    engine: Engine<'a>,
}

impl<'a> OrderManager<'a> {
    // 返回的对象的成员的生命周期小于等于'b, 'b的实际值是pool和on_trade两者生命周期的小值
    pub fn new<'b>(pool: &'b Pool, on_trade: &'b dyn Fn(TradeEvent)) -> OrderManager<'b>
    {
        let engine = Engine::new(8, on_trade);

        OrderManager {
            engine: engine,
            pool: pool,
        }
    }

    pub fn submit(&mut self, price: f64, volume: f64, side: u8, created_by: &str) {
        let id: u64 = Order::create(self.pool, price, volume, side, created_by);

        // into engine
        let side: Side = if side == 0 { Side::Sell } else { Side::Buy };
        let limit_order = LimitOrder::new(
            id,
            side,
            volume,
            price,
        );
        &(self.engine).submit(limit_order);
    }
}
