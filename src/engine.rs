use crate::side::Side;
use crate::order_book::OrderBook;
use crate::order_book_pair::OrderBookPair;
use crate::limit_order::LimitOrder;
use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use bigdecimal::ToPrimitive;

pub struct Engine<'a>
{
    market: String,
    order_book_pair: OrderBookPair,
    volume_decimals: u32,
    on_trade: &'a Fn(f64, f64, f64),
}

impl<'a> Engine<'a> 
{
    pub fn new(market: String, volume_decimals: u32, on_trade: &Fn(f64, f64, f64)) -> Engine {
        Engine {
            market: market,
            order_book_pair: OrderBookPair::new(),
            volume_decimals: volume_decimals,
            on_trade: on_trade,
        }
    }

    pub fn submit(&mut self, mut order: LimitOrder) {
        let (book, counter_book) = self.order_book_pair.get_books_mut(order.side);
        let on_trade = &(self.on_trade);
        Engine::do_matching(on_trade, &mut order, counter_book, self.volume_decimals);
        if !&order.is_filled() {
            book.add(order);
        }
    }

    fn do_matching(on_trade: &Fn(f64, f64, f64), order: &mut LimitOrder, counter_book: &mut OrderBook, volume_decimals: u32) {
        if !order.is_filled() && !order.is_tiny(volume_decimals) {
            match counter_book.top() {
                Some(counter_order) => {
                    match order.trade_with(counter_order) {
                        Some((trade_price, trade_volume, trade_funds)) => {
                            on_trade(trade_price, trade_volume, trade_funds);
                            counter_book.fill_top(trade_volume);
                            order.fill(trade_volume);
                            Engine::do_matching(on_trade, order, counter_book, volume_decimals);
                        },
                        None => ()
                    }
                },
                None => ()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Engine;
    use crate::side::Side;
    use crate::limit_order::LimitOrder;

    fn create_engine() -> Engine {
        let mut engine = Engine::new(String::from("ethbtc"), 8);
        let mut order1 = LimitOrder {
            id: 1,
            price: 1.34,
            volume: 1.2,
            side: Side::Buy,
            timestamp: 12345678
        };
        engine.submit(order1);
        
        let mut order2 = LimitOrder {
            id: 2,
            price: 1.35,
            volume: 0.9,
            side: Side::Buy,
            timestamp: 12345678
        };
        engine.submit(order2);
        return engine;
    }

    #[test]
    fn can_check_tiny() {
        let order = LimitOrder {
            id: 123456,
            price: 1.34,
            volume: 0.000000009,
            side: Side::Buy,
            timestamp: 12345678
        };
        assert!(&order.is_tiny(8));

        let order2 = LimitOrder {
            id: 123457,
            price: 1.34,
            volume: 0.00000001,
            side: Side::Buy,
            timestamp: 12345678
        };
        assert!(!&order2.is_tiny(8));
    }

    #[test]
    fn can_do_matching1() {
        let mut engine = create_engine();

        let (buy_book, _sell_book) = engine.order_book_pair.get_books(Side::Buy);
        assert_eq!(2, buy_book.len());
        assert_eq!(2, buy_book.top().unwrap().id);

        let mut order3 = LimitOrder {
            id: 3,
            price: 1.345,
            volume: 1.2,
            side: Side::Sell,
            timestamp: 12345678
        };
        engine.submit(order3);

        let (buy_book, sell_book) = engine.order_book_pair.get_books(Side::Buy);
        assert_eq!(1, buy_book.len());
        assert_eq!(1, sell_book.len());
        assert_eq!(1, buy_book.top().unwrap().id);
        assert_eq!(3, sell_book.top().unwrap().id);
    }

    #[test]
    fn can_do_matching2() {
        let mut engine = create_engine();

        let mut order3 = LimitOrder {
            id: 3,
            price: 1.345,
            volume: 0.8,
            side: Side::Sell,
            timestamp: 12345678
        };
        engine.submit(order3);

        let (buy_book, sell_book) = engine.order_book_pair.get_books(Side::Buy);
        assert_eq!(2, buy_book.len());
        assert_eq!(0, sell_book.len());
        assert_eq!(2, buy_book.top().unwrap().id);
        assert_eq!(None, sell_book.top());
    }
}
