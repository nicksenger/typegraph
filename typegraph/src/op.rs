use typosaurus::collections::graph::{self, ContainsId, Graph};
use typosaurus::collections::list::{self, List};
use typosaurus::collections::maybe::{IfNot, Just, Nothing};
use typosaurus::collections::record::GetEntryMaybe;
use typosaurus::collections::set::IntoList;

use crate::graph::{Index, IntoEdge, IntoNode, NodeOutput, Typegraph};

pub type Resolve<T> = <(graph::Empty, Just<List<(T, List<()>)>>) as TypeResolvable>::Out;

pub trait FieldResolver {
    type Out;
}
impl<T> FieldResolver for (Nothing, T) {
    type Out = Nothing;
}
impl<I> FieldResolver for (Just<I>, list::Empty)
where
    I: Index,
{
    type Out = Just<I>;
}
impl<I, T, U> FieldResolver for (Just<I>, List<(T, U)>)
where
    I: Index,
    (<I as Index>::Fields, T): GetEntryMaybe,
    (<(<I as Index>::Fields, T) as GetEntryMaybe>::Out, U): FieldResolver,
{
    type Out = <(<(<I as Index>::Fields, T) as GetEntryMaybe>::Out, U) as FieldResolver>::Out;
}

pub trait MaybeInsertNodes {
    type Out;
}
impl<N, I, O> MaybeInsertNodes for (Graph<N, I, O>, list::Empty) {
    type Out = Graph<N, I, O>;
}
impl<N, I, O> MaybeInsertNodes for (Graph<N, I, O>, Nothing) {
    type Out = Graph<N, I, O>;
}
impl<N, I, O> MaybeInsertNodes for (Graph<N, I, O>, Just<list::Empty>) {
    type Out = Graph<N, I, O>;
}
impl<T, U, N, I, O> MaybeInsertNodes for (Graph<N, I, O>, Just<List<(T, U)>>)
where
    T: IntoNode,
    (Graph<N, I, O>, <T as IntoNode>::Out): graph::InsertNode,
    (
        <(Graph<N, I, O>, <T as IntoNode>::Out) as graph::InsertNode>::Out,
        U,
    ): MaybeInsertNodes,
{
    type Out = <(
        <(Graph<N, I, O>, <T as IntoNode>::Out) as graph::InsertNode>::Out,
        U,
    ) as MaybeInsertNodes>::Out;
}
impl<T, U, N, I, O> MaybeInsertNodes for (Graph<N, I, O>, List<(T, U)>)
where
    T: IntoNode,
    (Graph<N, I, O>, <T as IntoNode>::Out): graph::InsertNode,
    (
        <(Graph<N, I, O>, <T as IntoNode>::Out) as graph::InsertNode>::Out,
        U,
    ): MaybeInsertNodes,
{
    type Out = <(
        <(Graph<N, I, O>, <T as IntoNode>::Out) as graph::InsertNode>::Out,
        U,
    ) as MaybeInsertNodes>::Out;
}

pub trait MaybeConnectNodes {
    type Out;
}
impl<N, I, O> MaybeConnectNodes for (Graph<N, I, O>, list::Empty) {
    type Out = Graph<N, I, O>;
}
impl<N, I, O> MaybeConnectNodes for (Graph<N, I, O>, Nothing) {
    type Out = Graph<N, I, O>;
}
impl<N, I, O> MaybeConnectNodes for (Graph<N, I, O>, Just<List<()>>) {
    type Out = Graph<N, I, O>;
}
impl<T, U, N, I, O> MaybeConnectNodes for (Graph<N, I, O>, Just<List<(T, U)>>)
where
    T: IntoEdge,
    (Graph<N, I, O>, <T as IntoEdge>::Out): graph::ConnectNodes,
    (
        <(Graph<N, I, O>, <T as IntoEdge>::Out) as graph::ConnectNodes>::Out,
        U,
    ): MaybeConnectNodes,
{
    type Out = <(
        <(Graph<N, I, O>, <T as IntoEdge>::Out) as graph::ConnectNodes>::Out,
        U,
    ) as MaybeConnectNodes>::Out;
}
impl<T, U, N, I, O> MaybeConnectNodes for (Graph<N, I, O>, List<(T, U)>)
where
    T: IntoEdge,
    (Graph<N, I, O>, <T as IntoEdge>::Out): graph::ConnectNodes,
    (
        <(Graph<N, I, O>, <T as IntoEdge>::Out) as graph::ConnectNodes>::Out,
        U,
    ): MaybeConnectNodes,
{
    type Out = <(
        <(Graph<N, I, O>, <T as IntoEdge>::Out) as graph::ConnectNodes>::Out,
        U,
    ) as MaybeConnectNodes>::Out;
}

