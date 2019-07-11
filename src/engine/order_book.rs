use crate::engine::Side;
use crate::engine::LimitOrder;
use std::collections::BTreeMap;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct OrderBook {
    pub side: Side,
    pub limit_orders: BTreeMap<String, VecDeque<LimitOrder>>
}

impl OrderBook {
    pub fn new(side: Side) -> OrderBook {
        OrderBook {
            side: side,
            limit_orders: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, order: LimitOrder) {
        if order.volume > 0.0 {
            let price_key = order.price.to_string();

            match self.limit_orders.get_mut(&price_key) {
                Some(orders) =>
                    orders.push_back(order),
                None => {
                    let mut orders = VecDeque::new();
                    orders.push_back(order);
                    self.limit_orders.insert(price_key, orders);
                }
            }
        }
    }

    pub fn remove(&mut self, order: &LimitOrder) -> Option<LimitOrder>{
        let price_key = order.price.to_string();
        let result_order = match self.limit_orders.get_mut(&price_key) {
            Some(queue) => {
                let index = queue.iter().position(|o| o.id == order.id).unwrap();
                queue.remove(index)
            },
            None => None
        };

        match self.limit_orders.get(&price_key) {
            Some(queue) => {
                if queue.is_empty() {
                    self.limit_orders.remove(&price_key);
                }
            },
            None => {}
        };

        return result_order;
    }

    pub fn top(&self) -> Option<&LimitOrder> {
        let line = match self.side {
            Side::Buy  => self.limit_orders.iter().last(),
            Side::Sell => self.limit_orders.iter().next()
        };

        match line {
            Some((_price_key, price_level)) => price_level.front(),
            None => None
        }
    }

    pub fn top_mut(&mut self) -> Option<&mut LimitOrder> {
        let line = match self.side {
            Side::Buy  => self.limit_orders.iter_mut().last(),
            Side::Sell => self.limit_orders.iter_mut().next()
        };

        match line {
            Some((_price_key, price_level)) => price_level.front_mut(),
            None => None
        }
    }

    // pub fn fill_top(&mut self, trade_volume: f64) {
        // match self.top_mut() {
            // Some(top_order) => {
                // top_order.fill(trade_volume);
                // let o = &top_order.clone();
                    // self.remove(o);
            // },
            // None => ()
        // }
    // }
    
    pub fn is_empty(&self) -> bool {
        self.limit_orders.is_empty()
    }

    pub fn len(&self) -> usize {
        self.limit_orders.len()
    }


}

#[cfg(test)]
mod tests {
    use super::OrderBook;
    use crate::limit_order::LimitOrder;
    use crate::side::Side;

    #[test]
    fn can_create_new_order_book() {
        let order_book = OrderBook::new(Side::Buy);
        assert!(order_book.is_empty());
    }

    #[test]
    fn can_add_order() {
        let mut order_book = OrderBook::new(Side::Buy);
        assert!(order_book.is_empty());

        let limit_order = LimitOrder {
            id: 123456,
            price: 1.34,
            volume: 3.00,
            side: Side::Buy,
            // timestamp: 12345678
        };
        order_book.add(limit_order);
        assert!(!order_book.is_empty());

        let order = order_book.top().unwrap();
        assert_eq!(order, &limit_order);
    }

    #[test]
    fn can_remove_order() {
        let mut order_book = OrderBook::new(Side::Buy);

        let limit_order = LimitOrder {
            id: 123456,
            price: 1.34,
            volume: 3.00,
            side: Side::Buy,
            // timestamp: 12345678
        };
        order_book.add(limit_order);

        let order = order_book.top().unwrap().clone();
        let o = order_book.remove(&order).unwrap();
        // 这里的比较？
        assert_eq!(limit_order, o);

        assert!(order_book.is_empty());
    }
}
