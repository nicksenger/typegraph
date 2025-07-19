use typosaurus::collections::graph::{Graph, OutgoingEdgeList, ValueList};
use typosaurus::collections::list::{self, List};
use typosaurus::collections::Container;

use crate::graph::{
    EdgeOutput, EdgeOutputData, NodeOutput, NodeOutputData, NodeOutputKind, Unsigned,
};

#[cfg(feature = "graphviz")]
pub mod graphviz;

pub trait ValueGraph<T = NodeKind> {
    type NodeList;
    type EdgeList;

    fn value() -> petgraph::Graph<T, EdgeKindWithIxs<petgraph::graph::NodeIndex>>;
}
impl<T, N, I, O> ValueGraph<T> for Graph<N, I, O>
where
    Graph<N, I, O>: ValueList + OutgoingEdgeList,
    <Graph<N, I, O> as ValueList>::Out: Vectorize<Node<T>>,
    <Graph<N, I, O> as OutgoingEdgeList>::Out: Vectorize<(u32, u32)>,
{
    type NodeList = <Graph<N, I, O> as ValueList>::Out;
    type EdgeList = <Graph<N, I, O> as OutgoingEdgeList>::Out;

    fn value() -> petgraph::Graph<T, EdgeKindWithIxs<petgraph::graph::NodeIndex>> {
        let edges = <Self::EdgeList as Vectorize<(u32, u32)>>::to_vec();
        let data = <Self::NodeList as Vectorize<Node<T>>>::to_vec();
        let mut graph: petgraph::Graph<T, EdgeKindWithIxs<petgraph::graph::NodeIndex>> =
            petgraph::Graph::new();
        let mut ixs = std::collections::HashMap::new();
        let mut nodes = std::collections::HashMap::new();
        for node in data {
            let id = node.id();
            let (node, data) = node.swap(());
            let ix = graph.add_node(data);
            nodes.insert(id, node);
            ixs.insert(id, ix);
        }
        for (from, to) in edges {
            let (a, b) = (nodes.get(&from).unwrap(), nodes.get(&to).unwrap());
            let (from, to) = (*ixs.get(&from).unwrap(), *ixs.get(&to).unwrap());
            graph.add_edge(from, to, ConnectedEdgeKind::implied(a, b).map(&ixs));
        }

        graph
    }
}

pub trait Value<T> {
    fn value() -> T;
}
impl<T> Value<u32> for T
where
    T: Unsigned,
{
    fn value() -> u32 {
        <T as Unsigned>::U32
    }
}
impl<T, U> Value<(u32, u32)> for (T, U)
where
    T: Unsigned,
    U: Unsigned,
{
    fn value() -> (u32, u32) {
        (<T as Unsigned>::U32, <U as Unsigned>::U32)
    }
}
impl Value<()> for () {
    fn value() {}
}

pub trait Vectorize<T> {
    fn to_vec() -> Vec<T>;
    fn fill(v: &mut Vec<T>);
}
impl<T> Vectorize<T> for list::Empty {
    fn to_vec() -> Vec<T> {
        vec![]
    }
    fn fill(_v: &mut Vec<T>) {}
}
impl<T, U, V> Vectorize<T> for List<(U, V)>
where
    List<(U, V)>: Container,
    <List<(U, V)> as Container>::Content: Value<T>,
    V: Vectorize<T>,
{
    fn to_vec() -> Vec<T> {
        let mut v = vec![];
        Self::fill(&mut v);

        v
    }

    fn fill(v: &mut Vec<T>) {
        v.push(<<List<(U, V)> as Container>::Content as Value<T>>::value());
        <<List<(U, V)> as list::Tailed>::Tail as Vectorize<T>>::fill(v);
    }
}

impl<Id, D> Value<Output> for EdgeOutput<Id, D>
where
    Id: Unsigned,
    D: EdgeOutputData,
{
    fn value() -> Output {
        todo!()
    }
}

