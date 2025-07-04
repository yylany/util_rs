pub mod client;

#[cfg(feature = "notify")]
pub mod notify;
pub mod tool;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        // 1000XXXUSDT，10000XXXUSDT，1000000XXXUSDT 1MXXXUSDT

        assert_eq!(super::tool::base_trim("1000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("10000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1000000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1MXXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1M10000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1M0000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1M0000XXXUSDT10000"), "XXXUSDT");
    }

    #[tokio::test]
    pub async fn test_tg() {
        use crate::notify::{Config, TgBot};

        let notify = TgBot::new(&Config {
            debug: false,
            token: "".to_string(),
            subscribers: vec!["".to_string()],
        });

        let long_text: String = "a".repeat(9000);

        notify.notify(long_text).await;
    }
}
