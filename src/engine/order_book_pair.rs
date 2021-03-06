use crate::engine::Side;
use crate::engine::OrderBook;

#[derive(Debug)]
pub struct OrderBookPair {
    pub sell_order_book: OrderBook,
    pub buy_order_book: OrderBook
}

impl OrderBookPair {
    pub fn new() -> OrderBookPair {
        let sell_order_book = OrderBook::new(Side::Sell);
        let buy_order_book  = OrderBook::new(Side::Buy);
        OrderBookPair {
            sell_order_book: sell_order_book,
            buy_order_book: buy_order_book
        }
    }

    pub fn get_books(&self, side: Side) -> (&OrderBook, &OrderBook) {
        match side {
            Side::Sell => (&self.sell_order_book, &self.buy_order_book),
            Side::Buy => (&self.buy_order_book, &self.sell_order_book)
        }
    }

    pub fn get_books_mut(&mut self, side: Side) -> (&mut OrderBook, &mut OrderBook) {
        match side {
            Side::Sell => (&mut self.sell_order_book, &mut self.buy_order_book),
            Side::Buy => (&mut self.buy_order_book, &mut self.sell_order_book)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OrderBookPair;
    use crate::engine::Side;

    #[test]
    fn can_get_books() {
        let order_book_pair = OrderBookPair::new();
        let (buy_order_book, sell_order_book) = order_book_pair.get_books(Side::Buy);
        assert_eq!(buy_order_book.side, Side::Buy);
        assert_eq!(sell_order_book.side, Side::Sell);
    }

}
