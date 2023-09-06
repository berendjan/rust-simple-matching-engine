#![feature(btree_cursors)]
#![feature(map_try_insert)]
pub mod matching_engine;
mod parser;

#[cfg(test)]
mod test {
    use crate::parser::run;

    #[test]
    pub fn insert() {
        let input = vec!["INSERT,1,AAPL,BUY,12.2,5".to_string()];

        let result = run(input);

        assert_eq!(result.len(), 0);
    }

    #[test]
    pub fn simple_match_sell_is_aggressive() {
        let input = vec![
            "INSERT,1,AAPL,BUY,12.2,5".to_string(),
            "INSERT,2,AAPL,SELL,12.1,8".to_string(),
        ];

        let result = run(input);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "AAPL,12.2,5,2,1".to_string());
    }

    #[test]
    pub fn simple_match_buy_is_aggressive() {
        let input = vec![
            "INSERT,1,AAPL,SELL,12.1,8".to_string(),
            "INSERT,2,AAPL,BUY,12.2,5".to_string(),
        ];

        let result = run(input);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "AAPL,12.1,5,2,1".to_string());
    }

    #[test]
    pub fn multi_insert_and_multi_match() {
        let input = vec![
            "INSERT,8,AAPL,BUY,14.235,5".to_string(),
            "INSERT,6,AAPL,BUY,14.235,6".to_string(),
            "INSERT,7,AAPL,BUY,14.235,12".to_string(),
            "INSERT,2,AAPL,BUY,14.234,5".to_string(),
            "INSERT,1,AAPL,BUY,14.23,3".to_string(),
            "INSERT,5,AAPL,SELL,14.237,8".to_string(),
            "INSERT,3,AAPL,SELL,14.24,9".to_string(),
            "PULL,8".to_string(),
            "INSERT,4,AAPL,SELL,14.234,25".to_string(),
        ];

        let result = run(input);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "AAPL,14.235,6,4,6".to_string());
        assert_eq!(result[1], "AAPL,14.235,12,4,7".to_string());
        assert_eq!(result[2], "AAPL,14.234,5,4,2".to_string());
    }

    #[test]
    pub fn multi_symbol() {
        let input = vec![
            "INSERT,1,WEBB,BUY,0.3854,5".to_string(),
            "INSERT,2,TSLA,BUY,412,31".to_string(),
            "INSERT,3,TSLA,BUY,410.5,27".to_string(),
            "INSERT,4,AAPL,SELL,21,8".to_string(),
            "INSERT,11,WEBB,SELL,0.3854,4".to_string(),
            "INSERT,13,WEBB,SELL,0.3853,6".to_string(),
        ];

        let result = run(input);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "WEBB,0.3854,4,11,1".to_string());
        assert_eq!(result[1], "WEBB,0.3854,1,13,1".to_string());
    }

    #[test]
    pub fn amend() {
        let input = vec![
            "INSERT,1,WEBB,BUY,45.95,5".to_string(),
            "INSERT,2,WEBB,BUY,45.95,6".to_string(),
            "INSERT,3,WEBB,BUY,45.95,12".to_string(),
            "INSERT,4,WEBB,SELL,46,8".to_string(),
            "AMEND,2,46,3".to_string(),
            "INSERT,5,WEBB,SELL,45.95,1".to_string(),
            "AMEND,1,45.95,3".to_string(),
            "INSERT,6,WEBB,SELL,45.95,1".to_string(),
            "AMEND,1,45.95,5".to_string(),
            "INSERT,7,WEBB,SELL,45.95,1".to_string(),
        ];

        let result = run(input);

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "WEBB,46,3,2,4".to_string());
        assert_eq!(result[1], "WEBB,45.95,1,5,1".to_string());
        assert_eq!(result[2], "WEBB,45.95,1,6,1".to_string());
        assert_eq!(result[3], "WEBB,45.95,1,7,3".to_string());
    }

    #[test]
    pub fn slam_order_book_bid() {
        let input = vec![
            "INSERT,1,WEBB,BUY,45.95,3".to_string(),
            "INSERT,2,WEBB,BUY,45.95,3".to_string(),
            "INSERT,3,WEBB,BUY,45.95,300".to_string(),
            "INSERT,4,WEBB,BUY,45.96,3".to_string(),
            "INSERT,5,WEBB,BUY,45.96,3".to_string(),
            "INSERT,6,WEBB,BUY,45.96,300".to_string(),
            "INSERT,7,WEBB,BUY,45.97,3".to_string(),
            "INSERT,8,WEBB,BUY,45.97,3".to_string(),
            "INSERT,9,WEBB,BUY,45.97,300".to_string(),
            "INSERT,10,WEBB,BUY,45,300".to_string(),
            "INSERT,11,WEBB,SELL,46,8".to_string(),
            "AMEND,11,46,1000".to_string(),
            "AMEND,11,45.0001,1000".to_string(),
        ];

        let result = run(input);

        assert_eq!(result.len(), 9);
        assert_eq!(result[0], "WEBB,45.97,3,11,7".to_string());
        assert_eq!(result[1], "WEBB,45.97,3,11,8".to_string());
        assert_eq!(result[2], "WEBB,45.97,300,11,9".to_string());
        assert_eq!(result[3], "WEBB,45.96,3,11,4".to_string());
        assert_eq!(result[4], "WEBB,45.96,3,11,5".to_string());
        assert_eq!(result[5], "WEBB,45.96,300,11,6".to_string());
        assert_eq!(result[6], "WEBB,45.95,3,11,1".to_string());
        assert_eq!(result[7], "WEBB,45.95,3,11,2".to_string());
        assert_eq!(result[8], "WEBB,45.95,300,11,3".to_string());
    }

    #[test]
    pub fn slam_order_book_ask() {
        let input = vec![
            "INSERT,1,WEBB,SELL,45.95,3".to_string(),
            "INSERT,2,WEBB,SELL,45.95,3".to_string(),
            "INSERT,3,WEBB,SELL,45.95,300".to_string(),
            "INSERT,4,WEBB,SELL,45.96,3".to_string(),
            "INSERT,5,WEBB,SELL,45.96,3".to_string(),
            "INSERT,6,WEBB,SELL,45.96,300".to_string(),
            "INSERT,7,WEBB,SELL,45.97,3".to_string(),
            "INSERT,8,WEBB,SELL,45.97,3".to_string(),
            "INSERT,9,WEBB,SELL,45.97,300".to_string(),
            "INSERT,10,WEBB,SELL,45.999,300".to_string(),
            "INSERT,11,WEBB,BUY,45,8".to_string(),
            "AMEND,11,45,1000".to_string(),
            "AMEND,11,45.9901,1000".to_string(),
        ];

        let result = run(input);

        assert_eq!(result.len(), 9);
        assert_eq!(result[0], "WEBB,45.95,3,11,1".to_string());
        assert_eq!(result[1], "WEBB,45.95,3,11,2".to_string());
        assert_eq!(result[2], "WEBB,45.95,300,11,3".to_string());
        assert_eq!(result[3], "WEBB,45.96,3,11,4".to_string());
        assert_eq!(result[4], "WEBB,45.96,3,11,5".to_string());
        assert_eq!(result[5], "WEBB,45.96,300,11,6".to_string());
        assert_eq!(result[6], "WEBB,45.97,3,11,7".to_string());
        assert_eq!(result[7], "WEBB,45.97,3,11,8".to_string());
        assert_eq!(result[8], "WEBB,45.97,300,11,9".to_string());
    }
}
