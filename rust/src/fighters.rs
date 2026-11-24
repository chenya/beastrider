//! Reaching any fighter in the scene, whatever its class, through one enum.
//!
//! An attack must find and hit heroes, enemies and training dummies, which are different Rust
//! classes. Every use is an exhaustive `match` on [`FighterRef`], so a new fighter class is a
//! compile error until each place handles it (constitution, "types over strings").

use godot::classes::{Node, SceneTree};
use godot::prelude::*;

use crate::enemy::Enemy;
use crate::fighter::{Hit, Target, Team};
use crate::hero::Hero;
use crate::training_dummy::TrainingDummy;

/// A handle to one fighter in the scene.
pub enum FighterRef {
    /// A player's hero.
    Hero(Gd<Hero>),
    /// An enemy of any kind.
    Enemy(Gd<Enemy>),
    /// A training dummy.
    Dummy(Gd<TrainingDummy>),
}

impl FighterRef {
    /// The fighter behind `node`, or `None` if it is not a fighter class.
    pub fn from_node(node: Gd<Node>) -> Option<Self> {
        let node = match node.try_cast::<Hero>() {
            Ok(hero) => return Some(Self::Hero(hero)),
            Err(node) => node,
        };
        let node = match node.try_cast::<Enemy>() {
            Ok(enemy) => return Some(Self::Enemy(enemy)),
            Err(node) => node,
        };
        node.try_cast::<TrainingDummy>().ok().map(Self::Dummy)
    }

    /// Every fighter in `team`'s group.
    pub fn in_team(tree: &Gd<SceneTree>, team: Team) -> Vec<Self> {
        tree.clone()
            .get_nodes_in_group(team.group())
            .iter_shared()
            .filter_map(Self::from_node)
            .collect()
    }

    /// Who it is.
    pub fn id(&self) -> InstanceId {
        match self {
            Self::Hero(hero) => hero.instance_id(),
            Self::Enemy(enemy) => enemy.instance_id(),
            Self::Dummy(dummy) => dummy.instance_id(),
        }
    }

    /// How the hit rule sees it, or `None` while it has no tuning. Never call it on the fighter
    /// that is running the attack: that one is already bound.
    pub fn target(&self) -> Option<Target<InstanceId>> {
        match self {
            Self::Hero(hero) => hero.bind().target(),
            Self::Enemy(enemy) => enemy.bind().target(),
            Self::Dummy(dummy) => dummy.bind().target(),
        }
    }

    /// Lands `hit` on it.
    pub fn take_hit(&mut self, hit: &Hit<InstanceId>) {
        match self {
            Self::Hero(hero) => hero.bind_mut().take_hit(hit),
            Self::Enemy(enemy) => enemy.bind_mut().take_hit(hit),
            Self::Dummy(dummy) => dummy.bind_mut().take_hit(hit),
        }
    }
}
