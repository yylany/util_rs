pub mod client;
pub mod notify;
pub mod tool;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