impl<Id, D, Mt, Mv> Value<Node<Mv>> for NodeOutput<Id, D, Mt>
where
    Id: Unsigned,
    D: NodeOutputData,
    Mt: Value<Mv>,
{
    fn value() -> Node<Mv> {
        let (id, name, kind) = (
            <D as NodeOutputData>::ID,
            <D as NodeOutputData>::NAME,
            <D as NodeOutputData>::KIND,
        );
        match kind {
            NodeOutputKind::Struct => Node::Struct(id, name, Mt::value()),
            NodeOutputKind::Generic => Node::Generic(id, name, Mt::value()),
            NodeOutputKind::Enum => Node::Enum(id, name, Mt::value()),
            NodeOutputKind::Function => Node::Function(id, name, Mt::value()),
            NodeOutputKind::AsyncFunction => Node::AsyncFunction(id, name, Mt::value()),
            NodeOutputKind::Implementation => Node::InherentImplementation(id, name, Mt::value()),
            NodeOutputKind::Primitive => Node::Primitive(id, name, Mt::value()),
            NodeOutputKind::Variant => Node::Variant(id, name, Mt::value()),
            NodeOutputKind::UnaryContainer => Node::UnaryContainer(id, name, Mt::value()),
            NodeOutputKind::BinaryContainer => Node::BinaryContainer(id, name, Mt::value()),
        }
    }
}

