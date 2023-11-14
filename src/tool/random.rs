use rand::distributions::{Distribution, Uniform};
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn random_str() -> String {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(0..25);
    (0..3)
        .map(|_| {
            let idx = die.sample(&mut rng);
            char::from(unsafe { *CHARSET.get_unchecked(idx) })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::tool::libtime;

    #[test]
    fn random() {
        let start = libtime::get_now_micros();
        for _ in 0..1000000 {
            //            println!("{}", random_str());
        }
        let end = libtime::get_now_micros();

        println!("{}", end - start);
    }
}
