//! The geometry of a blow landing on a target, as plain functions with no Godot types.
//!
//! Hits on a lane are judged by two things: the attack box overlapping the target's hurtbox
//! horizontally, and the two fighters' feet being at about the same depth. Box height plays no part
//! in that (two fighters at the same depth always overlap vertically); it is kept only so the
//! debug overlay can draw the boxes. Everything here is unit-tested without the engine, like
//! `movement::step`. Attack timing, teams and the full hit rule live in `fighter`.

/// Which way a fighter faces along the lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    /// Towards smaller x.
    Left,
    /// Towards larger x.
    Right,
}

impl Facing {
    /// `-1.0` for left, `1.0` for right: the direction a swing reaches and a knockback pushes.
    pub fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }

    /// The facing after a tick of horizontal input `x`: turns towards a non-zero input, and keeps
    /// the current facing when there is none, so a fighter faces the way it last moved.
    pub fn after_input(self, x: f32) -> Self {
        if x < 0.0 {
            Self::Left
        } else if x > 0.0 {
            Self::Right
        } else {
            self
        }
    }
}

/// An axis-aligned box in screen pixels (y grows downwards).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Smallest x.
    pub left: f32,
    /// Largest x.
    pub right: f32,
    /// Smallest y.
    pub top: f32,
    /// Largest y.
    pub bottom: f32,
}

/// The area a swing reaches: from the fighter's front edge (`half_width` from the feet origin) to
/// `reach` beyond it in the facing direction, and `height` tall above the feet.
pub fn attack_box(
    origin: (f32, f32),
    facing: Facing,
    half_width: f32,
    reach: f32,
    height: f32,
) -> Rect {
    let near = origin.0 + facing.sign() * half_width;
    let far = origin.0 + facing.sign() * (half_width + reach);
    Rect {
        left: near.min(far),
        right: near.max(far),
        top: origin.1 - height,
        bottom: origin.1,
    }
}

/// The area a target can be hit in: `width` wide, centred on the feet origin, `height` tall above
/// the feet.
pub fn hurtbox(origin: (f32, f32), width: f32, height: f32) -> Rect {
    Rect {
        left: origin.0 - width / 2.0,
        right: origin.0 + width / 2.0,
        top: origin.1 - height,
        bottom: origin.1,
    }
}

/// Whether a blow lands: the boxes overlap horizontally (touching edges count) and the feet heights
/// differ by at most `depth_tolerance`.
pub fn lands(
    attack: &Rect,
    hurt: &Rect,
    attacker_feet_y: f32,
    target_feet_y: f32,
    depth_tolerance: f32,
) -> bool {
    attack.left <= hurt.right
        && hurt.left <= attack.right
        && (attacker_feet_y - target_feet_y).abs() <= depth_tolerance
}

/// The targets this tick's blow lands on that the swing has not hit yet. Each target is
/// `(id, hurtbox, feet y)`; `already_hit` lists the ids this swing has hit, so each target is hit
/// at most once per swing however many ticks the boxes overlap.
pub fn new_hits<T: Copy + PartialEq>(
    attack: &Rect,
    attacker_feet_y: f32,
    depth_tolerance: f32,
    targets: &[(T, Rect, f32)],
    already_hit: &[T],
) -> Vec<T> {
    targets
        .iter()
        .filter(|(id, hurt, feet_y)| {
            !already_hit.contains(id)
                && lands(attack, hurt, attacker_feet_y, *feet_y, depth_tolerance)
        })
        .map(|(id, _, _)| *id)
        .collect()
}

/// How far a knocked-back target has slid after `elapsed` seconds: it covers `distance` in `time`,
/// slowing evenly to rest, so the slide ends exactly `distance` away whatever the frame times.
pub fn knockback_offset(distance: f32, time: f32, elapsed: f32) -> f32 {
    if distance <= 0.0 {
        return 0.0;
    }
    if time <= 0.0 {
        return distance;
    }
    let remaining = 1.0 - (elapsed / time).clamp(0.0, 1.0);
    distance * (1.0 - remaining * remaining)
}

/// Keeps a feet position on the screen (`half_width` from each side) and inside the lane.
pub fn clamp_to_area(
    position: (f32, f32),
    half_width: f32,
    screen_width: f32,
    lane_top: f32,
    lane_bottom: f32,
) -> (f32, f32) {
    (
        position
            .0
            .clamp(half_width, (screen_width - half_width).max(half_width)),
        position.1.clamp(lane_top, lane_bottom),
    )
}

