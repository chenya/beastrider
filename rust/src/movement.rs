//! The hero's movement rule as a pure function, kept free of Godot types so `cargo test` can
//! check the parts that are easy to get subtly wrong: diagonals, depth speed and the limits.

/// How fast a character walks along each axis, in pixels per second.
///
/// The two speeds differ so that stepping deeper into the lane (vertical) reads as depth rather
/// than as a flat plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Speeds {
    /// Speed along the lane, left and right.
    pub horizontal: f32,
    /// Speed into and out of the lane, up and down the screen.
    pub vertical: f32,
}

/// The box a character's origin (its feet) must stay inside, in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    /// Smallest allowed x.
    pub min_x: f32,
    /// Largest allowed x.
    pub max_x: f32,
    /// Smallest allowed y: the far edge of the lane.
    pub min_y: f32,
    /// Largest allowed y: the near edge of the lane.
    pub max_y: f32,
}

/// Returns the position after one tick of movement.
///
/// `input` is the direction held this tick, with a length of at most 1 (Godot's
/// `Input.get_vector` already normalises diagonals). There is no stored velocity, so movement
/// starts and stops on the tick the input changes. The result is clamped after moving, so holding
/// into a limit lands exactly on it every tick instead of shaking, and a position that starts
/// outside the limits is pulled to the nearest point inside. Holding a diagonal while standing
/// on a limit slides along it at full speed (see [`slide_along_limits`]).
pub fn step(
    position: (f32, f32),
    input: (f32, f32),
    speeds: Speeds,
    delta: f32,
    limits: Limits,
) -> (f32, f32) {
    let input = slide_along_limits(position, input, limits);
    let x = position.0 + input.0 * speeds.horizontal * delta;
    let y = position.1 + input.1 * speeds.vertical * delta;
    (
        x.clamp(limits.min_x, limits.max_x),
        y.clamp(limits.min_y, limits.max_y),
    )
}

/// Removes the part of `input` that pushes into a limit the position is already on, and gives
/// its share to the other axis.
///
/// A held diagonal arrives already shortened (each axis about 0.71), so without this a hero
/// pressed against the lane top while holding up and right would slide at only 71 % of its
/// horizontal speed. The spec asks for normal speed along a limit. The input's length is kept, so
/// a diagonal is still never faster than a straight line.
fn slide_along_limits(position: (f32, f32), input: (f32, f32), limits: Limits) -> (f32, f32) {
    let blocked_x = (input.0 < 0.0 && position.0 <= limits.min_x)
        || (input.0 > 0.0 && position.0 >= limits.max_x);
    let blocked_y = (input.1 < 0.0 && position.1 <= limits.min_y)
        || (input.1 > 0.0 && position.1 >= limits.max_y);
    let length = input.0.hypot(input.1);
    match (blocked_x, blocked_y) {
        (true, false) if input.1 != 0.0 => (0.0, length.copysign(input.1)),
        (false, true) if input.0 != 0.0 => (length.copysign(input.0), 0.0),
        _ => input,
    }
}

#[cfg(test)]
mod tests {
    use super::{Limits, Speeds, step};

    const FAST: f32 = 250.0;
    const SLOW: f32 = 150.0;
    const DIAGONAL: f32 = std::f32::consts::FRAC_1_SQRT_2;
    const ORIGIN: (f32, f32) = (500.0, 550.0);

    fn speeds(horizontal: f32, vertical: f32) -> Speeds {
        Speeds {
            horizontal,
            vertical,
        }
    }

    /// Limits wide enough that one second of movement from `ORIGIN` never reaches them.
    fn open_limits() -> Limits {
        Limits {
            min_x: 0.0,
            max_x: 1000.0,
            min_y: 200.0,
            max_y: 900.0,
        }
    }

    fn distance(from: (f32, f32), to: (f32, f32)) -> f32 {
        (to.0 - from.0).hypot(to.1 - from.1)
    }

    #[test]
    fn zero_input_inside_limits_does_not_move() {
        let moved = step(ORIGIN, (0.0, 0.0), speeds(FAST, SLOW), 1.0, open_limits());
        assert_eq!(moved, ORIGIN);
    }

