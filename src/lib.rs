pub mod client;

#[cfg(feature = "notify")]
pub mod notify;
pub mod tool;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(super::tool::base_trim("1000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("100XXXUSDT"), "100XXXUSDT");
        assert_eq!(super::tool::base_trim("10000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1000000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1MXXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1M10000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1M0000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1M0000XXXUSDT10000"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("0XXXUSDT10000"), "0XXXUSDT");
    }
}
