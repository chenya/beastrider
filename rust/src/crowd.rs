//! How a crowd of enemies takes turns and keeps apart, as pure rules tested without the engine.
//!
//! [`AttackTokens`] is the pool that limits who may close in and attack: an enemy holds its kind's
//! cost while it has a turn, and free tokens go to the waiting enemy nearest its target, unless one
//! has waited past the maximum wait (research R5). [`separate`] pushes enemies that come too close apart, so a crowd never stacks into one
//! shape (research R7). The `WaveDirector` owns both and applies them once per physics frame.

/// The attack-token pool. The tokens held never add up to more than the capacity.
#[derive(Debug, Clone, PartialEq)]
pub struct AttackTokens<T> {
    capacity: u32,
    holders: Vec<(T, u32)>,
}

impl<T: Copy + PartialEq + Ord> AttackTokens<T> {
    /// An empty pool of `capacity` tokens.
    pub fn new(capacity: u32) -> Self {
        Self {
            capacity,
            holders: Vec::new(),
        }
    }

    /// Changes the pool's size. Holders keep their tokens even if they now exceed it; the new size
    /// applies as tokens are freed and handed out again.
    pub fn set_capacity(&mut self, capacity: u32) {
        self.capacity = capacity;
    }

    /// Tokens held right now.
    pub fn in_use(&self) -> u32 {
        self.holders.iter().map(|(_, cost)| cost).sum()
    }

    /// Whether `id` holds a turn.
    pub fn holds(&self, id: T) -> bool {
        self.holders.iter().any(|(holder, _)| *holder == id)
    }

    /// Frees the tokens of every holder not in `still_holding`: those that finished their
    /// attack's recovery, were stunned, died or are gone.
    pub fn reconcile(&mut self, still_holding: &[T]) {
        self.holders.retain(|(id, _)| still_holding.contains(id));
    }

    /// Hands free tokens to `waiting` enemies in turn order (see [`turn_order`]). Granting stops
    /// at the first enemy whose whole cost does not fit, so nobody jumps ahead of a near or
    /// overdue brute. Returns the enemies granted a turn, in order.
    pub fn grant(&mut self, waiting: &[TurnRequest<T>], max_wait: f32) -> Vec<T> {
        let mut granted = Vec::new();
        for request in turn_order(waiting, max_wait) {
            if self.holds(request.id) {
                continue;
            }
            if self.in_use() + request.cost > self.capacity {
                break;
            }
            self.holders.push((request.id, request.cost));
            granted.push(request.id);
        }
        granted
    }
}

/// One waiting enemy asking for a turn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TurnRequest<T> {
    /// Who asks.
    pub id: T,
    /// The tokens its kind costs.
    pub cost: u32,
    /// Its distance to its target, in pixels.
    pub distance: f32,
    /// Seconds it has been waiting.
    pub waited: f32,
}

/// The order waiting enemies get turns: those that have waited at least `max_wait` first, longest
/// waiting first, so no enemy waits forever (planning answer); then the rest, nearest first. Ties
/// go to the lower id.
pub fn turn_order<T: Copy + Ord>(waiting: &[TurnRequest<T>], max_wait: f32) -> Vec<TurnRequest<T>> {
    let mut order = waiting.to_vec();
    order.sort_by(|a, b| {
        let overdue = |r: &TurnRequest<T>| r.waited >= max_wait;
        match (overdue(a), overdue(b)) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (true, true) => b.waited.total_cmp(&a.waited),
            (false, false) => a.distance.total_cmp(&b.distance),
        }
        .then(a.id.cmp(&b.id))
    });
    order
}

/// How readily a body gives way in the separation pass. In a pair, the one that gives way more
/// takes the whole push; equals split it, except two that hold, which stay put.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Give {
    /// Attacking: never moved, so its aim holds.
    Holds,
    /// Approaching with a turn: pushes waiting enemies aside, so nobody blocks its way in.
    Presses,
    /// Waiting, retreating or stunned: gives way.
    Yields,
}

/// One enemy as the separation pass sees it: its feet and how readily it gives way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Body {
    /// Feet x, in pixels.
    pub x: f32,
    /// Feet y (depth), in pixels.
    pub y: f32,
    /// How readily it gives way.
    pub give: Give,
}

