use crate::matching_engine::{MatchingEngine, Order, Side};

enum Command {
    Insert(Order),
    Amend(u64, u64, u64),
    Cancel(u64),
}

#[allow(unused)]
pub fn run(input: Vec<String>) -> Vec<String> {
    let mut engine = MatchingEngine::default();
    let out = input
        .iter()
        .flat_map(|line| match parse(line) {
            Command::Insert(order) => parse_trade(engine.insert(order.clone()), order.order_id),
            Command::Amend(order_id, price, volume) => {
                parse_trade(engine.amend(order_id, price, volume), order_id)
            }
            Command::Cancel(order_id) => {
                engine.cancel(order_id);
                Vec::new()
            }
        })
        .collect();
    println!("{:?}", engine);
    out
}

fn parse_trade(trades: Vec<Order>, taker_order_id: u64) -> Vec<String> {
    trades
        .iter()
        .map(|trade| {
            format!(
                "{},{},{},{},{}",
                trade.symbol,
                ((trade.price as f64) / 10_000f64),
                trade.volume,
                taker_order_id,
                trade.order_id
            )
        })
        .collect()
}

///
/// "INSERT,1,AAPL,BUY,12.2,5"
/// "PULL,8"
/// "AMEND,2,46,3"
///
fn parse(input: &String) -> Command {
    let mut iter = input.split(",");
    match iter.next() {
        Some("INSERT") => {
            let order_id = iter.next().unwrap().parse::<u64>().unwrap();
            let symbol = iter.next().unwrap();
            let side = if iter.next().unwrap() == "BUY" {
                Side::BID
            } else {
                Side::ASK
            };
            let price = (iter.next().unwrap().parse::<f64>().unwrap() * 10_000 as f64) as u64;
            let volume = iter.next().unwrap().parse::<u64>().unwrap();
            Command::Insert(Order {
                symbol: symbol.to_string(),
                side: side,
                order_id: order_id,
                price: price,
                volume: volume,
            })
        }
        Some("AMEND") => {
            let order_id = iter.next().unwrap().parse::<u64>().unwrap();
            let price = (iter.next().unwrap().parse::<f64>().unwrap() * 10_000 as f64) as u64;
            let volume = iter.next().unwrap().parse::<u64>().unwrap();
            Command::Amend(order_id, price, volume)
        }
        Some("PULL") => {
            let order_id = iter.next().unwrap().parse::<u64>().unwrap();
            Command::Cancel(order_id)
        }
        _ => panic!("What?"),
    }
}
