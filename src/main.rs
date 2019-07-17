use mysql::*;
use amiquip::{FieldTable, ExchangeType, ExchangeDeclareOptions, Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};

// extern crate intrusive_collections;

mod engine;
mod models;
mod managers;
mod errors;

use engine::*;
use models::*;
use managers::OrderManager;

fn main(){
    let pool = Pool::new("mysql://root:123456@localhost:3306/matching").unwrap();

    let on_trade = |event: TradeEvent| {
        println!("price: {}, volume: {}", event.price, event.volume);
        &pool
            .start_transaction(false, Some(IsolationLevel::RepeatableRead), Some(false))
            .and_then(|mut tx| {
                Trade::create(&mut tx, event.price, event.volume, event.ask_order_id, event.bid_order_id);

                Order::sub_volume(&mut tx, event.ask_order_id, event.volume, event.ask_order_filled);
                Order::add_volume(&mut tx, event.bid_order_id, event.volume, event.bid_order_filled);
                tx.commit()
            });
    };

    let on_cancel = |order_id| {
        &pool
            .start_transaction(false, Some(IsolationLevel::RepeatableRead), Some(false))
            .and_then(|mut tx| {
                Order::set_canceled(&mut tx, order_id);
                tx.commit()
            });
    };

    let mut order_manager = OrderManager::new(&pool, 8, 8, &on_trade, &on_cancel);

    env_logger::init();
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672").unwrap();
    let channel = connection.open_channel(None).unwrap();
    let queue = channel.queue_declare("hello", QueueDeclareOptions::default()).unwrap();
    let exchange = channel.exchange_declare(
        ExchangeType::Direct,
        "exchange.orders",
        ExchangeDeclareOptions {
            durable: true,
            ..ExchangeDeclareOptions::default()
        }
    ).unwrap();
    queue.bind(&exchange, "order.ethbtc", FieldTable::new()).unwrap();
    // Start a consumer.
    let consumer = queue.consume(ConsumerOptions::default()).unwrap();
    println!("Waiting for messages. Press Ctrl-C to exit.");
    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                let split = body.split(",").collect::<Vec<&str>>();
                if split.len() == 4 {
                    let price = split[0].parse::<f64>().unwrap();
                    let volume = split[1].parse::<f64>().unwrap();
                    let side = split[2].parse::<u8>().unwrap();
                    let user_id = split[3];

                    let id = order_manager.submit(price, volume, side, user_id).unwrap();
                    // println!("({:>3}) Received [{}]", i, body);
                    consumer.ack(delivery);
                }
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close();

    // let path = "./src/orders.csv";
    // let input = File::open(path);
    // let buffered = BufReader::new(input);
    // for line in buffered.lines() {
        // let row = line;
        // let mut split = row.split(",");
        // let id: u32 = split.next().unwrap().parse().unwrap();
        // let price: f64 = split.next().unwrap().parse().unwrap();
        // let volume: f64 = split.next().unwrap().parse().unwrap();
        // let order_type = split.next().unwrap();
        // let side = if order_type == "OrderAsk" {
            // Side::Sell
        // } else {
            // Side::Buy
        // };
        // let limit_order = LimitOrder::new(
            // id,
            // side,
            // volume,
            // price,
        // );
        // engine.submit(limit_order);
    // }
    


    // order_manager.submit(0.12, 11.000000035, 0, "u123456");
    // order_manager.submit(0.13, 11.00000003, 1, "u123456");

}
