use crate::side::Side;
use crate::order_book::OrderBook;
use crate::order_book_pair::OrderBookPair;
use crate::limit_order::LimitOrder;
use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use bigdecimal::ToPrimitive;

static PRICE_DECIMALS: u32 = 8;
static VOLUME_DECIMALS: u32 = 8;

pub struct Engine<'a>
{
    market: String,
    order_book_pair: OrderBookPair,
    volume_decimals: u32,
    on_trade: &'a Fn(TradeEvent),
}

pub struct TradeEvent {
    pub price: f64,
    pub volume: f64,
    pub funds: f64,
    pub ask_order_id: u64,
    pub ask_order_filled: bool,
    pub bid_order_id: u64,
    pub bid_order_filled: bool,
}

impl<'a> Engine<'a> 
{
    pub fn new(market: String, volume_decimals: u32, on_trade: &Fn(TradeEvent)) -> Engine {
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
        if !Engine::filled(&order, self.volume_decimals) {
            book.add(order);
        }
    }

    fn filled(order: &LimitOrder, volume_decimals: u32) -> bool {
        let min_volume = 1.0_f64 / (10_u64.pow(volume_decimals)) as f64;
        order.volume < min_volume
    }

    fn do_matching(on_trade: &Fn(TradeEvent), order: &mut LimitOrder, counter_book: &mut OrderBook, volume_decimals: u32) {
        match counter_book.top_mut() {
            Some(counter_order) => {
                match order.trade_with(counter_order) {
                    Some((trade_price, trade_volume, trade_funds)) => {

                        let order_id = order.id;
                        let counter_order_id = counter_order.id;

                        // fill orders
                        order.fill(trade_volume);
                        counter_order.fill(trade_volume);

                        // filled?
                        let order_filled = Engine::filled(&order, volume_decimals);
                        let cloned_counter_order = counter_order.clone();
                        let counter_order_filled = Engine::filled(&cloned_counter_order, volume_decimals);

                        // if counter_order has filled, remove it from counter_book
                        if counter_order_filled {
                            counter_book.remove(&cloned_counter_order);
                        }

                        match order.side {
                            Side::Sell => {
                                let trade_event = TradeEvent {
                                    price: trade_price,
                                    volume: trade_volume,
                                    funds: trade_funds,
                                    ask_order_id: order_id,
                                    ask_order_filled: order_filled,
                                    bid_order_id: counter_order_id,
                                    bid_order_filled: counter_order_filled,
                                };
                                on_trade(trade_event)
                            },
                            Side::Buy => {
                                let trade_event = TradeEvent {
                                    price: trade_price,
                                    volume: trade_volume,
                                    funds: trade_funds,
                                    ask_order_id: counter_order_id,
                                    ask_order_filled: counter_order_filled,
                                    bid_order_id: order_id,
                                    bid_order_filled: order_filled,
                                };
                                on_trade(trade_event)
                            } 
                        }

                        if !order_filled {
                            Engine::do_matching(on_trade, order, counter_book, volume_decimals);
                        } 

                    },
                    None => ()
                }
            },
            None => ()
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