/// The freeze time left after asking for `new` seconds while `remaining` are left: the longer of
/// the two, never the sum, so hits close together do not stack into a long stall.
pub fn extend_freeze(remaining: f32, new: f32) -> f32 {
    remaining.max(new)
}

/// Where a new training dummy should stand: in front of the fighter at its depth, with its far edge
/// at the end of the reach; if a dummy already stands there (`others_x`, within half a width), the
/// spot steps half a width further along the facing until it is free. Kept on the screen.
pub fn spawn_point(
    origin: (f32, f32),
    facing: Facing,
    half_width: f32,
    reach: f32,
    dummy_width: f32,
    others_x: &[f32],
    screen_width: f32,
) -> (f32, f32) {
    let half = dummy_width / 2.0;
    let step = facing.sign() * half;
    let mut x = origin.0 + facing.sign() * (half_width + reach - half);
    while half > 0.0
        && (half..=screen_width - half).contains(&x)
        && others_x.iter().any(|other| (other - x).abs() < half)
    {
        x += step;
    }
    (x.clamp(half, (screen_width - half).max(half)), origin.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values deliberately different from the starting values in tuning.json.
    const HALF_WIDTH: f32 = 20.0;
    const REACH: f32 = 50.0;
    const HEIGHT: f32 = 40.0;
    const TOLERANCE: f32 = 10.0;
    const SCREEN_WIDTH: f32 = 1000.0;

    fn hurt_at(x: f32, y: f32) -> Rect {
        hurtbox((x, y), 30.0, 100.0)
    }

    #[test]
    fn attack_box_is_mirrored_by_facing() {
        let right = attack_box((100.0, 500.0), Facing::Right, HALF_WIDTH, REACH, HEIGHT);
        let left = attack_box((100.0, 500.0), Facing::Left, HALF_WIDTH, REACH, HEIGHT);
        assert_eq!((right.left, right.right), (120.0, 170.0));
        assert_eq!((left.left, left.right), (30.0, 80.0));
        assert_eq!((right.top, right.bottom), (460.0, 500.0));
        assert_eq!((left.top, left.bottom), (460.0, 500.0));
    }

    #[test]
    fn hurtbox_is_centred_on_the_feet() {
        let hurt = hurtbox((300.0, 600.0), 40.0, 90.0);
        assert_eq!(
            (hurt.left, hurt.right, hurt.top, hurt.bottom),
            (280.0, 320.0, 510.0, 600.0)
        );
    }

    #[test]
    fn lands_needs_horizontal_overlap_and_close_depth() {
        let attack = attack_box((100.0, 500.0), Facing::Right, HALF_WIDTH, REACH, HEIGHT);
        // Overlapping at the same depth.
        assert!(lands(
            &attack,
            &hurt_at(160.0, 500.0),
            500.0,
            500.0,
            TOLERANCE
        ));
        // Touching edges count: hurtbox left edge at 170.
        assert!(lands(
            &attack,
            &hurt_at(185.0, 500.0),
            500.0,
            500.0,
            TOLERANCE
        ));
        // Exactly at the tolerance lands; just beyond misses.
        assert!(lands(
            &attack,
            &hurt_at(160.0, 510.0),
            500.0,
            510.0,
            TOLERANCE
        ));
        assert!(!lands(
            &attack,
            &hurt_at(160.0, 510.5),
            500.0,
            510.5,
            TOLERANCE
        ));
        // Apart horizontally.
        assert!(!lands(
            &attack,
            &hurt_at(200.0, 500.0),
            500.0,
            500.0,
            TOLERANCE
        ));
        // Behind the hero.
        assert!(!lands(
            &attack,
            &hurt_at(60.0, 500.0),
            500.0,
            500.0,
            TOLERANCE
        ));
    }

    #[test]
    fn zero_tolerance_lands_only_at_equal_depth() {
        let attack = attack_box((100.0, 500.0), Facing::Right, HALF_WIDTH, REACH, HEIGHT);
        assert!(lands(&attack, &hurt_at(160.0, 500.0), 500.0, 500.0, 0.0));
        assert!(!lands(&attack, &hurt_at(160.0, 501.0), 500.0, 501.0, 0.0));
    }

    #[test]
    fn facing_follows_the_last_horizontal_input() {
        assert_eq!(Facing::Right.after_input(-0.5), Facing::Left);
        assert_eq!(Facing::Left.after_input(0.7), Facing::Right);
        assert_eq!(Facing::Left.after_input(0.0), Facing::Left);
        assert_eq!(Facing::Right.sign(), 1.0);
        assert_eq!(Facing::Left.sign(), -1.0);
    }

    #[test]
    fn one_swing_hits_each_target_once_even_when_overlapping_for_many_ticks() {
        let attack = attack_box((100.0, 500.0), Facing::Right, HALF_WIDTH, REACH, HEIGHT);
        let targets = [(7_u64, hurt_at(160.0, 500.0), 500.0)];
        let mut hit: Vec<u64> = Vec::new();
        for _ in 0..6 {
            let new = new_hits(&attack, 500.0, TOLERANCE, &targets, &hit);
            hit.extend(new);
        }
        assert_eq!(hit, vec![7]);
    }

    #[test]
    fn one_swing_hits_every_target_in_reach() {
        let attack = attack_box((100.0, 500.0), Facing::Right, HALF_WIDTH, REACH, HEIGHT);
        let targets = [
            (1_u64, hurt_at(150.0, 500.0), 500.0),
            (2_u64, hurt_at(170.0, 505.0), 505.0),
            (3_u64, hurt_at(170.0, 540.0), 540.0),
        ];
        assert_eq!(
            new_hits(&attack, 500.0, TOLERANCE, &targets, &[]),
            vec![1, 2]
        );
    }

    #[test]
    fn knockback_slows_evenly_and_ends_at_the_distance() {
        assert_eq!(knockback_offset(60.0, 0.2, 0.0), 0.0);
        assert_eq!(knockback_offset(60.0, 0.2, 0.2), 60.0);
        assert_eq!(knockback_offset(60.0, 0.2, 5.0), 60.0);
        assert_eq!(knockback_offset(0.0, 0.2, 0.1), 0.0);
        // Halfway through the time, three quarters of the distance is covered.
        assert!((knockback_offset(60.0, 0.2, 0.1) - 45.0).abs() < 1e-4);
        let mut previous = 0.0;
        for step in 1..=20 {
            let offset = knockback_offset(60.0, 0.2, step as f32 * 0.01);
            assert!(offset >= previous, "not monotonic at {step}");
            previous = offset;
        }
    }

    #[test]
    fn clamping_keeps_a_dummy_inside_the_lane_and_the_screen() {
        assert_eq!(
            clamp_to_area((-50.0, 500.0), 20.0, SCREEN_WIDTH, 400.0, 650.0),
            (20.0, 500.0)
        );
        assert_eq!(
            clamp_to_area((990.0, 500.0), 20.0, SCREEN_WIDTH, 400.0, 650.0),
            (980.0, 500.0)
        );
        assert_eq!(
            clamp_to_area((500.0, 300.0), 20.0, SCREEN_WIDTH, 400.0, 650.0),
            (500.0, 400.0)
        );
        assert_eq!(
            clamp_to_area((500.0, 700.0), 20.0, SCREEN_WIDTH, 400.0, 650.0),
            (500.0, 650.0)
        );
    }

    #[test]
    fn a_new_freeze_takes_the_longer_time_never_the_sum() {
        assert_eq!(extend_freeze(0.05, 0.03), 0.05);
        assert_eq!(extend_freeze(0.02, 0.06), 0.06);
        assert_eq!(extend_freeze(0.0, 0.0), 0.0);
    }

    #[test]
    fn spawn_point_starts_in_reach_and_steps_along_the_facing() {
        let origin = (100.0, 500.0);
        let width = 30.0;
        // Front edge at 120, reach ends at 170: centre at 155 so the far edge sits at 170.
        let first = spawn_point(
            origin,
            Facing::Right,
            HALF_WIDTH,
            REACH,
            width,
            &[],
            SCREEN_WIDTH,
        );
        assert_eq!(first, (155.0, 500.0));
        let second = spawn_point(
            origin,
            Facing::Right,
            HALF_WIDTH,
            REACH,
            width,
            &[155.0],
            1e4,
        );
        assert_eq!(second, (170.0, 500.0));
        let third = spawn_point(
            origin,
            Facing::Right,
            HALF_WIDTH,
            REACH,
            width,
            &[155.0, 170.0],
            1e4,
        );
        assert_eq!(third, (185.0, 500.0));
        let left = spawn_point(origin, Facing::Left, HALF_WIDTH, REACH, width, &[45.0], 1e4);
        assert_eq!(left, (30.0, 500.0));
    }

    #[test]
    fn spawn_point_stays_on_screen() {
        let crowded = spawn_point(
            (980.0, 500.0),
            Facing::Right,
            HALF_WIDTH,
            REACH,
            30.0,
            &[],
            1000.0,
        );
        assert_eq!(crowded, (985.0, 500.0));
    }
}
