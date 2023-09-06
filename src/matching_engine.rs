use std::collections::{BTreeMap, HashMap, VecDeque};

#[derive(Debug, PartialEq, Clone)]
pub enum Side {
    BID,
    ASK,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub symbol: String,
    pub side: Side,
    pub order_id: u64,
    pub price: u64,
    pub volume: u64,
}

type Orders = VecDeque<Order>;
type OrderBookSide = BTreeMap<u64, Orders>;

#[derive(Debug, Default)]
pub struct OrderBook {
    ask_side: OrderBookSide,
    bid_side: OrderBookSide,
}

#[derive(Debug, Default)]
pub struct MatchingEngine {
    markets: HashMap<String, OrderBook>,
    order_map: HashMap<u64, Order>,
}

impl MatchingEngine {
    pub fn insert(&mut self, mut order: Order) -> Vec<Order> {
        let mut trades = Vec::new();
        if order.side == Side::BID {
            self.match_bid_order(&mut order, &mut trades);
            self.insert_bid_order(order);
        } else {
            self.match_ask_order(&mut order, &mut trades);
            self.insert_ask_order(order);
        }
        for trade in trades.iter() {
            if let Some(map_order) = self.order_map.get_mut(&trade.order_id) {
                if map_order.volume == trade.volume {
                    self.order_map.remove(&trade.order_id);
                } else {
                    map_order.volume -= trade.volume;
                }
            }
        }
        trades
    }

