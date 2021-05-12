use std::collections::HashMap;

use petgraph::graphmap::DiGraphMap;

use crate::types::{ NodeType, EdgeType };

#[derive(Debug)]
pub struct PageGraphDescriptor {
    pub version: String,
    pub about: String,
    pub url: String,
    pub is_root: bool,
    pub frame_id: FrameId,
    pub time: PageGraphTime,
}

#[derive(Debug)]
pub struct PageGraphTime {
    pub start: u64,
    pub end: u64,
}

/// The main PageGraph data structure.
#[derive(Debug)]
pub struct PageGraph {
    pub desc: PageGraphDescriptor,
    pub edges: HashMap<EdgeId, Edge>,
    pub nodes: HashMap<NodeId, Node>,
    pub graph: DiGraphMap<NodeId, Vec<EdgeId>>,

    next_node_id: std::cell::RefCell<usize>,
    next_edge_id: std::cell::RefCell<usize>,
}

impl PageGraph {
    pub fn new(desc: PageGraphDescriptor, edges: HashMap<EdgeId, Edge>, nodes: HashMap<NodeId, Node>, graph: DiGraphMap<NodeId, Vec<EdgeId>>) -> Self {
        Self {
            desc,
            edges,
            nodes,
            graph,
            next_edge_id: std::cell::RefCell::new(usize::MAX),
            next_node_id: std::cell::RefCell::new(usize::MAX),
        }
    }

    /// Returns a new edge id that is guaranteed not to collide with an existing id in the graph.
    pub(crate) fn new_edge_id(&self) -> EdgeId {
        let new_id = EdgeId::from(self.next_edge_id.replace_with(|id| *id - 1));
        assert!(!self.edges.contains_key(&new_id));
        new_id
    }

    pub fn source_node<'a>(&'a self, edge: &Edge) -> &'a Node {
        self.nodes.get(&edge.source).unwrap_or_else(|| panic!("Source node for edge {:?} could not be found in the graph", edge))
    }

    pub fn target_node<'a>(&'a self, edge: &Edge) -> &'a Node {
        self.nodes.get(&edge.target).unwrap_or_else(|| panic!("Target node for edge {:?} could not be found in the graph", edge))
    }

    pub fn outgoing_edges<'a>(&'a self, node: &Node) -> impl Iterator<Item=&'a Edge> {
        self.edges_iter_directed(node, petgraph::Direction::Outgoing)
    }

    pub fn incoming_edges<'a>(&'a self, node: &Node) -> impl Iterator<Item=&'a Edge> {
        self.edges_iter_directed(node, petgraph::Direction::Incoming)
    }

    fn edges_iter_directed<'a>(&'a self, node: &Node, direction: petgraph::Direction) -> impl Iterator<Item=&'a Edge> {
        self.graph.edges_directed(node.id, direction).map(move |(_a, _b, edge_ids)| {
            edge_ids
        })
            .flatten()
            .map(move |edge_id| {
                self.edges.get(&edge_id).unwrap()
            })
    }

    pub fn outgoing_neighbors<'a>(&'a self, node: &Node) -> impl Iterator<Item=&'a Node> {
        self.nodes_iter_directed(node, petgraph::Direction::Outgoing)
    }

    pub fn incoming_neighbors<'a>(&'a self, node: &Node) -> impl Iterator<Item=&'a Node> {
        self.nodes_iter_directed(node, petgraph::Direction::Incoming)
    }

    fn nodes_iter_directed<'a>(&'a self, node: &Node, direction: petgraph::Direction) -> impl Iterator<Item=&'a Node> {
        self.graph.neighbors_directed(node.id, direction).map(move |node_id| {
            self.nodes.get(&node_id).unwrap()
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
struct GraphItemId {
    id: usize,
    frame_id: Option<FrameId>,
}

impl From<usize> for GraphItemId {
    fn from(v: usize) -> Self {
        Self {
            id: v,
            frame_id: None
        }
    }
}

impl GraphItemId {
    fn copy_for_frame_id(&self, frame_id: &FrameId) -> Self {
        Self {
            id: self.id,
            frame_id: Some(frame_id.clone()),
        }
    }
}

pub trait HasFrameId {
    fn get_frame_id(&self) -> Option<FrameId>;
}

pub fn is_same_frame_context<A: HasFrameId, B: HasFrameId>(a: A, b: B) -> bool {
    a.get_frame_id() == b.get_frame_id()
}

/// An identifier used to reference a node.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct NodeId(GraphItemId);

impl From<usize> for NodeId {
    fn from(v: usize) -> Self {
        Self(v.into())
    }
}

impl NodeId {
    pub fn copy_for_frame_id(&self, frame_id: &FrameId) -> Self {
        Self(self.0.copy_for_frame_id(frame_id))
    }
}

impl HasFrameId for NodeId {
    fn get_frame_id(&self) -> Option<FrameId> {
        self.0.frame_id
    }
}

/// A node, representing a side effect of a page load.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub node_timestamp: isize,
    pub node_type: NodeType,
}

/// An identifier used to reference an edge.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct EdgeId(GraphItemId);

impl From<usize> for EdgeId {
    fn from(v: usize) -> Self {
        EdgeId(v.into())
    }
}

impl EdgeId {
    pub fn copy_for_frame_id(&self, frame_id: &FrameId) -> Self {
        Self(self.0.copy_for_frame_id(frame_id))
    }
}

impl HasFrameId for EdgeId {
    fn get_frame_id(&self) -> Option<FrameId> {
        self.0.frame_id
    }
}

/// An edge, representing an action taken during page load.
#[derive(Debug, Clone)]
pub struct Edge {
    pub id: EdgeId,
    pub edge_timestamp: Option<isize>,
    pub edge_type: EdgeType,
    pub source: NodeId,
    pub target: NodeId,
}

impl PartialEq for Edge {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct FrameId(u128);

impl From<&str> for FrameId {
    /// Chromium formats these 128-bit tokens as 32-character hexadecimal strings.
    fn from(v: &str) -> Self {
        assert_eq!(v.len(), 32);
        Self(u128::from_str_radix(v, 16).unwrap_or_else(|_| panic!("{} is an incorrectly formatted frame id", v)))
    }
}

impl std::fmt::Display for FrameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>32X}", self.0)
    }
}

impl std::fmt::Debug for FrameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{:0>32X}\"", self.0)
    }
}
