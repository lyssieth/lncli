#![warn(clippy::pedantic, clippy::nursery, clippy::perf, clippy::cargo)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::multiple_crate_versions)]
// The above sadly has to be enabled because `dirs` and `parking_lot` (subdependency of `cursive`) depend on different versions of redox_syscall
// and `reqwest` depends on two different versions of `socket2`

use color_eyre::Report;

type Res<T> = Result<T, Report>;

mod app;
mod data;
mod scrape;
mod state;

fn main() -> Res<()> {
    color_eyre::install()?;

    app::run()?;

    Ok(())
}
