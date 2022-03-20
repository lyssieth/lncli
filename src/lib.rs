#![warn(clippy::pedantic)]

use color_eyre::Report;

pub type Res<T> = Result<T, Report>;

#[must_use]
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
