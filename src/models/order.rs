use chrono::prelude::NaiveDateTime;
use mysql::prelude::GenericConnection;

#[derive(Debug)]
pub struct Order {
    id: u64,
    price: f64,
    volume: f64,
    origin_volume: f64,
    state: u16,
    side: u8, //0: ask, 1: buy
    trades_count: u16,
    created_by: Option<String>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

const WAIT: u8 = 100; 
const DONE: u8 = 200; 
const CANCEL: u8 = 0; 

impl Order {
    pub fn create(pool: &mysql::Pool, price: f64, volume: f64, side: u8,  created_by: &str) -> u64 {
        let mut stmt = pool.prepare(r"INSERT INTO orders 
                            (price, volume, origin_volume, state, side, created_by)
                        VALUES
                            (:price, :volume, :origin_volume, :state, :side, :created_by)").unwrap();
        let id = stmt.execute((
            price,
            volume,
            volume,
            WAIT,
            side,
            created_by,
        )).unwrap().last_insert_id();

        id
    }

    pub fn sub_volume<T>(conn: &mut T, id: u64, delta_volume: f64, filled: bool)
    where T: GenericConnection
    {
        let mut stmt = conn.prepare(r"UPDATE orders SET trades_count=trades_count+1, volume=volume-cast(:trade_volume as decimal(32,16)), state=:state WHERE id=:id").unwrap();
        let state = if filled { DONE } else { WAIT };
        stmt.execute((
            delta_volume,
            state,
            id,
        ));
    }

    pub fn add_volume<T>(conn: &mut T, id: u64, delta_volume: f64, filled: bool)
    where T: GenericConnection
    {
        let mut stmt = conn.prepare(r"update orders set trades_count=trades_count+1, volume=volume+cast(:trade_volume as decimal(32,16)), state=:state where id=:id").unwrap();
        let state = if filled { DONE } else { WAIT };
        stmt.execute((
            delta_volume,
            state,
            id,
        ));
    }

    pub fn find(pool: &mysql::Pool) -> Vec<Order> {
        let selected_payments: Vec<Order> =
            pool.prep_exec("SELECT id, price, volume, origin_volume, state, side, trades_count, created_by, created_at, updated_at from orders", ())
            .map(|result| { // In this closure we will map `QueryResult` to `Vec<Order>`
                // `QueryResult` is iterator over `MyResult<row, err>` so first call to `map`
                // will map each `MyResult` to contained `row` (no proper error handling)
                // and second call to `map` will map each `row` to `Order`
                result.map(|x| x.unwrap()).map(|row| {
                    // ⚠️ Note that from_row will panic if you don't follow your schema
                    let (id, price, volume, origin_volume, state, side, trades_count, created_by, created_at, updated_at) = mysql::from_row(row);
                    Order {
                        id: id,
                        price: price,
                        volume: volume,
                        origin_volume: origin_volume,
                        state: state,
                        side: side,
                        trades_count: trades_count,
                        created_by: created_by,
                        created_at: created_at,
                        updated_at: updated_at,
                    }
                }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Order>`
            }).unwrap(); // Unwrap `Vec<Order>`
        selected_payments
    }

}
