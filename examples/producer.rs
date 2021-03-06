extern crate amqp;

use amqp::session::Options;
use amqp::session::Session;
use amqp::protocol;
use amqp::table;
use amqp::basic::Basic;
use std::default::Default;


fn main() {
    let mut session = Session::new(Options{.. Default::default()}).ok().unwrap();
    let mut channel = session.open_channel(1).ok().unwrap();
    println!("Openned channel: {:?}", channel.id);

    let queue_name = "test_queue";
    //queue: &str, passive: bool, durable: bool, exclusive: bool, auto_delete: bool, nowait: bool, arguments: Table
    let queue_declare = channel.queue_declare(queue_name, false, true, false, false, false, table::new());
    println!("Queue declare: {:?}", queue_declare);

    loop {
        channel.basic_publish("", queue_name, true, false,
            protocol::basic::BasicProperties{ content_type: Some("text".to_string()), ..Default::default()},
            (b"Hello from rust!").to_vec());
    }
    channel.close(200, "Bye".to_string());
    session.close(200, "Good Bye".to_string());
}
