use core::ops::Add;
use std::boxed::Box;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::format;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, Weak};
use std::thread::JoinHandle;
use std::vec::Vec;

use typosaurus::bool::{False, Falsy, Or};
use typosaurus::cmp::Equality;
use typosaurus::num::consts::*;
use typosaurus::{list, set};

use crate::{Edge, NodeOutput, NodeOutputData, NodeOutputKind, Typegraph};
#[cfg(feature = "value")]
use crate::{NodeKind, Value};

#[cfg(feature = "value")]
macro_rules! impl_unary_container {
    ($t:ident => $node:ident,$id:ty,$vid:literal,$label:literal) => {
        pub struct $node<T>(std::marker::PhantomData<T>);
        impl<T> $crate::Value<$crate::NodeKind> for $node<T>
        where
            T: $crate::Value<$crate::NodeKind>,
        {
            fn value() -> $crate::NodeKind {
                $crate::NodeKind::UnaryContainer(
                    format!($label, <T as $crate::Value<$crate::NodeKind>>::value().label()).leak(),
                    <T as $crate::Value<$crate::NodeKind>>::value().cluster(),
                )
            }
        }
        impl<T> $crate::NodeOutputData for $node<T>
        where
            T: $crate::NodeOutputData,
        {
            const ID: u32 = <T as $crate::NodeOutputData>::ID + $vid;
            const KIND: $crate::NodeOutputKind = $crate::NodeOutputKind::UnaryContainer;
            const NAME: &'static str = stringify!($n<T>);
        }
        impl<T> $crate::Typegraph for $t<T>
        where
            T: $crate::Typegraph,
            <T as $crate::Typegraph>::Id: core::ops::Add<$id>,
        {
            type Id = <<T as $crate::Typegraph>::Id as core::ops::Add<$id>>::Output;
            type Node = $node<<T as $crate::Typegraph>::Node>;
            type Nodes = $crate::set![$crate::NodeOutput<
                                                        <<T as $crate::Typegraph>::Id as core::ops::Add<$id>>::Output,
                                                        $node<<T as $crate::Typegraph>::Node>,
                                                        $node<<T as $crate::Typegraph>::Node>,
                                                    >];
            type Edges =
                $crate::set![$crate::Edge<<<T as $crate::Typegraph>::Id as core::ops::Add<$id>>::Output, <T as $crate::Typegraph>::Id>];
            type Types = $crate::list![T];
        }
    };
}
#[cfg(not(feature = "value"))]
macro_rules! impl_unary_container {
    ($t:ident => $node:ident,$id:ty,$vid:literal,$label:literal) => {
        pub struct $node<T>(std::marker::PhantomData<T>);
        impl<T> $crate::NodeOutputData for $node<T>
        where
            T: $crate::NodeOutputData,
        {
            const ID: u32 = <T as $crate::NodeOutputData>::ID + $vid;
            const KIND: $crate::NodeOutputKind = $crate::NodeOutputKind::UnaryContainer;
            const NAME: &'static str = stringify!($n<T>);
        }
        impl<T> $crate::Typegraph for $t<T>
        where
            T: $crate::Typegraph,
            <T as $crate::Typegraph>::Id: core::ops::Add<$id>,
        {
            type Id = <<T as $crate::Typegraph>::Id as core::ops::Add<$id>>::Output;
            type Node = $node<<T as $crate::Typegraph>::Node>;
            type Nodes = $crate::set![$crate::NodeOutput<
                                                        <<T as $crate::Typegraph>::Id as core::ops::Add<$id>>::Output,
                                                        $node<<T as $crate::Typegraph>::Node>,
                                                        $node<<T as $crate::Typegraph>::Node>,
                                                    >];
            type Edges =
                $crate::set![$crate::Edge<<<T as $crate::Typegraph>::Id as core::ops::Add<$id>>::Output, <T as $crate::Typegraph>::Id>];
            type Types = $crate::list![T];
        }
    };
}

