#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::let_unit_value)]

//! # Aery - with public everything for use with barebones `bevy_ecs`
//! A plugin that adds a subset of Entity Relationship features to Bevy using Non-fragmenting
//! ZST relations.
//!
//! ### Currently supported:
//! - ZST relations
//! - Fragmenting on (relation) type
//! - Cleanup policies
//! - Declarative APIs for:
//!   - Joining
//!   - Traversing
//!   - Spawning
//!
//! # API tour:
//! Non exhaustive. Covers most common parts.
//! ```
//! use bevy_ecs::prelude::*;
//! use aery::prelude::*;
//!
//! fn main() {
//!     let mut world = World::default();
//!     init_aery(&mut world);
//! }
//!
//! #[derive(Component)]
//! struct Foo;
//!
//! #[derive(Component)]
//! struct Bar;
//!
//! #[derive(Relation)]
//! #[cleanup(policy = "Recursive")]
//! struct ChildOf;
//!
//! #[derive(Relation)]
//! #[multi]
//! struct Bag;
//!
//! // Spawning entities with relations
//! fn setup(mut commands: Commands) {
//!     // A hierarchy of Foos with (chocolate? OwO) Bars in their Bags
//!     commands.add(|wrld: &mut World| {
//!         wrld.spawn(Foo)
//!             .scope::<ChildOf>(|_, mut child| {
//!                 child.insert(Foo);
//!                 child.scope_target::<Bag>(|_, mut bag| { bag.insert(Bar); });
//!             })
//!             .scope::<ChildOf>(|_, mut child| {
//!                 child.insert(Foo);
//!                 child.scope_target::<Bag>(|_, mut bag| { bag.insert(Bar); });
//!             });
//!     })
//! }
//!
//! // Listening for relation events
//! fn alert(mut events: EventReader<TargetEvent>) {
//!     for event in events.iter() {
//!         if event.matches(Wc, TargetOp::Set, ChildOf, Wc) {
//!             println!("{:?} was added as a child of {:?}", event.host, event.target);
//!         }
//!     }
//! }
//!
//! // Relation Queries
//! fn sys(
//!     foos: Query<(&Foo, Relations<(Bag, ChildOf)>)>,
//!     roots: Query<Entity, Root<ChildOf>>,
//!     bars: Query<&Bar>,
//! ) {
//!     foos.ops()
//!         .join::<Bag>(&bars)
//!         .traverse::<ChildOf>(roots.iter())
//!         .for_each(|foo_parent, foo, bar| {
//!             // ..
//!         })
//! }
//! ```

pub mod commands;
pub mod events;
pub mod operations;
pub mod relation;
pub mod scope;
pub mod tuple_traits;

use commands::RefragmentHooks;
use events::{CleanupEvent, TargetEvent};

use bevy_ecs::{entity::Entity, event::Events, world::World};

/// A type to enable wildcard APIs
pub enum Var<T> {
    /// Sepcific value.
    Val(T),
    /// Wildcard. Will match anything.
    Wc,
}

impl<T: PartialEq> PartialEq<T> for Var<T> {
    fn eq(&self, other: &T) -> bool {
        match self {
            Self::Val(v) if v == other => true,
            Self::Wc => true,
            _ => false,
        }
    }
}

impl<T> From<Option<T>> for Var<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Val(value),
            None => Self::Wc,
        }
    }
}

impl From<Entity> for Var<Entity> {
    fn from(value: Entity) -> Self {
        Self::Val(value)
    }
}

pub fn init_aery(world: &mut World) {
    world.insert_resource::<RefragmentHooks>(Default::default());
    world.insert_resource(Events::<TargetEvent>::default());
    world.insert_resource(Events::<CleanupEvent>::default());
}

// pub struct Aery;

// impl Plugin for Aery {
//     fn build(&self, app: &mut App) {
//         app.init_resource::<RefragmentHooks>()
//             .add_event::<TargetEvent>()
//             .add_event::<CleanupEvent>();
//     }
// }

pub mod prelude {
    pub use super::Var::{self, Wc};
    pub use crate::{
        commands::{RelationCommands, Set, Unset},
        events::{TargetEvent, TargetOp},
        operations::{
            AeryQueryExt, ControlFlow, ForEachPermutations, ForEachPermutations3Arity, Join,
            Relations, Traverse,
        },
        relation::{CheckRelations, CleanupPolicy, Participates, Relation, Root, ZstOrPanic},
        scope::Scope,
        tuple_traits::{Joinable, RelationSet},
        init_aery,
    };
    pub use aery_macros::*;
}
