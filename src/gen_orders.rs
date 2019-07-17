extern crate amqp;
use amqp::{Basic, Session, Channel, Table, protocol};
use std::default::Default;
use std::thread;

fn main(){

    env_logger::init();
    let amqp_url = "amqp://guest:guest@127.0.0.1//";
    let mut session = match Session::open_url(amqp_url) {
        Ok(session) => session,
        Err(error) => panic!("Can't create session: {:?}", error)
    };
    let mut channel = session.open_channel(1).ok().expect("Can't open channel");
    println!("Openned channel: {}", channel.id);

    let queue_name = "test_queue";
    //queue: &str, passive: bool, durable: bool, exclusive: bool, auto_delete: bool, nowait: bool, arguments: Table
    let queue_declare = channel.queue_declare(queue_name, false, true, false, false, false, Table::new());
    println!("Queue declare: {:?}", queue_declare);
    for get_result in channel.basic_get(queue_name, false) {
        println!("Headers: {:?}", get_result.headers);
        println!("Reply: {:?}", get_result.reply);
        println!("Body: {:?}", String::from_utf8_lossy(&get_result.body));
        get_result.ack();
    }

    channel.basic_publish("", queue_name, true, false,
        protocol::basic::BasicProperties{ content_type: Some("text".to_string()), ..Default::default()},
        (b"Hello from rust!").to_vec());
    channel.close(200, "Bye");
    session.close(200, "Good Bye");

    // let path = "./src/orders.csv";
    // let input = File::open(path)?;
    // let buffered = BufReader::new(input);
    // for line in buffered.lines() {
        // let row = line?;
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
    

//    let pool = Pool::new("mysql://root:123456@localhost:3306/matching").unwrap();
//
//    let on_trade = |event: TradeEvent| {
//        println!("price: {}, volume: {}", event.price, event.volume);
//        &pool
//            .start_transaction(false, Some(IsolationLevel::RepeatableRead), Some(false))
//            .and_then(|mut tx| {
//                Trade::create(&mut tx, event.price, event.volume, event.ask_order_id, event.bid_order_id);
//
//                Order::sub_volume(&mut tx, event.ask_order_id, event.volume, event.ask_order_filled);
//                Order::add_volume(&mut tx, event.bid_order_id, event.volume, event.bid_order_filled);
//                tx.commit()
//            });
//    };
//
//    let on_cancel = |order_id| {
//        &pool
//            .start_transaction(false, Some(IsolationLevel::RepeatableRead), Some(false))
//            .and_then(|mut tx| {
//                Order::set_canceled(&mut tx, order_id);
//                tx.commit()
//            });
//    };
//
//    let mut order_manager = OrderManager::new(&pool, 8, 8, &on_trade, &on_cancel);
//    // order_manager.submit(0.12, 11.000000035, 0, "u123456");
//    // order_manager.submit(0.13, 11.00000003, 1, "u123456");
//
//    let id = order_manager.submit(0.000003456, 11.000000035, 0, "u123456").unwrap();
//    order_manager.print_orderbook();
//    order_manager.cancel(id, 0.000003456, 11.000000035, 0, "u123456");
//    order_manager.print_orderbook();
//    order_manager.submit(0.000004,    11.0000000315, 1, "u123456");
}
