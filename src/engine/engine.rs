use crate::engine::Side;
use crate::engine::OrderBook;
use crate::engine::OrderBookPair;
use crate::engine::LimitOrder;

pub struct Engine<'a>
{
    pub order_book_pair: OrderBookPair,
    on_trade: &'a dyn Fn(TradeEvent),
    on_cancel: &'a dyn Fn(u64),
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
    pub fn new<'b>(on_trade: &'b dyn Fn(TradeEvent), on_cancel: &'b dyn Fn(u64)) -> Engine<'b> {
        Engine {
            order_book_pair: OrderBookPair::new(),
            on_trade: on_trade,
            on_cancel: on_cancel,
        }
    }

    pub fn cancel(&mut self, order: LimitOrder) {
        let (book, counter_book) = self.order_book_pair.get_books_mut(order.side);
        match book.remove(&order) {
            Some(removed_order) => (self.on_cancel)(removed_order.id),
            None => ()
        };
    }

    pub fn submit(&mut self, mut order: LimitOrder) {
        let (book, counter_book) = self.order_book_pair.get_books_mut(order.side);
        let on_trade = &(self.on_trade);
        Engine::do_matching(on_trade, &mut order, counter_book);
        if !order.filled() {
            book.add(order);
        }
    }

    fn do_matching(on_trade: &dyn Fn(TradeEvent), order: &mut LimitOrder, counter_book: &mut OrderBook) {
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
                        let order_filled = order.filled();
                        let cloned_counter_order = counter_order.clone();
                        let counter_order_filled = cloned_counter_order.filled();

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
                            Engine::do_matching(on_trade, order, counter_book);
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
    use crate::engine::Side;
    use crate::engine::LimitOrder;
    use super::TradeEvent;

    fn create_engine<'a>(on_trade: &'a dyn Fn(TradeEvent)) -> Engine<'a> {
        let mut engine = Engine::new(on_trade);

        let mut order1 = LimitOrder {
            id: 1,
            price: 1.34,
            volume: 1.2,
            side: Side::Buy,
            
        };
        engine.submit(order1);
        
        let mut order2 = LimitOrder {
            id: 2,
            price: 1.35,
            volume: 0.9,
            side: Side::Buy,
            
        };
        engine.submit(order2);
        return engine;
    }

    #[test]
    fn can_do_matching1() {
        let on_trade = |event: TradeEvent| {
             println!("price: {}, volume: {}", event.price, event.volume);
        };
        let mut engine = create_engine(&on_trade);

        let (buy_book, _sell_book) = engine.order_book_pair.get_books(Side::Buy);
        assert_eq!(2, buy_book.len());
        assert_eq!(2, buy_book.top().unwrap().id);

        let mut order3 = LimitOrder {
            id: 3,
            price: 1.345,
            volume: 1.2,
            side: Side::Sell,
            
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
        let on_trade = |event: TradeEvent| {
             println!("price: {}, volume: {}", event.price, event.volume);
        };
        let mut engine = create_engine(&on_trade);

        let mut order3 = LimitOrder {
            id: 3,
            price: 1.345,
            volume: 0.8,
            side: Side::Sell,
            
        };
        engine.submit(order3);

        let (buy_book, sell_book) = engine.order_book_pair.get_books(Side::Buy);
        assert_eq!(2, buy_book.len());
        assert_eq!(0, sell_book.len());
        assert_eq!(2, buy_book.top().unwrap().id);
        assert_eq!(None, sell_book.top());
    }
}