    pub fn amend(&mut self, order_id: u64, new_price: u64, new_volume: u64) -> Vec<Order> {
        if let Some(order) = self.order_map.get(&order_id) {
            if let Some(order_book) = self.markets.get_mut(&order.symbol) {
                match order.side {
                    Side::BID => {
                        if let Some(order_queue) = order_book.bid_side.get_mut(&order.price) {
                            if let Some(order_in_book) = order_queue
                                .iter_mut()
                                .find(|o| o.order_id == order.order_id)
                            {
                                if new_price == order_in_book.price
                                    && new_volume <= order_in_book.volume
                                {
                                    order_in_book.volume = new_volume;
                                } else {
                                    let mut new_order = order_in_book.clone();
                                    new_order.price = new_price;
                                    new_order.volume = new_volume;
                                    self.cancel(order.order_id);
                                    return self.insert(new_order);
                                }
                            }
                        }
                    }
                    Side::ASK => {
                        if let Some(order_queue) = order_book.ask_side.get_mut(&order.price) {
                            if let Some(order_in_book) = order_queue
                                .iter_mut()
                                .find(|o| o.order_id == order.order_id)
                            {
                                if new_price == order_in_book.price
                                    && new_volume <= order_in_book.volume
                                {
                                    order_in_book.volume = new_volume;
                                } else {
                                    let mut new_order = order_in_book.clone();
                                    new_order.price = new_price;
                                    new_order.volume = new_volume;
                                    self.cancel(order.order_id);
                                    return self.insert(new_order);
                                }
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }

    pub fn cancel(&mut self, order_id: u64) {
        if let Some(order) = self.order_map.remove(&order_id) {
            if let Some(order_book) = self.markets.get_mut(&order.symbol) {
                match order.side {
                    Side::BID => Self::cancel_order(&mut order_book.bid_side, &order),
                    Side::ASK => Self::cancel_order(&mut order_book.ask_side, &order),
                }
            }
        }
    }

    fn cancel_order(order_book_side: &mut OrderBookSide, order: &Order) {
        if let Some(order_queue) = order_book_side.get_mut(&order.price) {
            if let Some(index) = order_queue
                .iter()
                .position(|o| o.order_id == order.order_id)
            {
                order_queue.remove(index);
            }
            if order_queue.is_empty() {
                order_book_side.remove(&order.price);
            }
        }
    }

    fn match_bid_order(&mut self, order: &mut Order, trades: &mut Vec<Order>) {
        let mut top = self
            .get_ask_side(&order.symbol)
            .lower_bound_mut(std::ops::Bound::Unbounded);

        while let Some((price, order_queue)) = top.key_value_mut() {
            if order.price < *price || order.volume == 0 {
                break;
            }

            Self::match_order_queue(order, order_queue, trades);

            if order_queue.is_empty() {
                top.remove_current();
            } else {
                break;
            }
        }
    }

    fn match_ask_order(&mut self, order: &mut Order, trades: &mut Vec<Order>) {
        let mut top = self
            .get_bid_side(&order.symbol)
            .upper_bound_mut(std::ops::Bound::Unbounded);

        while let Some((price, order_queue)) = top.key_value_mut() {
            if order.price > *price || order.volume == 0 {
                break;
            }

            Self::match_order_queue(order, order_queue, trades);

            if order_queue.is_empty() {
                top.remove_current_and_move_back();
            } else {
                break;
            }
        }
    }

    fn insert_bid_order(&mut self, order: Order) {
        if order.volume > 0 {
            self.get_bid_side(&order.symbol)
                .entry(order.price)
                .or_default()
                .push_back(order.clone());
            self.order_map.insert(order.order_id, order);
        }
    }

    fn insert_ask_order(&mut self, order: Order) {
        if order.volume > 0 {
            self.get_ask_side(&order.symbol)
                .entry(order.price)
                .or_default()
                .push_back(order.clone());
            self.order_map.insert(order.order_id, order);
        }
    }

    fn get_ask_side(&mut self, symbol: &String) -> &mut OrderBookSide {
        &mut self.markets.entry(symbol.clone()).or_default().ask_side
    }

    fn get_bid_side(&mut self, symbol: &String) -> &mut OrderBookSide {
        &mut self.markets.entry(symbol.clone()).or_default().bid_side
    }

    fn match_order_queue(
        order: &mut Order,
        order_queue: &mut VecDeque<Order>,
        trades: &mut Vec<Order>,
    ) {
        while let Some(order_queue_it) = order_queue.front_mut() {
            let match_size = std::cmp::min(order.volume, order_queue_it.volume);

            trades.push(order_queue_it.clone());
            trades.last_mut().unwrap().volume = match_size;
            order_queue_it.volume -= match_size;
            order.volume -= match_size;

            if order_queue_it.volume == 0 {
                order_queue.pop_front();
            }
            if order.volume == 0 {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    pub fn insert_bid_order() {
        let mut engine = super::MatchingEngine::default();

        let bid_order = super::Order {
            symbol: "AAPL".to_string(),
            side: super::Side::BID,
            order_id: 1,
            price: 123,
            volume: 10,
        };

        let trades = engine.insert(bid_order);

        assert!(trades.is_empty());
    }

    #[test]
    pub fn match_order() {
        let mut engine = super::MatchingEngine::default();

        let ask_order = super::Order {
            symbol: "AAPL".to_string(),
            side: super::Side::ASK,
            order_id: 1,
            price: 123,
            volume: 10,
        };

        let trades = engine.insert(ask_order.clone());
        assert!(trades.is_empty());

        let bid_order = super::Order {
            symbol: "AAPL".to_string(),
            side: super::Side::BID,
            order_id: 2,
            price: 123,
            volume: 20,
        };

        let trades = engine.insert(bid_order.clone());
        assert!(!trades.is_empty());
        assert_eq!(trades, vec![ask_order]);

        let ask_order_2 = super::Order {
            symbol: "AAPL".to_string(),
            side: super::Side::ASK,
            order_id: 3,
            price: 123,
            volume: 10,
        };

        let expected_trade = super::Order {
            symbol: "AAPL".to_string(),
            side: super::Side::BID,
            order_id: 2,
            price: 123,
            volume: 10,
        };

        let trades = engine.insert(ask_order_2.clone());
        assert!(!trades.is_empty());
        assert_eq!(trades, vec![expected_trade]);
    }

    #[test]
    pub fn test_cancel_order() {
        let mut engine = super::MatchingEngine::default();

        let order_id_1: u64 = 1;
        let order_id_2: u64 = 2;
        let ask_order = super::Order {
            symbol: "AAPL".to_string(),
            side: super::Side::ASK,
            order_id: order_id_1,
            price: 123,
            volume: 10,
        };

        let bid_order = super::Order {
            symbol: "AAPL".to_string(),
            side: super::Side::BID,
            order_id: order_id_2,
            price: 123,
            volume: 20,
        };

        engine.insert(ask_order);
        engine.cancel(order_id_1);

        let trades = engine.insert(bid_order);
        assert!(trades.is_empty());

        engine.cancel(order_id_2);
    }
}