pub enum Node<T> {
    Struct(u32, &'static str, T),
    Generic(u32, &'static str, T),
    Enum(u32, &'static str, T),
    InherentImplementation(u32, &'static str, T),
    TraitImplementation(u32, &'static str, T),
    Function(u32, &'static str, T),
    AsyncFunction(u32, &'static str, T),
    Primitive(u32, &'static str, T),
    Variant(u32, &'static str, T),
    UnaryContainer(u32, &'static str, T),
    BinaryContainer(u32, &'static str, T),
}
impl<T> Node<T> {
    fn id(&self) -> u32 {
        match self {
            Self::Struct(id, _, _)
            | Self::Enum(id, _, _)
            | Self::Generic(id, _, _)
            | Self::InherentImplementation(id, _, _)
            | Self::TraitImplementation(id, _, _)
            | Self::Function(id, _, _)
            | Self::AsyncFunction(id, _, _)
            | Self::Primitive(id, _, _)
            | Self::Variant(id, _, _)
            | Self::UnaryContainer(id, _, _)
            | Self::BinaryContainer(id, _, _) => *id,
        }
    }

    fn swap<U>(self, input: U) -> (Node<U>, T) {
        match self {
            Self::Struct(id, s, d) => (Node::Struct(id, s, input), d),
            Self::Generic(id, s, d) => (Node::Generic(id, s, input), d),
            Self::Enum(id, s, d) => (Node::Enum(id, s, input), d),
            Self::InherentImplementation(id, s, d) => {
                (Node::InherentImplementation(id, s, input), d)
            }
            Self::TraitImplementation(id, s, d) => (Node::TraitImplementation(id, s, input), d),
            Self::Function(id, s, d) => (Node::Function(id, s, input), d),
            Self::AsyncFunction(id, s, d) => (Node::AsyncFunction(id, s, input), d),
            Self::Primitive(id, s, d) => (Node::Primitive(id, s, input), d),
            Self::Variant(id, s, d) => (Node::Variant(id, s, input), d),
            Self::UnaryContainer(id, s, d) => (Node::UnaryContainer(id, s, input), d),
            Self::BinaryContainer(id, s, d) => (Node::BinaryContainer(id, s, input), d),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EdgeKind {
    Property,
    Variant,
    Implementation,
    Function,
    AsyncFunction,
    Argument,
    Returns,
    Call,
    Contains,
    Generic,
    Unknown,
}
impl EdgeKind {
    fn implied<T>(a: &Node<T>, b: &Node<T>) -> Self {
        match (a, b) {
            (
                Node::UnaryContainer { .. } | Node::BinaryContainer { .. },
                Node::UnaryContainer { .. }
                | Node::BinaryContainer { .. }
                | Node::Struct { .. }
                | Node::Enum { .. }
                | Node::Primitive { .. }
                | Node::Generic { .. },
            ) => Self::Contains,
            (Node::Enum { .. }, Node::Variant { .. }) => Self::Variant,
            (Node::Function { .. }, Node::Function { .. }) => Self::Call,
            (
                Node::Struct { .. } | Node::Enum { .. },
                Node::InherentImplementation { .. } | Node::TraitImplementation { .. },
            ) => Self::Implementation,
            (
                Node::TraitImplementation { .. } | Node::InherentImplementation { .. },
                Node::Function { .. },
            ) => Self::Function,
            (
                Node::TraitImplementation { .. } | Node::InherentImplementation { .. },
                Node::AsyncFunction { .. },
            ) => Self::AsyncFunction,
            (
                Node::Function { .. } | Node::AsyncFunction { .. },
                Node::Struct { .. }
                | Node::Enum { .. }
                | Node::Primitive(_, _, _)
                | Node::UnaryContainer { .. }
                | Node::BinaryContainer { .. }
                | Node::Generic { .. },
            ) => Self::Returns,
            (
                Node::Variant { .. } | Node::Struct { .. } | Node::Enum { .. },
                Node::Struct { .. }
                | Node::Enum { .. }
                | Node::Primitive(_, _, _)
                | Node::UnaryContainer { .. }
                | Node::BinaryContainer { .. },
            ) => Self::Property,
            (
                Node::Struct { .. }
                | Node::Enum { .. }
                | Node::Primitive(_, _, _)
                | Node::BinaryContainer { .. }
                | Node::UnaryContainer { .. }
                | Node::Generic { .. },
                Node::Function { .. } | Node::AsyncFunction { .. },
            ) => Self::Argument,
            (
                Node::Generic { .. },
                Node::InherentImplementation { .. } | Node::TraitImplementation { .. },
            ) => Self::Generic,
            (Node::Variant { .. } | Node::Struct { .. }, Node::Generic { .. }) => Self::Property,

            _ => Self::Unknown,
        }
    }

    pub fn arrowhead(&self) -> &str {
        match self {
            Self::Property | Self::Contains => "dot",
            Self::Variant => "obox",
            Self::Function => "odot",
            Self::AsyncFunction => "odot",
            Self::Implementation => "dot",
            Self::Argument => "normal",
            Self::Generic => "normal",
            Self::Returns => "vee",
            _ => "normal",
        }
    }

    pub fn weight(&self) -> u8 {
        1
    }

    pub fn penwidth(&self) -> u8 {
        1
    }

    pub fn color(&self) -> &str {
        match self {
            Self::Returns => "#7aa2f7",
            Self::Implementation => "#9ece6a",
            Self::Function => "#e0af68",
            Self::AsyncFunction => "#ff9e64",
            Self::Property => "#7aa2f67",
            Self::Contains => "#7aa2f67",
            Self::Argument => "#e0af68",
            Self::Generic => "#bb9af7",
            Self::Variant => "#7dcfff",
            _ => "black",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Output {
    Struct { id: u32, name: &'static str },
    Enum { id: u32, name: &'static str },
    InherentImplementation { id: u32 },
    TraitImplementation { id: u32, trait_name: &'static str },
    Method { id: u32, name: &'static str },
}

impl Output {
    pub fn id(&self) -> u32 {
        match *self {
            Self::InherentImplementation { id } => id,
            Self::Enum { id, .. } => id,
            Self::Struct { id, .. } => id,
            Self::TraitImplementation { id, .. } => id,
            Self::Method { id, .. } => id,
        }
    }
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    Struct(
        &'static str,
        &'static [&'static str],
        &'static [(&'static str, u32)],
    ),
    Generic(
        &'static str,
        &'static [&'static str],
        &'static [(&'static str, u32)],
    ),
    Enum(
        &'static str,
        &'static [&'static str],
        &'static [(&'static str, u32)],
    ),
    Variant(
        &'static str,
        &'static [&'static str],
        &'static [(&'static str, u32)],
    ),
    Implementation(&'static str, &'static [&'static str]),
    Function(
        &'static str,
        &'static [&'static str],
        &'static [(&'static str, u32)],
    ),
    AsyncFunction(
        &'static str,
        &'static [&'static str],
        &'static [(&'static str, u32)],
    ),
    Primitive(&'static str, &'static [&'static str]),
    UnaryContainer(&'static str, &'static [&'static str]),
    BinaryContainer(&'static str, &'static [&'static str]),
}
impl NodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Struct(s, _, _)
            | Self::Enum(s, _, _)
            | Self::Generic(s, _, _)
            | Self::Variant(s, _, _)
            | Self::Implementation(s, _)
            | Self::Function(s, _, _)
            | Self::AsyncFunction(s, _, _)
            | Self::Primitive(s, _)
            | Self::UnaryContainer(s, _)
            | Self::BinaryContainer(s, _) => s,
        }
    }

    pub fn shape(&self) -> &str {
        match self {
            Self::Struct(_, _, _) => "box3d",
            Self::Generic(_, _, _) => "diamond",
            Self::UnaryContainer(_, _) => "box3d",
            Self::BinaryContainer(_, _) => "box3d",
            Self::Enum(_, _, _) => "folder",
            Self::Variant(_, _, _) => "note",
            Self::Implementation(_, _) => "cylinder",
            Self::Function(_, _, _) => "ellipse",
            Self::AsyncFunction(_, _, _) => "ellipse",
            Self::Primitive(_, _) => "square",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Self::Struct(_, _, _) => "#7aa2f7",
            Self::Generic(_, _, _) => "#bb9af7",
            Self::UnaryContainer(_, _) => "#7aa2f7",
            Self::BinaryContainer(_, _) => "#7aa2f7",
            Self::Enum(_, _, _) => "#7dcfff",
            Self::Variant(_, _, _) => "#7dcfff",
            Self::Implementation(_, _) => "#9ece6a",
            Self::Function(_, _, _) => "#e0af68",
            Self::AsyncFunction(_, _, _) => "#ff9e64",
            Self::Primitive(_, _) => "#c0caf5",
        }
    }

    pub fn cluster(&self) -> &'static [&'static str] {
        match self {
            Self::Struct(_, c, _)
            | Self::Enum(_, c, _)
            | Self::Generic(_, c, _)
            | Self::Variant(_, c, _)
            | Self::Implementation(_, c)
            | Self::Function(_, c, _)
            | Self::AsyncFunction(_, c, _)
            | Self::Primitive(_, c)
            | Self::UnaryContainer(_, c)
            | Self::BinaryContainer(_, c) => c,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdgeKindWithIxs<T> {
    pub kind: EdgeKind,
    pub from: u32,
    pub to: u32,
    pub from_ix: T,
    pub to_ix: T,
}
impl<T> EdgeKindWithIxs<T> {
    pub fn label(&self, meta: &str) -> String {
        use EdgeKind::*;
        match self.kind {
            Property => format!("field ({meta})"),
            Variant => "variant".to_string(),
            Implementation => "impl".to_string(),
            Function => "fn".to_string(),
            AsyncFunction => "async fn".to_string(),
            Argument => format!("arg ({meta})"),
            Returns => "returns".to_string(),
            Call => "calls".to_string(),
            Contains => "content".to_string(),
            Generic => "generic".to_string(),
            Unknown => "unknown".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ConnectedEdgeKind {
    pub kind: EdgeKind,
    pub from: u32,
    pub to: u32,
}
impl ConnectedEdgeKind {
    pub fn implied<T>(a: &Node<T>, b: &Node<T>) -> Self {
        Self {
            kind: EdgeKind::implied(a, b),
            from: a.id(),
            to: b.id(),
        }
    }

    pub fn map<T: Copy>(self, ids: &std::collections::HashMap<u32, T>) -> EdgeKindWithIxs<T> {
        EdgeKindWithIxs {
            kind: self.kind,
            from: self.from,
            to: self.to,
            from_ix: *ids.get(&self.from).unwrap(),
            to_ix: *ids.get(&self.to).unwrap(),
        }
    }
}
