// Entity Component System module
// TODO: Move ECS code from lib.rs here

pub mod world;
pub mod component;
pub mod system;
pub mod query;

pub use world::*;
pub use component::*;
pub use system::*;
pub use query::*;
