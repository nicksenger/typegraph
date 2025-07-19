use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Condvar;
use std::task::Waker;
use std::thread::ThreadId;
use std::time::Duration;

use typosaurus::num::consts::*;
use typosaurus::{list, set};

use crate::graph::{NodeOutput, NodeOutputData, NodeOutputKind, Typegraph, Unsigned};

#[cfg(feature = "value")]
use crate::value::{NodeKind, Value};

#[cfg(feature = "value")]
macro_rules! standard_typegraph_impl {
    ($p:ty => $n:ident,$t:ty) => {
        pub struct $n;
        impl Value<NodeKind> for $n {
            fn value() -> NodeKind {
                NodeKind::Struct(stringify!($p), &[], &[])
            }
        }
        impl NodeOutputData for $n {
            const ID: u32 = <$t as Unsigned>::U32;
            const KIND: NodeOutputKind = NodeOutputKind::Struct;
            const NAME: &'static str = stringify!($p);
        }
        impl Typegraph for $p {
            type Id = $t;
            type Node = $n;
            type Nodes = set![NodeOutput<$t, $n, $n>];
            type Edges = set![];
            type Types = list![];
        }
    };
}
#[cfg(not(feature = "value"))]
macro_rules! standard_typegraph_impl {
    ($p:ty => $n:ident,$t:ty) => {
        pub struct $n;
        impl NodeOutputData for $n {
            const ID: u32 = <$t as Unsigned>::U32;
            const KIND: NodeOutputKind = NodeOutputKind::Struct;
            const NAME: &'static str = stringify!($p);
        }
        impl Typegraph for $p {
            type Id = $t;
            type Node = $n;
            type Nodes = set![NodeOutput<$t, $n, $n>];
            type Edges = set![];
            type Types = list![];
        }
    };
}

standard_typegraph_impl!(Duration => DurationNode, U300);
standard_typegraph_impl!(AtomicBool => AtomicBoolNode, U301);
standard_typegraph_impl!(AtomicUsize => AtomicUsizeNode, U302);
standard_typegraph_impl!(Condvar => CondvarNode, U303);
standard_typegraph_impl!(ThreadId => ThreadIdNode, U304);
standard_typegraph_impl!(Waker => WakerNode, U305);
standard_typegraph_impl!(String => StringNode, U306);
