pub mod client;
pub mod tool;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let text = "fCm/zsBEoqxvrrBtTKtvmVnTTmjrbbRF5pWlpdVMjpZflH7eyM9Cgz5mm3blQ9po7WnaKqWXrI3YWXko5v6jO6P16UfspFDQjIXJZ0omUBs=";

        let k = b"bQbGOojbKzSOo3CwbQbGOojbKzSOo3Cw";

        let d = tool::aes::aes_32_ecb(text, k).unwrap();
        println!("{}", d);
    }
}
