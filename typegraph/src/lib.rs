#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
mod container;
mod graph;
mod op;
mod primitive;
#[cfg(feature = "std")]
mod standard;

#[cfg(feature = "value")]
mod value;

pub use graph::*;
pub use op::Resolve;
pub use typegraph_macros::{typegraph, Typegraph};
pub use typosaurus::bool::{False, True};
pub use typosaurus::cmp::Equality;
pub use typosaurus::collections::array::Arrayify;
pub use typosaurus::collections::graph::{Get, IdList, Incoming, Insert, Outgoing, Topo};
pub use typosaurus::collections::list::{self, List};
pub use typosaurus::collections::maybe::{Just, Nothing};
pub use typosaurus::collections::record::GetEntry;
pub use typosaurus::collections::set;
pub use typosaurus::num::consts::{B0, B1, U0, U1, U2, U3, U4, U5};
pub use typosaurus::num::{UInt, UTerm};
pub use typosaurus::{
    assert_type_eq, connect_graph, elements, graph, insert_nodes, list, maybe_connect_graph,
    maybe_insert_nodes, merge_graphs, merge_lists, merge_records, merge_sets, record, set,
};

pub mod num {
    pub use typosaurus::num::consts::*;
    pub use typosaurus::num::{UInt, UTerm};
}

#[cfg(feature = "graphviz")]
pub use value::graphviz::Graphviz;
#[cfg(feature = "value")]
pub use value::{NodeKind, Value, ValueGraph};
