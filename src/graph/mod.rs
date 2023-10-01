#![forbid(unsafe_code)]

use core::marker::PhantomData;

use crate::types::*;

pub mod four_way_grid;

pub type NodeIndexValue = i16;
pub type Cost = i32;

const MAX_NEIGHBORS: usize = 8;

#[derive(Debug, Eq)]
pub struct NodeIndex<T: GraphBase> {
    value: NodeIndexValue,
    graph_type: PhantomData<T>,
}
impl<T: GraphBase> NodeIndex<T> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "debug")]{
            #[inline]
            pub fn new(value: NodeIndexValue) -> Result<Self, Error> {
                if value > T::MAX_NODE_INDEX {
                    Err(Error::OutOfRange)
                } else {
                    Ok(Self {
                        value,
                        graph_type: PhantomData,
                    })
                }
            }
        } else {
            #[inline]
            pub fn new(value: NodeIndexValue) -> Result<Self, Error> {
                Ok(Self { value, graph_type: PhantomData })
            }
        }
    }
    #[inline]
    pub fn value(&self) -> NodeIndexValue {
        self.value
    }
    #[inline]
    pub fn to_agent_state(&self, from_index: Option<NodeIndex<T>>) -> AgentState {
        T::agent_state_by_node_index(*self, from_index)
    }
}
// NOTE: we need to implement PartialEq and PartialOrd traits manually because T can be incompatible
impl<T: GraphBase> PartialEq for NodeIndex<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: GraphBase> PartialOrd for NodeIndex<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}
// NOTE: we need to implement Copy trait manually because T can be non-copiable
impl<T: GraphBase> Copy for NodeIndex<T> {}
impl<T: GraphBase> Clone for NodeIndex<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            graph_type: PhantomData,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Edge<T: GraphBase> {
    from: NodeIndex<T>,
    to: NodeIndex<T>,
    cost: Cost,
}
impl<T: GraphBase> Edge<T> {
    #[inline]
    pub fn new(from: NodeIndex<T>, to: NodeIndex<T>) -> Self {
        Self {
            from,
            to,
            cost: T::distance(from, to),
        }
    }
    #[inline]
    pub fn from(&self) -> NodeIndex<T> {
        self.from
    }
    #[inline]
    pub fn to(&self) -> NodeIndex<T> {
        self.to
    }
    #[inline]
    pub fn cost(&self) -> Cost {
        self.cost
    }
    #[inline]
    pub fn agent_state_at_from(&self) -> AgentState {
        self.from.to_agent_state(None)
    }
    #[inline]
    pub fn agent_state_at_to(&self) -> AgentState {
        self.to.to_agent_state(Some(self.from))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Route<T: GraphBase> {
    // TODO: use MAX_NODE_INDEX (once generic_const_exprs gets stabilized)
    nodes: Vec<NodeIndex<T>, { WIDTH * WIDTH }>,
    cost: Cost,
}

pub trait GraphBase: Sized {
    const MAX_NODE_INDEX: NodeIndexValue;
    fn distance(from: NodeIndex<Self>, to: NodeIndex<Self>) -> Cost;
    fn optimistic_distance(from: NodeIndex<Self>, to: NodeIndex<Self>) -> Cost;
    fn agent_state_by_node_index(
        index: NodeIndex<Self>,
        from_index: Option<NodeIndex<Self>>,
    ) -> AgentState;
    fn node_index_by_agent_state(agent_state: AgentState) -> Result<NodeIndex<Self>, Error>;
    fn neighbors(&self, from: NodeIndex<Self>) -> Vec<Edge<Self>, MAX_NEIGHBORS>;
    fn edge(&self, from: NodeIndex<Self>, to: NodeIndex<Self>) -> Option<Edge<Self>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct DummyGraph {}
    impl GraphBase for DummyGraph {
        const MAX_NODE_INDEX: NodeIndexValue = 10;
        fn distance(_from: NodeIndex<Self>, _to: NodeIndex<Self>) -> Cost {
            1
        }
        fn optimistic_distance(_from: NodeIndex<Self>, _to: NodeIndex<Self>) -> Cost {
            1
        }
        fn agent_state_by_node_index(
            _index: NodeIndex<Self>,
            _from_index: Option<NodeIndex<Self>>,
        ) -> AgentState {
            AgentState {
                location: CoordXY::with_u8(0, 0).unwrap(),
                local_location: CellLocalLocation::Center,
                heading_vector: VectorXY { x: 0, y: 1 },
            }
        }
        fn node_index_by_agent_state(_agent_state: AgentState) -> Result<NodeIndex<Self>, Error> {
            Ok(NodeIndex::new(1).unwrap())
        }
        fn neighbors(&self, _from: NodeIndex<Self>) -> Vec<Edge<Self>, MAX_NEIGHBORS> {
            Vec::<Edge<Self>, MAX_NEIGHBORS>::new()
        }
        fn edge(&self, _from: NodeIndex<Self>, _to: NodeIndex<Self>) -> Option<Edge<Self>> {
            None
        }
    }

    #[test]
    fn node_index_new() {
        assert_eq!(NodeIndex::<DummyGraph>::new(0).unwrap().value, 0)
    }
    #[test]
    fn node_index_new_out_of_range() {
        assert!(NodeIndex::<DummyGraph>::new(11).is_err());
        assert_eq!(
            NodeIndex::<DummyGraph>::new(11).err(),
            Some(Error::OutOfRange)
        );
    }
    #[test]
    fn node_index_to_agent_state() {
        assert_eq!(
            NodeIndex::<DummyGraph>::new(0)
                .unwrap()
                .to_agent_state(None),
            AgentState {
                location: CoordXY::with_u8(0, 0).unwrap(),
                local_location: CellLocalLocation::Center,
                heading_vector: VectorXY { x: 0, y: 1 },
            }
        );
    }
    #[test]
    fn edge_new() {
        let edge: Edge<DummyGraph> =
            Edge::new(NodeIndex::new(0).unwrap(), NodeIndex::new(1).unwrap());
        assert_eq!(edge.from.value, 0);
        assert_eq!(edge.to.value, 1);
        assert_eq!(edge.cost, 1);
    }
    #[test]
    fn edge_agent_state_at_from() {
        let edge: Edge<DummyGraph> =
            Edge::new(NodeIndex::new(0).unwrap(), NodeIndex::new(1).unwrap());
        let agent_state = edge.agent_state_at_from();
        assert_eq!(
            agent_state,
            AgentState {
                location: CoordXY::with_u8(0, 0).unwrap(),
                local_location: CellLocalLocation::Center,
                heading_vector: VectorXY { x: 0, y: 1 },
            }
        );
    }
    #[test]
    fn edge_agent_state_at_to() {
        let edge: Edge<DummyGraph> =
            Edge::new(NodeIndex::new(0).unwrap(), NodeIndex::new(1).unwrap());
        let agent_state = edge.agent_state_at_from();
        assert_eq!(
            agent_state,
            AgentState {
                location: CoordXY::with_u8(0, 0).unwrap(),
                local_location: CellLocalLocation::Center,
                heading_vector: VectorXY { x: 0, y: 1 },
            }
        );
    }
}
