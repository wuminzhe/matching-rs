use mysql::*;
use amiquip::{FieldTable, ExchangeType, ExchangeDeclareOptions, Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};

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
        // TODO: 隔离级别需要调整
        &pool
            .start_transaction(false, Some(IsolationLevel::RepeatableRead), Some(false))
            .and_then(|mut tx| {
                Trade::create(&mut tx, event.price, event.volume, event.ask_order_id, event.bid_order_id);

                Order::sub_volume(&mut tx, event.ask_order_id, event.volume, event.ask_order_filled);
                Order::sub_volume(&mut tx, event.bid_order_id, event.volume, event.bid_order_filled);
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
                    // TODO: 考虑去掉ack
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
}