    #[test]
    fn diagonal_is_not_faster_than_horizontal_when_vertical_is_slower() {
        let limits = open_limits();
        let straight = step(ORIGIN, (1.0, 0.0), speeds(FAST, SLOW), 1.0, limits);
        let diagonal = step(
            ORIGIN,
            (DIAGONAL, -DIAGONAL),
            speeds(FAST, SLOW),
            1.0,
            limits,
        );
        assert!(distance(ORIGIN, diagonal) <= distance(ORIGIN, straight));
    }

    #[test]
    fn diagonal_is_not_faster_than_the_faster_axis_when_vertical_is_faster() {
        let limits = open_limits();
        let straight = step(ORIGIN, (0.0, 1.0), speeds(SLOW, FAST), 1.0, limits);
        let diagonal = step(
            ORIGIN,
            (DIAGONAL, DIAGONAL),
            speeds(SLOW, FAST),
            1.0,
            limits,
        );
        assert!(distance(ORIGIN, diagonal) <= distance(ORIGIN, straight));
    }

    #[test]
    fn vertical_covers_less_ground_than_horizontal() {
        let limits = open_limits();
        let horizontal = step(ORIGIN, (1.0, 0.0), speeds(FAST, SLOW), 1.0, limits);
        let vertical = step(ORIGIN, (0.0, 1.0), speeds(FAST, SLOW), 1.0, limits);
        assert!(distance(ORIGIN, vertical) < distance(ORIGIN, horizontal));
    }

    #[test]
    fn each_limit_stops_movement_exactly_on_it() {
        let limits = Limits {
            min_x: 450.0,
            max_x: 550.0,
            min_y: 500.0,
            max_y: 600.0,
        };
        let s = speeds(FAST, SLOW);
        assert_eq!(step(ORIGIN, (-1.0, 0.0), s, 1.0, limits).0, limits.min_x);
        assert_eq!(step(ORIGIN, (1.0, 0.0), s, 1.0, limits).0, limits.max_x);
        assert_eq!(step(ORIGIN, (0.0, -1.0), s, 1.0, limits).1, limits.min_y);
        assert_eq!(step(ORIGIN, (0.0, 1.0), s, 1.0, limits).1, limits.max_y);
    }

    #[test]
    fn holding_into_a_limit_never_moves_the_position() {
        let limits = open_limits();
        let corner = (limits.max_x, limits.min_y);
        let mut position = corner;
        for _ in 0..100 {
            position = step(
                position,
                (DIAGONAL, -DIAGONAL),
                speeds(FAST, SLOW),
                1.0 / 60.0,
                limits,
            );
            assert_eq!(position, corner);
        }
    }

    #[test]
    fn slides_along_the_lane_top_at_full_horizontal_speed() {
        let limits = open_limits();
        let on_top = (ORIGIN.0, limits.min_y);
        let moved = step(
            on_top,
            (DIAGONAL, -DIAGONAL),
            speeds(FAST, SLOW),
            1.0,
            limits,
        );
        assert_eq!(moved, (ORIGIN.0 + FAST, limits.min_y));
    }

    #[test]
    fn slides_along_a_screen_edge_at_full_vertical_speed() {
        let limits = open_limits();
        let on_right = (limits.max_x, ORIGIN.1);
        let moved = step(
            on_right,
            (DIAGONAL, DIAGONAL),
            speeds(FAST, SLOW),
            1.0,
            limits,
        );
        assert_eq!(moved, (limits.max_x, ORIGIN.1 + SLOW));
    }

    #[test]
    fn pushing_straight_into_a_limit_does_not_slide() {
        let limits = open_limits();
        let on_left = (limits.min_x, ORIGIN.1);
        let moved = step(on_left, (-1.0, 0.0), speeds(FAST, SLOW), 1.0, limits);
        assert_eq!(moved, on_left);
    }

    #[test]
    fn huge_delta_still_ends_inside_the_limits() {
        let limits = open_limits();
        let moved = step(
            ORIGIN,
            (DIAGONAL, DIAGONAL),
            speeds(FAST, SLOW),
            1000.0,
            limits,
        );
        assert!(
            (limits.min_x..=limits.max_x).contains(&moved.0),
            "{moved:?}"
        );
        assert!(
            (limits.min_y..=limits.max_y).contains(&moved.1),
            "{moved:?}"
        );
    }

    #[test]
    fn position_outside_the_limits_moves_to_the_nearest_point_inside() {
        let limits = open_limits();
        let moved = step((-40.0, 100.0), (0.0, 0.0), speeds(FAST, SLOW), 1.0, limits);
        assert_eq!(moved, (limits.min_x, limits.min_y));
    }
}
