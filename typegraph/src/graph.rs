use core::marker::PhantomData;

use typosaurus::bool::And;
use typosaurus::traits::semigroup::MappendG;
use typosaurus::{bool::False, cmp::Equality, num::UInt};

pub trait NodeOutputData {
    const ID: u32;
    const KIND: NodeOutputKind;
    const NAME: &str;
}

#[derive(Debug, Clone, Copy)]
pub enum NodeOutputKind {
    Struct,
    Enum,
    Function,
    AsyncFunction,
    Implementation,
    Primitive,
    Variant,
    UnaryContainer,
    BinaryContainer,
    Generic,
}

#[derive(Debug, Clone, Copy)]
pub enum EdgeOutputKind {
    MethodOf,
    PropertyOf,
    Invokes,
    Accesses,
    ImplementedBy,
    Provides,
}

pub trait EdgeOutputData {
    const FROM: u32;
    const TO: u32;
    const KIND: EdgeOutputKind;
}

pub trait Index {
    type Fields;
    type Methods;
}
pub trait ArgIndex<M> {
    type Args;
}
pub trait Implementation {
    type Methods;
}
pub trait ImplementorRef {
    type Ref;
}

pub struct NodeOutput<Id, Data, Meta>(PhantomData<Id>, PhantomData<Data>, PhantomData<Meta>);
impl<Id1, D1, M1, Id2, D2, M2> Equality<NodeOutput<Id1, D1, M1>> for NodeOutput<Id2, D2, M2>
where
    Id1: Equality<Id2>,
{
    type Out = <Id1 as Equality<Id2>>::Out;
}
impl<Id, D, M> Equality<()> for NodeOutput<Id, D, M> {
    type Out = False;
}
impl<Id, D, M> Equality<NodeOutput<Id, D, M>> for () {
    type Out = False;
}

pub struct Edge<From, To>(PhantomData<From>, PhantomData<To>);
impl<F1, F2, T1, T2> Equality<Edge<F2, T2>> for Edge<F1, T1>
where
    F1: Equality<F2>,
    T1: Equality<T2>,
    (<F1 as Equality<F2>>::Out, <T1 as Equality<T2>>::Out): And,
{
    type Out = <(<F1 as Equality<F2>>::Out, <T1 as Equality<T2>>::Out) as And>::Out;
}

pub struct MethodId<T>(PhantomData<T>);
impl<T, U> Equality<MethodId<T>> for MethodId<U> {
    type Out = ();
}

pub trait IntoId {
    type Out;
}
impl<T> IntoId for MethodId<T> {
    type Out = T;
}

pub trait IntoEdge {
    type Out;
}
impl<From, To> IntoEdge for Edge<From, To> {
    type Out = (From, To);
}
pub trait IntoNode {
    type Out;
}
impl<Id, Data, Meta> IntoNode for NodeOutput<Id, Data, Meta> {
    type Out = (Id, NodeOutput<Id, Data, Meta>);
}

pub struct EdgeOutput<Id, Data>(PhantomData<Id>, PhantomData<Data>);
impl<Id1, D1, Id2, D2> Equality<EdgeOutput<Id1, D1>> for EdgeOutput<Id2, D2>
where
    Id1: Equality<Id2>,
{
    type Out = <Id1 as Equality<Id2>>::Out;
}
impl<Id, D> Equality<()> for EdgeOutput<Id, D> {
    type Out = False;
}
impl<Id, D> Equality<EdgeOutput<Id, D>> for () {
    type Out = False;
}

impl<Id1, D1, M1, Id2, D2, M2> MappendG<NodeOutput<Id2, D2, M2>> for NodeOutput<Id1, D1, M1> {
    type Out = NodeOutput<Id1, D1, M1>;
}
impl<Id, D, M> MappendG<NodeOutput<Id, D, M>> for () {
    type Out = NodeOutput<Id, D, M>;
}
impl<Id, D, M> MappendG<()> for NodeOutput<Id, D, M> {
    type Out = NodeOutput<Id, D, M>;
}

pub trait Unsigned {
    const U32: u32;
}
impl Unsigned for typosaurus::num::UTerm {
    const U32: u32 = 0;
}
impl<U, B> Unsigned for typosaurus::num::UInt<U, B>
where
    U: typosaurus::num::Unsigned,
    B: typosaurus::num::Bit,
{
    const U32: u32 = <UInt<U, B> as typosaurus::num::Unsigned>::U32;
}
impl<T> Unsigned for typosaurus::collections::maybe::Just<T>
where
    T: Unsigned,
{
    const U32: u32 = <T as Unsigned>::U32;
}
impl Unsigned for typosaurus::collections::maybe::Nothing {
    const U32: u32 = 0;
}

/// A type that is also a graph.
pub trait Typegraph {
    type Id;
    type Node;
    type Nodes;
    type Edges;
    type Types;
}
