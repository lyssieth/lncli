#![warn(clippy::pedantic)]

pub fn do_stuff() -> bool {
    println!("Doing...");
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn do_stuff_test() {
        assert!(crate::do_stuff());
    }
}