impl_unary_container!(Vec => VecNode, U100, 100, "Vec<{}>");
impl_unary_container!(Mutex => MutexNode, U101, 101, "Mutex<{}>");
impl_unary_container!(Arc => ArcNode, U102, 102, "Arc<{}>");
impl_unary_container!(Option => OptionNode, U103, 103, "Option<{}>");
impl_unary_container!(JoinHandle => JoinHandleNode, U104, 104, "JoinHandle<{}>");
impl_unary_container!(VecDeque => VecDequeNode, U105, 105, "VecDeque<{}>");
impl_unary_container!(Weak => WeakNode, U106, 106, "Weak<{}>");
impl_unary_container!(RefCell => RefCellNode, U107, 107, "RefCell<{}>");
impl_unary_container!(Box => BoxNode, U108, 108, "Box<{}>");
impl_unary_container!(PhantomData => PhantomDataNode, U109, 109, "PhantomData<{}>");

#[cfg(feature = "value")]
macro_rules! impl_binary_container {
    ($t:ident => $node:ident,$id:ty,$vid:literal,$label:literal) => {
        pub struct $node<T, U>(PhantomData<T>, PhantomData<U>);
        impl<T, U> Value<NodeKind> for $node<T, U>
        where
            T: Value<NodeKind>,
            U: Value<NodeKind>,
        {
            fn value() -> NodeKind {
                let t_cluster = <T as Value<NodeKind>>::value().cluster();
                let u_cluster = <U as Value<NodeKind>>::value().cluster();
                NodeKind::BinaryContainer(
                    format!(
                        $label,
                        <T as Value<NodeKind>>::value().label(),
                        <U as Value<NodeKind>>::value().label()
                    )
                    .leak(),
                    if t_cluster.is_empty() {
                        u_cluster
                    } else {
                        t_cluster
                    },
                )
            }
        }
        impl<T, U> NodeOutputData for $node<T, U>
        where
            T: NodeOutputData,
            U: NodeOutputData,
        {
            const ID: u32 = <T as NodeOutputData>::ID + <U as NodeOutputData>::ID + $vid;
            const KIND: NodeOutputKind = NodeOutputKind::BinaryContainer;
            const NAME: &'static str = stringify!($n<T>);
        }
        impl<T, U> Typegraph for $t<T, U>
        where
            T: Typegraph,
            U: Typegraph,
            <T as Typegraph>::Id: Add<<U as Typegraph>::Id>,
            <<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output: Add<$id>,
            Edge<
                <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output,
                <U as Typegraph>::Id
            >: Equality<Edge<<<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output, <T as Typegraph>::Id>>,
            (<Edge<
                <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output,
                <U as Typegraph>::Id
            > as Equality<Edge<<<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output, <T as Typegraph>::Id>>>::Out, False): Or,
            <(<Edge<
                <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output,
                <U as Typegraph>::Id
            > as Equality<Edge<<<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output, <T as Typegraph>::Id>>>::Out, False) as Or>::Out: Falsy
        {
            type Id = <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output;
            type Node = $node<<T as Typegraph>::Node, <U as Typegraph>::Node>;
            type Nodes = set![
                NodeOutput<Self::Id, $node<<T as Typegraph>::Node, <U as Typegraph>::Node>, $node<<T as Typegraph>::Node, <U as Typegraph>::Node>>,
            ];
            type Edges = set![
                Edge<Self::Id, <T as Typegraph>::Id>,
                Edge<Self::Id, <U as Typegraph>::Id>
            ];
            type Types = list![T, U];
        }
    };
}
#[cfg(not(feature = "value"))]
macro_rules! impl_binary_container {
    ($t:ident => $node:ident,$id:ty,$vid:literal,$label:literal) => {
        pub struct $node<T, U>(PhantomData<T>, PhantomData<U>);
        impl<T, U> NodeOutputData for $node<T, U>
        where
            T: NodeOutputData,
            U: NodeOutputData,
        {
            const ID: u32 = <T as NodeOutputData>::ID + <U as NodeOutputData>::ID + $vid;
            const KIND: NodeOutputKind = NodeOutputKind::BinaryContainer;
            const NAME: &'static str = stringify!($n<T>);
        }
        impl<T, U> Typegraph for $t<T, U>
        where
            T: Typegraph,
            U: Typegraph,
            <T as Typegraph>::Id: Add<<U as Typegraph>::Id>,
            <<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output: Add<$id>,
            Edge<
                <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output,
                <U as Typegraph>::Id
            >: Equality<Edge<<<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output, <T as Typegraph>::Id>>,
            (<Edge<
                <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output,
                <U as Typegraph>::Id
            > as Equality<Edge<<<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output, <T as Typegraph>::Id>>>::Out, False): Or,
            <(<Edge<
                <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output,
                <U as Typegraph>::Id
            > as Equality<Edge<<<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output, <T as Typegraph>::Id>>>::Out, False) as Or>::Out: Falsy
        {
            type Id = <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<$id>>::Output;
            type Node = $node<<T as Typegraph>::Node, <U as Typegraph>::Node>;
            type Nodes = set![
                NodeOutput<Self::Id, $node<<T as Typegraph>::Node, <U as Typegraph>::Node>, $node<<T as Typegraph>::Node, <U as Typegraph>::Node>>,
            ];
            type Edges = set![
                Edge<Self::Id, <T as Typegraph>::Id>,
                Edge<Self::Id, <U as Typegraph>::Id>
            ];
            type Types = list![T, U];
        }
    };
}

