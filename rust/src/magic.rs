//! The hero's magic: the meter of charges and the spell that spends them, as pure rules tested
//! without the engine. The hero holds one meter and casts; these are the rules it follows.

use crate::combat::Rect;

/// One hero's magic charges, from 0 to a maximum. Each hero holds its own, so it belongs to that
/// hero's player index. A fresh meter is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagicMeter {
    charges: u32,
    max: u32,
}

impl MagicMeter {
    /// An empty meter holding at most `max` charges.
    pub fn new(max: u32) -> Self {
        Self { charges: 0, max }
    }

    /// Charges held.
    pub fn charges(&self) -> u32 {
        self.charges
    }

    /// The most charges it can hold.
    pub fn max(&self) -> u32 {
        self.max
    }

    /// Whether it holds the most it can.
    pub fn is_full(&self) -> bool {
        self.charges >= self.max
    }

    /// Whether it holds none.
    pub fn is_empty(&self) -> bool {
        self.charges == 0
    }

    /// Adds one charge unless full; true if it changed.
    pub fn add_charge(&mut self) -> bool {
        if self.is_full() {
            return false;
        }
        self.charges += 1;
        true
    }

    /// Fills the meter (the F9 cheat).
    pub fn refill(&mut self) {
        self.charges = self.max;
    }

    /// Uses every charge: returns how many there were and leaves none.
    pub fn spend_all(&mut self) -> u32 {
        std::mem::take(&mut self.charges)
    }

    /// Takes a new maximum from a tuning reload; charges above it are cut, none are added.
    pub fn set_max(&mut self, max: u32) {
        self.max = max;
        self.charges = self.charges.min(max);
    }
}

/// How a cast's damage is worked out from the charges it spends (`magic.damage_mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellDamageMode {
    /// The damage per charge times the charges spent.
    PerCharge,
    /// The same damage however many charges were spent.
    Flat,
}

/// The damage a cast does to each enemy it hits: the damage per charge times the charges spent,
/// or the flat damage whatever the charges, by `mode`. Every charge is spent either way.
pub fn spell_damage(mode: SpellDamageMode, per_charge: f32, flat: f32, charges: u32) -> f32 {
    match mode {
        SpellDamageMode::PerCharge => per_charge * charges as f32,
        SpellDamageMode::Flat => flat,
    }
}

/// Whether a fighter's feet are inside `screen` (edges count). The spell hits only those, so an
/// enemy still walking in from beyond an edge is spared (spec, planning answer).
pub fn on_screen(feet: (f32, f32), screen: &Rect) -> bool {
    (screen.left..=screen.right).contains(&feet.0) && (screen.top..=screen.bottom).contains(&feet.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values deliberately different from the starting values in tuning.json.
    const MAX: u32 = 4;

    fn screen() -> Rect {
        Rect {
            left: 0.0,
            right: 1000.0,
            top: 0.0,
            bottom: 600.0,
        }
    }

    #[test]
    fn per_charge_damage_grows_with_the_charges_and_flat_damage_does_not() {
        assert_eq!(
            spell_damage(SpellDamageMode::PerCharge, 15.0, 35.0, 1),
            15.0
        );
        assert_eq!(
            spell_damage(SpellDamageMode::PerCharge, 15.0, 35.0, 3),
            45.0
        );
        assert_eq!(spell_damage(SpellDamageMode::Flat, 15.0, 35.0, 1), 35.0);
        assert_eq!(spell_damage(SpellDamageMode::Flat, 15.0, 35.0, 3), 35.0);
    }

    #[test]
    fn only_feet_inside_the_screen_are_on_screen() {
        assert!(on_screen((500.0, 300.0), &screen()));
        assert!(on_screen((0.0, 600.0), &screen()), "edges count");
        assert!(!on_screen((-1.0, 300.0), &screen()), "beyond the left edge");
        assert!(
            !on_screen((1001.0, 300.0), &screen()),
            "beyond the right edge"
        );
        assert!(!on_screen((500.0, 601.0), &screen()), "below the bottom");
        assert!(!on_screen((500.0, -1.0), &screen()), "above the top");
    }

    #[test]
    fn a_new_meter_is_empty() {
        let meter = MagicMeter::new(MAX);
        assert_eq!((meter.charges(), meter.max()), (0, MAX));
        assert!(meter.is_empty());
        assert!(!meter.is_full());
    }

    #[test]
    fn charges_stop_at_the_maximum() {
        let mut meter = MagicMeter::new(MAX);
        for _ in 0..MAX {
            assert!(meter.add_charge());
        }
        assert!(meter.is_full());
        assert!(!meter.add_charge(), "full: unchanged");
        assert_eq!(meter.charges(), MAX);
    }

    #[test]
    fn spend_all_returns_every_charge_and_leaves_none() {
        let mut meter = MagicMeter::new(MAX);
        meter.add_charge();
        meter.add_charge();
        assert_eq!(meter.spend_all(), 2);
        assert!(meter.is_empty());
        assert_eq!(meter.spend_all(), 0, "nothing left to spend");
    }

    #[test]
    fn refill_fills_the_meter() {
        let mut meter = MagicMeter::new(MAX);
        meter.refill();
        assert_eq!(meter.charges(), MAX);
    }

    #[test]
    fn a_lower_maximum_cuts_the_charges_and_a_higher_one_adds_none() {
        let mut meter = MagicMeter::new(MAX);
        meter.refill();
        meter.set_max(2);
        assert_eq!((meter.charges(), meter.max()), (2, 2));
        meter.set_max(6);
        assert_eq!((meter.charges(), meter.max()), (2, 6));
    }
}
