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

    pub fn submit(&mut self, price: f64, volume: f64, side: u8, created_by: &str) {
        // price 采用四舍五入
        let price = OrderManager::round(price, self.price_decimals);
        // volume 采用截断
        let volume = OrderManager::floor(volume, self.volume_decimals);

        // 创建订单
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