/// The offsets that push every pair of bodies closer than `radius` apart, one per body, in order.
///
/// Distance counts a depth gap `depth_scale` times (the hero's horizontal over vertical speed, so a
/// gap counts as much as the time it takes to walk it). Each pair is pushed along the line between
/// them by the overlap: all onto the one that gives way more (see [`Give`]), half each between
/// equals, none between two that hold. Bodies at the same spot split along x, the lower index to the left. A
/// radius of 0 turns the pass off. The caller clamps the moved bodies to the lane and screen.
pub fn separate(bodies: &[Body], radius: f32, depth_scale: f32) -> Vec<(f32, f32)> {
    let mut offsets = vec![(0.0_f32, 0.0_f32); bodies.len()];
    if radius <= 0.0 {
        return offsets;
    }
    for i in 0..bodies.len() {
        for j in i + 1..bodies.len() {
            let (a, b) = (bodies[i], bodies[j]);
            let (share_a, share_b) = match a.give.cmp(&b.give) {
                std::cmp::Ordering::Greater => (1.0, 0.0),
                std::cmp::Ordering::Less => (0.0, 1.0),
                std::cmp::Ordering::Equal => match a.give {
                    Give::Holds => continue,
                    Give::Presses | Give::Yields => (0.5, 0.5),
                },
            };
            let dx = b.x - a.x;
            let dy = (b.y - a.y) * depth_scale;
            let distance = dx.hypot(dy);
            if distance >= radius {
                continue;
            }
            let (ux, uy) = if distance > 0.0 {
                (dx / distance, dy / distance)
            } else {
                (1.0, 0.0)
            };
            let overlap = radius - distance;
            offsets[i].0 -= ux * overlap * share_a;
            offsets[i].1 -= uy * overlap * share_a;
            offsets[j].0 += ux * overlap * share_b;
            offsets[j].1 += uy * overlap * share_b;
        }
    }
    for offset in &mut offsets {
        offset.1 /= depth_scale;
    }
    offsets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat;

    // Values deliberately different from the starting values in tuning.json.
    const RADIUS: f32 = 50.0;
    const SCALE: f32 = 1.5;

    // --- Attack tokens. ---

    /// A long max wait, so these tests see the nearest-first order alone.
    const PATIENT: f32 = 1000.0;

    /// Requests from `(id, cost, distance)`, none of them waiting long.
    fn asks(list: &[(u32, u32, f32)]) -> Vec<TurnRequest<u32>> {
        list.iter()
            .map(|&(id, cost, distance)| TurnRequest {
                id,
                cost,
                distance,
                waited: 0.0,
            })
            .collect()
    }

    #[test]
    fn tokens_never_exceed_capacity() {
        let mut tokens = AttackTokens::new(3);
        // A mixed sequence of grants and releases.
        let waiting: Vec<(u32, u32, f32)> = (0..8)
            .map(|id| (id, 1 + id % 2, id as f32 * 10.0))
            .collect();
        for round in 0..20 {
            tokens.grant(&asks(&waiting), PATIENT);
            assert!(tokens.in_use() <= 3, "round {round}: {}", tokens.in_use());
            let keep: Vec<u32> = (0..8).filter(|id| (id + round) % 3 != 0).collect();
            tokens.reconcile(&keep);
            assert!(tokens.in_use() <= 3, "round {round}: {}", tokens.in_use());
        }
    }

    #[test]
    fn reconcile_frees_the_tokens_of_enemies_no_longer_holding() {
        let mut tokens = AttackTokens::new(2);
        assert_eq!(
            tokens.grant(&asks(&[(1, 1, 5.0), (2, 1, 9.0)]), PATIENT),
            vec![1, 2]
        );
        // 1 finished its recovery, was stunned or died: it is not in the list any more.
        tokens.reconcile(&[2]);
        assert_eq!(tokens.in_use(), 1);
        assert!(!tokens.holds(1));
        assert!(tokens.holds(2));
    }

    #[test]
    fn the_nearest_waiting_enemy_gets_the_free_token() {
        let mut tokens = AttackTokens::new(1);
        assert_eq!(
            tokens.grant(
                &asks(&[(7, 1, 300.0), (3, 1, 120.0), (5, 1, 200.0)]),
                PATIENT
            ),
            vec![3]
        );
        tokens.reconcile(&[]);
        // Ties go to the lower id.
        assert_eq!(
            tokens.grant(&asks(&[(9, 1, 100.0), (4, 1, 100.0)]), PATIENT),
            vec![4]
        );
    }

    #[test]
    fn a_near_brute_that_does_not_fit_blocks_farther_grunts() {
        let mut tokens = AttackTokens::new(2);
        assert_eq!(tokens.grant(&asks(&[(1, 1, 50.0)]), PATIENT), vec![1]);
        // One token free; the nearest waiting enemy is a brute costing 2.
        assert!(
            tokens
                .grant(&asks(&[(2, 2, 80.0), (3, 1, 150.0)]), PATIENT)
                .is_empty()
        );
        assert_eq!(tokens.in_use(), 1);
        // Once the grunt's turn ends, the brute gets both.
        tokens.reconcile(&[]);
        assert_eq!(
            tokens.grant(&asks(&[(2, 2, 80.0), (3, 1, 150.0)]), PATIENT),
            vec![2]
        );
        assert_eq!(tokens.in_use(), 2);
    }

    #[test]
    fn a_smaller_capacity_keeps_holders_and_blocks_new_grants() {
        let mut tokens = AttackTokens::new(3);
        assert_eq!(
            tokens
                .grant(&asks(&[(1, 1, 1.0), (2, 1, 2.0), (3, 1, 3.0)]), PATIENT)
                .len(),
            3
        );
        tokens.set_capacity(1);
        assert_eq!(tokens.in_use(), 3, "holders keep their tokens");
        tokens.reconcile(&[2, 3]);
        assert!(
            tokens.grant(&asks(&[(4, 1, 1.0)]), PATIENT).is_empty(),
            "still 2 held, pool 1"
        );
        tokens.reconcile(&[]);
        assert_eq!(
            tokens.grant(&asks(&[(4, 1, 1.0), (5, 1, 2.0)]), PATIENT),
            vec![4]
        );
    }

    #[test]
    fn an_enemy_already_holding_is_not_granted_twice() {
        let mut tokens = AttackTokens::new(3);
        tokens.grant(&asks(&[(1, 1, 1.0)]), PATIENT);
        assert!(tokens.grant(&asks(&[(1, 1, 1.0)]), PATIENT).is_empty());
        assert_eq!(tokens.in_use(), 1);
    }

    #[test]
    fn an_enemy_past_the_max_wait_goes_before_nearer_ones() {
        let requests = [
            TurnRequest {
                id: 1,
                cost: 1,
                distance: 100.0,
                waited: 1.0,
            },
            TurnRequest {
                id: 2,
                cost: 2,
                distance: 300.0,
                waited: 5.0,
            },
            TurnRequest {
                id: 3,
                cost: 1,
                distance: 150.0,
                waited: 6.0,
            },
        ];
        let order: Vec<u32> = turn_order(&requests, 4.0).iter().map(|r| r.id).collect();
        assert_eq!(order, vec![3, 2, 1], "overdue longest first, then nearest");
        // An overdue brute that does not fit blocks the nearer grunt.
        let mut tokens = AttackTokens::new(2);
        tokens.grant(&asks(&[(9, 1, 10.0)]), PATIENT);
        assert!(tokens.grant(&requests[..2], 4.0).is_empty());
        tokens.reconcile(&[]);
        assert_eq!(tokens.grant(&requests[..2], 4.0), vec![2]);
    }

    #[test]
    fn a_zero_max_wait_serves_the_longest_waiting() {
        let mut requests = asks(&[(1, 1, 50.0), (2, 1, 400.0)]);
        requests[1].waited = 2.0;
        requests[0].waited = 0.5;
        let order: Vec<u32> = turn_order(&requests, 0.0).iter().map(|r| r.id).collect();
        assert_eq!(order, vec![2, 1]);
    }

    // --- Separation. ---

    fn body(x: f32, y: f32) -> Body {
        Body {
            x,
            y,
            give: Give::Yields,
        }
    }

    fn apply(bodies: &[Body], offsets: &[(f32, f32)]) -> Vec<(f32, f32)> {
        bodies
            .iter()
            .zip(offsets)
            .map(|(b, o)| (b.x + o.0, b.y + o.1))
            .collect()
    }

    fn scaled_distance(a: (f32, f32), b: (f32, f32)) -> f32 {
        (b.0 - a.0).hypot((b.1 - a.1) * SCALE)
    }

    #[test]
    fn overlapping_bodies_end_a_radius_apart() {
        let bodies = [body(100.0, 500.0), body(120.0, 505.0)];
        let moved = apply(&bodies, &separate(&bodies, RADIUS, SCALE));
        assert!((scaled_distance(moved[0], moved[1]) - RADIUS).abs() < 1e-3);
        // Each moved half the way, in opposite directions.
        assert!(moved[0].0 < 100.0 && moved[1].0 > 120.0);
    }

    #[test]
    fn bodies_far_enough_apart_are_left_alone() {
        let bodies = [body(100.0, 500.0), body(200.0, 500.0)];
        assert_eq!(
            separate(&bodies, RADIUS, SCALE),
            vec![(0.0, 0.0), (0.0, 0.0)]
        );
    }

    #[test]
    fn a_fixed_body_stays_and_its_neighbour_takes_the_whole_push() {
        let bodies = [
            Body {
                x: 100.0,
                y: 500.0,
                give: Give::Holds,
            },
            body(110.0, 500.0),
        ];
        let offsets = separate(&bodies, RADIUS, SCALE);
        assert_eq!(offsets[0], (0.0, 0.0));
        assert!((offsets[1].0 - 40.0).abs() < 1e-4, "{offsets:?}");
    }

    #[test]
    fn an_approaching_enemy_pushes_a_waiting_one_aside() {
        let bodies = [
            Body {
                x: 820.0,
                y: 536.0,
                give: Give::Presses,
            },
            body(820.0, 560.0),
        ];
        let offsets = separate(&bodies, RADIUS, SCALE);
        assert_eq!(offsets[0], (0.0, 0.0), "the one with a turn keeps its way");
        assert!(offsets[1].1 > 0.0, "{offsets:?}");
        // Two approaching enemies still split the push.
        let pressing = |x| Body {
            x,
            y: 560.0,
            give: Give::Presses,
        };
        let offsets = separate(&[pressing(500.0), pressing(510.0)], RADIUS, SCALE);
        assert_eq!(offsets[0], (-20.0, 0.0));
        assert_eq!(offsets[1], (20.0, 0.0));
    }

    #[test]
    fn two_fixed_bodies_are_left_alone() {
        let fixed = |x| Body {
            x,
            y: 500.0,
            give: Give::Holds,
        };
        assert_eq!(
            separate(&[fixed(100.0), fixed(101.0)], RADIUS, SCALE),
            vec![(0.0, 0.0), (0.0, 0.0)]
        );
    }

    #[test]
    fn bodies_on_the_same_spot_split_along_x_lower_index_left() {
        let bodies = [body(300.0, 550.0), body(300.0, 550.0)];
        let offsets = separate(&bodies, RADIUS, SCALE);
        assert_eq!(offsets[0], (-25.0, 0.0));
        assert_eq!(offsets[1], (25.0, 0.0));
    }

    #[test]
    fn a_depth_gap_counts_by_the_depth_scale() {
        // 30 px apart in depth: 45 scaled, inside the radius; 40 px is 60 scaled, outside it.
        let near = [body(100.0, 500.0), body(100.0, 530.0)];
        let offsets = separate(&near, RADIUS, SCALE);
        assert!(offsets[0].1 < 0.0 && offsets[1].1 > 0.0, "{offsets:?}");
        let moved = apply(&near, &offsets);
        assert!((scaled_distance(moved[0], moved[1]) - RADIUS).abs() < 1e-3);
        let far = [body(100.0, 500.0), body(100.0, 540.0)];
        assert_eq!(separate(&far, RADIUS, SCALE), vec![(0.0, 0.0), (0.0, 0.0)]);
    }

    #[test]
    fn a_zero_radius_turns_the_pass_off() {
        let bodies = [body(100.0, 500.0), body(100.0, 500.0)];
        assert_eq!(separate(&bodies, 0.0, SCALE), vec![(0.0, 0.0), (0.0, 0.0)]);
    }

    #[test]
    fn pushed_bodies_clamped_to_the_lane_never_leave_it() {
        let (top, bottom) = (420.0, 690.0);
        let bodies = [body(400.0, top + 2.0), body(400.0, top + 4.0)];
        let offsets = separate(&bodies, RADIUS, SCALE);
        for (b, o) in bodies.iter().zip(&offsets) {
            let (_, y) = combat::clamp_to_area((b.x + o.0, b.y + o.1), 26.0, 1280.0, top, bottom);
            assert!((top..=bottom).contains(&y), "{y}");
        }
    }
}
