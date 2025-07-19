use typosaurus::num::consts::*;
use typosaurus::{list, set};

use crate::graph::{NodeOutput, NodeOutputData, NodeOutputKind, Typegraph, Unsigned};

#[cfg(feature = "value")]
use crate::value::{NodeKind, Value};

#[cfg(feature = "value")]
macro_rules! primitive_typegraph_impl {
    ($p:ty => $n:ident,$t:ty) => {
        pub struct $n;
        impl Value<NodeKind> for $n {
            fn value() -> NodeKind {
                NodeKind::Primitive(stringify!($p), &[])
            }
        }
        impl NodeOutputData for $n {
            const ID: u32 = <$t as Unsigned>::U32;
            const KIND: NodeOutputKind = NodeOutputKind::Primitive;
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
macro_rules! primitive_typegraph_impl {
    ($p:ty => $n:ident,$t:ty) => {
        pub struct $n;
        impl NodeOutputData for $n {
            const ID: u32 = <$t as Unsigned>::U32;
            const KIND: NodeOutputKind = NodeOutputKind::Primitive;
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
primitive_typegraph_impl!(() => UnitNode, U1);
primitive_typegraph_impl!(i8 => I8Node, U2);
primitive_typegraph_impl!(i16 => I16Node, U3);
primitive_typegraph_impl!(i32 => I32Node, U4);
primitive_typegraph_impl!(i64 => I64Node, U5);
primitive_typegraph_impl!(i128 => I128Node, U6);
primitive_typegraph_impl!(isize => IsizeNode, U7);
primitive_typegraph_impl!(u8 => U8Node, U8);
primitive_typegraph_impl!(u16 => U16Node, U9);
primitive_typegraph_impl!(u32 => U32Node, U10);
primitive_typegraph_impl!(u64 => U64Node, U11);
primitive_typegraph_impl!(u128 => U128Node, U12);
primitive_typegraph_impl!(usize => UsizeNode, U13);
primitive_typegraph_impl!(f32 => F32Node, U14);
primitive_typegraph_impl!(f64 => F64Node, U15);
primitive_typegraph_impl!(char => CharNode, U17);
primitive_typegraph_impl!(bool => BoolNode, U18);
primitive_typegraph_impl!(str => StrNode, U19);

pub struct StrSliceNode;
#[cfg(feature = "value")]
impl Value<NodeKind> for StrSliceNode {
    fn value() -> NodeKind {
        NodeKind::Primitive("&str", &[])
    }
}
impl NodeOutputData for StrSliceNode {
    const ID: u32 = 20;
    const KIND: NodeOutputKind = NodeOutputKind::Primitive;
    const NAME: &'static str = "&str";
}
impl Typegraph for &str {
    type Id = U20;
    type Node = StrSliceNode;
    type Nodes = set![NodeOutput<U20, StrSliceNode, StrSliceNode>];
    type Edges = set![];
    type Types = list![];
}
