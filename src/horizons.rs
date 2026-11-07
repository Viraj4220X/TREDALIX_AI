#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Horizon {
    pub name: &'static str,
    pub seconds: u64,
}

pub const HORIZONS: [Horizon; 9] = [
    Horizon {
        name: "30 seconds",
        seconds: 30,
    },
    Horizon {
        name: "1 minute",
        seconds: 60,
    },
    Horizon {
        name: "2 minutes",
        seconds: 120,
    },
    Horizon {
        name: "3 minutes",
        seconds: 180,
    },
    Horizon {
        name: "5 minutes",
        seconds: 300,
    },
    Horizon {
        name: "7 minutes",
        seconds: 420,
    },
    Horizon {
        name: "10 minutes",
        seconds: 600,
    },
    Horizon {
        name: "20 minutes",
        seconds: 1200,
    },
    Horizon {
        name: "1 hour",
        seconds: 3600,
    },
];

pub fn show_horizons() {
    println!("\n===== PREDICTION HORIZONS =====");

    for horizon in HORIZONS {
        println!("{:<12} ({} seconds)", horizon.name, horizon.seconds);
    }

    println!("Forecast status: CONFIGURED, NOT YET VALIDATED");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_nine_horizons_are_configured() {
        assert_eq!(HORIZONS.len(), 9);
    }

    #[test]
    fn horizons_are_in_ascending_order() {
        for pair in HORIZONS.windows(2) {
            assert!(pair[0].seconds < pair[1].seconds);
        }
    }

    #[test]
    fn shortest_and_longest_horizons_are_correct() {
        assert_eq!(HORIZONS[0].seconds, 30);
        assert_eq!(HORIZONS[8].seconds, 3600);
    }
}