pub trait TypeResolvable {
    type Out;
}
impl<N, I, O> TypeResolvable for (Graph<N, I, O>, Nothing) {
    type Out = Graph<N, I, O>;
}
impl<N, I, O> TypeResolvable for (Graph<N, I, O>, Just<List<()>>) {
    type Out = Graph<N, I, O>;
}
impl<T, U, N, I, O> TypeResolvable for (Graph<N, I, O>, Just<List<(T, U)>>)
where
    T: Typegraph,
    <T as Typegraph>::Nodes: IntoList,
    <T as Typegraph>::Edges: IntoList,
    NodeOutput<<T as Typegraph>::Id, <T as Typegraph>::Node, <T as Typegraph>::Node>: IntoNode,
    (Graph<N, I, O>, <T as Typegraph>::Id): ContainsId,
    (
        Just<<<T as Typegraph>::Nodes as IntoList>::Out>,
        <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
    ): IfNot,
    (
        Just<<T as Typegraph>::Types>,
        <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
    ): IfNot,
    (
        Just<<<T as Typegraph>::Edges as IntoList>::Out>,
        <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
    ): IfNot,
    (
        Graph<N, I, O>,
        <(
            Just<<<T as Typegraph>::Nodes as IntoList>::Out>,
            <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
        ) as IfNot>::Out,
    ): MaybeInsertNodes,
    (
        <(
            Graph<N, I, O>,
            <(
                Just<<<T as Typegraph>::Nodes as IntoList>::Out>,
                <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
            ) as IfNot>::Out,
        ) as MaybeInsertNodes>::Out,
        Just<U>,
    ): TypeResolvable,
    (
        <(
            <(
                Graph<N, I, O>,
                <(
                    Just<<<T as Typegraph>::Nodes as IntoList>::Out>,
                    <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
                ) as IfNot>::Out,
            ) as MaybeInsertNodes>::Out,
            Just<U>,
        ) as TypeResolvable>::Out,
        <(
            Just<<T as Typegraph>::Types>,
            <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
        ) as IfNot>::Out,
    ): TypeResolvable,
    (
        <(
            <(
                <(
                    Graph<N, I, O>,
                    <(
                        Just<<<T as Typegraph>::Nodes as IntoList>::Out>,
                        <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
                    ) as IfNot>::Out,
                ) as MaybeInsertNodes>::Out,
                Just<U>,
            ) as TypeResolvable>::Out,
            <(
                Just<<T as Typegraph>::Types>,
                <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
            ) as IfNot>::Out,
        ) as TypeResolvable>::Out,
        <(
            Just<<<T as Typegraph>::Edges as IntoList>::Out>,
            <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
        ) as IfNot>::Out,
    ): MaybeConnectNodes,
{
    type Out = <(
        <(
            <(
                <(
                    Graph<N, I, O>,
                    <(
                        Just<<<T as Typegraph>::Nodes as IntoList>::Out>,
                        <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
                    ) as IfNot>::Out,
                ) as MaybeInsertNodes>::Out,
                Just<U>,
            ) as TypeResolvable>::Out,
            <(
                Just<<T as Typegraph>::Types>,
                <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
            ) as IfNot>::Out,
        ) as TypeResolvable>::Out,
        <(
            Just<<<T as Typegraph>::Edges as IntoList>::Out>,
            <(Graph<N, I, O>, <T as Typegraph>::Id) as ContainsId>::Out,
        ) as IfNot>::Out,
    ) as MaybeConnectNodes>::Out;
}