impl_binary_container!(Result => ResultNode, U200, 200, "Result<{}, {}>");
impl_binary_container!(HashMap => HashMapNode, U201, 201, "HashMap<{}, {}>");

#[cfg(feature = "value")]
impl<T, U> Value<NodeKind> for (T, U)
where
    T: Value<NodeKind>,
    U: Value<NodeKind>,
{
    fn value() -> NodeKind {
        let t_cluster = <T as Value<NodeKind>>::value().cluster();
        let u_cluster = <U as Value<NodeKind>>::value().cluster();
        NodeKind::BinaryContainer(
            format!(
                "({}, {})",
                <T as Value<NodeKind>>::value().label(),
                <U as Value<NodeKind>>::value().label()
            )
            .leak(),
            if t_cluster.is_empty() {
                u_cluster
            } else {
                t_cluster
            },
        )
    }
}
impl<T, U> NodeOutputData for (T, U)
where
    T: NodeOutputData,
    U: NodeOutputData,
{
    const ID: u32 = <T as NodeOutputData>::ID + <U as NodeOutputData>::ID + 202;
    const KIND: NodeOutputKind = NodeOutputKind::BinaryContainer;
    const NAME: &'static str = stringify!($n<T>);
}
impl<T, U> Typegraph for (T, U)
where
    T: Typegraph,
    U: Typegraph,
    <T as Typegraph>::Id: Add<<U as Typegraph>::Id>,
    <<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output: Add<U202>,
    Edge<
        <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<U202>>::Output,
        <U as Typegraph>::Id,
    >: Equality<
        Edge<
            <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<U202>>::Output,
            <T as Typegraph>::Id,
        >,
    >,
    (
        <Edge<
            <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<U202>>::Output,
            <U as Typegraph>::Id,
        > as Equality<
            Edge<
                <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<U202>>::Output,
                <T as Typegraph>::Id,
            >,
        >>::Out,
        False,
    ): Or,
    <(
        <Edge<
            <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<U202>>::Output,
            <U as Typegraph>::Id,
        > as Equality<
            Edge<
                <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<U202>>::Output,
                <T as Typegraph>::Id,
            >,
        >>::Out,
        False,
    ) as Or>::Out: Falsy,
{
    type Id = <<<T as Typegraph>::Id as Add<<U as Typegraph>::Id>>::Output as Add<U202>>::Output;
    type Node = (<T as Typegraph>::Node, <U as Typegraph>::Node);
    type Nodes = set![
        NodeOutput <Self::Id,
            (<T as Typegraph>::Node, <U as Typegraph>::Node),
            (<T as Typegraph>::Node, <U as Typegraph>::Node)
        >
    ];
    type Edges = set![
        Edge<Self::Id, <T as Typegraph>::Id>,
        Edge<Self::Id, <U as Typegraph>::Id>
    ];
    type Types = list![T, U];
}
