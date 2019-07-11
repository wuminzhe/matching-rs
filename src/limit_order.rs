use crate::side::Side;
use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use bigdecimal::ToPrimitive;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LimitOrder {
    pub id: u64,
    // pub timestamp: u64,
    pub side: Side,
    pub volume: f64,
    pub price: f64,
}

impl LimitOrder {
    pub fn new(id: u64, side: Side, volume: f64, price: f64) -> LimitOrder {
        LimitOrder {
            id: id,
            // timestamp: timestamp,
            side: side,
            volume: volume,
            price: price,
        }
    }

    pub fn fill(&mut self, trade_volume: f64) {
        if self.volume >= trade_volume {
            let result = BigDecimal::from_f64(self.volume).unwrap() - BigDecimal::from_f64(trade_volume).unwrap();
            self.volume = result.to_f64().unwrap();
        } 
    }

    fn is_crossed(&self, price: f64) -> bool {
        match self.side {
            Side::Sell => price >= self.price,
            Side::Buy  => price <= self.price
        }
    }

    // counter order是老订单，所以价格以他的为准
    pub fn trade_with(&self, counter_order: &LimitOrder) -> Option<(f64, f64, f64)> {
        if self.is_crossed(counter_order.price) {
            let trade_price = counter_order.price;
            let trade_volume = self.volume.min(counter_order.volume);
            // println!("{:?}", (&big_trade_volume * &big_trade_price).to_f64());
            let trade_funds = BigDecimal::from_f64(trade_volume).unwrap() * BigDecimal::from_f64(trade_price).unwrap();
            // println!("{0} - {1}", self.id, counter_order.id);
            Some((trade_price, trade_volume, trade_funds.to_f64().unwrap()))
        } else {
            None
        }
    }

}

#[cfg(test)]
mod tests {
    use super::LimitOrder;
    use crate::side::Side;
    use bigdecimal::BigDecimal;
    use bigdecimal::FromPrimitive;
    use bigdecimal::ToPrimitive;

    fn create_limit_order() -> LimitOrder {
        LimitOrder::new(
            123456,
            Side::Buy,
            32.12,
            2.12,
        )
    }

    #[test]
    fn can_fill_by_volume() {
        let mut limit_order = create_limit_order();
        limit_order.fill(50.0);
        assert_eq!(limit_order.volume, 32.12);
        limit_order.fill(10.0);
        assert_eq!(limit_order.volume, 22.12);
        limit_order.fill(22.12);
        assert!(limit_order.filled(8));
    }

    #[test]
    fn can_cross() {
        let limit_order = create_limit_order();
        assert!(limit_order.is_crossed(2.03));
        assert!(!limit_order.is_crossed(2.25));
    }

    #[test]
    fn can_trade_with_counter_order() {
        let buy_order = create_limit_order();
        let sell_order = LimitOrder::new(
            123457,
            Side::Sell,
            15.88,
            2.0,
        );
        match buy_order.trade_with(&sell_order) {
            Some((trade_price, trade_volume, trade_funds)) => {
                assert_eq!(trade_price, 2.0);
                assert_eq!(trade_volume, 15.88);
                assert_eq!(trade_funds, 31.76);
            }
            None => assert!(false)
        }

        // assert_eq!(0.3, 0.1+0.2);
        let result = BigDecimal::from_f64(0.1).unwrap() + BigDecimal::from_f64(0.2).unwrap();
        assert_eq!(0.3, result.to_f64().unwrap());
    }

}
