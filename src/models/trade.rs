use chrono::prelude::NaiveDateTime;
use mysql::prelude::GenericConnection;

#[derive(Debug)]
pub struct Trade {
    id: u64,
    price: f64,
    volume: f64,
    trend: u16,
    ask_order_id: u64,
    bid_order_id: u64,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

impl Trade {
    pub fn create<T>(conn: &mut T, price: f64, volume: f64, ask_order_id: u64, bid_order_id: u64) -> u64 
    where T: GenericConnection
    {
        let mut stmt = conn.prepare(
            r"INSERT INTO trades 
                (price, volume, ask_order_id, bid_order_id)
              VALUES
                (:price, :volume, :ask_order_id, :bid_order_id)"
        ).unwrap();
        let id = stmt.execute((
            price,
            volume,
            ask_order_id,
            bid_order_id,
        )).unwrap().last_insert_id();

        id
    }

}
