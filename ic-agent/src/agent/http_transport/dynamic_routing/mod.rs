//! Dynamic routing implementation.
pub mod dynamic_route_provider;
/// Health check implementation.
pub mod health_check;
/// Messages used in dynamic routing.
pub(super) mod messages;
/// Node implementation.
pub mod node;
/// Nodes fetch implementation.
pub mod nodes_fetch;
/// Routing snapshot implementation.
pub mod snapshot;
/// Testing and benchmarking helpers.
#[cfg(any(test, feature = "bench"))]
pub mod test_utils;
/// Type aliases used in dynamic routing.
pub(super) mod type_aliases;
