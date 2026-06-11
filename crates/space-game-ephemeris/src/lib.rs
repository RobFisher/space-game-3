//! Ephemeris calculations and data structures for space simulations.

/// Returns the library name.
pub fn crate_name() -> &'static str {
    "space_game_ephemeris"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_crate_name() {
        assert_eq!(crate_name(), "space_game_ephemeris");
    }
}
