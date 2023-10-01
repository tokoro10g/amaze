#![forbid(unsafe_code)]

use crate::graph::*;

#[derive(Debug)]
pub struct Graph {
    pub maze: Maze,
}
impl Graph {
    fn coord_xy_by_node_index(index: NodeIndex<Self>) -> Result<CoordXY, Error> {
        let x = (index.value as u8) % WIDTH as u8;
        let y = (index.value as u8) / WIDTH as u8;
        CoordXY::with_u8(x, y)
    }
    fn vector_xy_by_node_index_pair(from: NodeIndex<Self>, to: NodeIndex<Self>) -> VectorXY {
        Self::coord_xy_by_node_index(to).unwrap() - Self::coord_xy_by_node_index(from).unwrap()
    }
    fn node_index_diff_by_vector_xy(vector: VectorXY) -> NodeIndexValue {
        vector.x as NodeIndexValue + vector.y as NodeIndexValue * WIDTH as NodeIndexValue
    }
    fn node_index_by_coord_xy(coord: CoordXY) -> Result<NodeIndex<Self>, Error> {
        NodeIndex::new(
            coord.x().value() as NodeIndexValue
                + coord.y().value() as NodeIndexValue * WIDTH as NodeIndexValue,
        )
    }
    fn edge_impl(cell: Cell, direction: Direction, index: NodeIndex<Self>) -> Option<Edge<Self>> {
        let to = NodeIndex::new(index.value + Self::node_index_diff_by_vector_xy(direction.into()))
            .unwrap();
        if !cell.state_by_direction(direction) {
            return Some(Edge::new(index, to));
        }
        None
    }
}
impl GraphBase for Graph {
    const MAX_NODE_INDEX: NodeIndexValue = WIDTH as NodeIndexValue * WIDTH as NodeIndexValue - 1;
    fn distance(from: NodeIndex<Graph>, to: NodeIndex<Graph>) -> Cost {
        Graph::optimistic_distance(from, to)
    }
    fn optimistic_distance(from: NodeIndex<Graph>, to: NodeIndex<Graph>) -> Cost {
        let vector = Self::vector_xy_by_node_index_pair(from, to);
        vector.x.abs() as Cost + vector.y.abs() as Cost
    }
    fn agent_state_by_node_index(
        index: NodeIndex<Graph>,
        from_index: Option<NodeIndex<Graph>>,
    ) -> AgentState {
        let x = (index.value as u8) % WIDTH as u8;
        let y = (index.value as u8) / WIDTH as u8;
        let mut state = AgentState {
            location: CoordXY::with_u8(x, y).unwrap(),
            local_location: CellLocalLocation::Center,
            heading_vector: VectorXY { x: 0, y: 0 },
        };
        if let Some(from_index) = from_index {
            state.heading_vector = Self::vector_xy_by_node_index_pair(from_index, index);
        }
        state
    }
    fn node_index_by_agent_state(agent_state: AgentState) -> Result<NodeIndex<Self>, Error> {
        if agent_state.local_location != CellLocalLocation::Center {
            return Err(Error::InvalidLocation);
        }
        Ok(Self::node_index_by_coord_xy(agent_state.location).unwrap())
    }
    fn edge(&self, from: NodeIndex<Self>, to: NodeIndex<Self>) -> Option<Edge<Self>> {
        let state = Graph::agent_state_by_node_index(from, None);
        let cell = self.maze.cell(state.location);
        if let Ok(direction) = Self::vector_xy_by_node_index_pair(from, to).try_into() {
            return Self::edge_impl(cell, direction, from);
        }
        None
    }
    fn neighbors(&self, from: NodeIndex<Self>) -> Vec<Edge<Self>, MAX_NEIGHBORS> {
        let state = Graph::agent_state_by_node_index(from, None);
        let cell = self.maze.cell(state.location);
        let mut vec = Vec::<Edge<Graph>, MAX_NEIGHBORS>::new();
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            if let Some(edge) = Self::edge_impl(cell, direction, from) {
                vec.push(edge).unwrap();
            }
        }
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAZE_STR: &str = "\
        +   +   +   +   +\n\
        |                \n\
        +   +---+---+---+\n\
        |   |           |\n\
        +   +   +   +   +\n\
        |   |           |\n\
        +---+   +   +   +\n\
        |               |\n\
        +---+---+---+---+\n";

    #[test]
    fn vector_xy_by_node_index_pair() {
        assert_eq!(
            Graph::vector_xy_by_node_index_pair(
                NodeIndex::new(0).unwrap(),
                NodeIndex::new(1).unwrap()
            ),
            VectorXY { x: 1, y: 0 }
        );
        assert_eq!(
            Graph::vector_xy_by_node_index_pair(
                NodeIndex::new(1).unwrap(),
                NodeIndex::new(WIDTH as NodeIndexValue).unwrap()
            ),
            VectorXY { x: -1, y: 1 }
        );
        assert_eq!(
            Graph::vector_xy_by_node_index_pair(
                NodeIndex::new(WIDTH as NodeIndexValue - 1).unwrap(),
                NodeIndex::new(WIDTH as NodeIndexValue).unwrap()
            ),
            VectorXY {
                x: -(WIDTH as i8 - 1),
                y: 1
            }
        );
    }
    #[test]
    fn node_index_diff_by_vector_xy() {
        assert_eq!(
            Graph::node_index_diff_by_vector_xy(VectorXY { x: 2, y: 4 }),
            WIDTH as NodeIndexValue * 4 + 2
        )
    }
    #[test]
    fn node_index_by_coord_xy() {
        assert_eq!(
            Graph::node_index_by_coord_xy(CoordXY::with_u8(2, 4).unwrap())
                .unwrap()
                .value,
            WIDTH as NodeIndexValue * 4 + 2
        )
    }
    #[test]
    fn edge_impl() {
        let mut cell = Cell::new();
        cell.set_east(true);
        let index: NodeIndex<Graph> = NodeIndex::new(WIDTH as NodeIndexValue + 1).unwrap();
        assert!(Graph::edge_impl(cell, Direction::East, index).is_none());
        let edge = Graph::edge_impl(cell, Direction::South, index);
        assert!(edge.is_some());
        let edge = edge.unwrap();
        assert_eq!(edge.from, index);
        assert_eq!(edge.to.value, 1);
    }
    #[test]
    fn distance() {
        let d = Graph::distance(
            NodeIndex::new(1).unwrap(),
            NodeIndex::new(WIDTH as NodeIndexValue + 3).unwrap(),
        );
        assert_eq!(d, 3);
    }
    #[test]
    fn optimistic_distance() {
        let d = Graph::optimistic_distance(
            NodeIndex::new(1).unwrap(),
            NodeIndex::new(WIDTH as NodeIndexValue + 3).unwrap(),
        );
        assert_eq!(d, 3);
    }
    #[test]
    fn agent_state_by_node_index() {
        let state = Graph::agent_state_by_node_index(
            NodeIndex::new(WIDTH as NodeIndexValue + 2).unwrap(),
            None,
        );
        assert_eq!(state.location.x().value(), 2);
        assert_eq!(state.location.y().value(), 1);
        assert_eq!(state.local_location, CellLocalLocation::Center);
        assert_eq!(state.heading_vector, VectorXY { x: 0, y: 0 });
    }
    #[test]
    fn agent_state_by_node_index_with_from_index() {
        let state = Graph::agent_state_by_node_index(
            NodeIndex::new(WIDTH as NodeIndexValue + 2).unwrap(),
            Some(NodeIndex::new(WIDTH as NodeIndexValue + 1).unwrap()),
        );
        assert_eq!(state.location.x().value(), 2);
        assert_eq!(state.location.y().value(), 1);
        assert_eq!(state.local_location, CellLocalLocation::Center);
        assert_eq!(state.heading_vector, VectorXY { x: 1, y: 0 });
    }
    #[test]
    fn node_index_by_agent_state() {
        let node_index = Graph::node_index_by_agent_state(AgentState {
            location: CoordXY::with_u8(2, 3).unwrap(),
            local_location: CellLocalLocation::Center,
            heading_vector: VectorXY { x: 0, y: 0 },
        })
        .unwrap();
        assert_eq!(node_index.value(), 3 * WIDTH as NodeIndexValue + 2);
    }
    #[test]
    fn node_index_by_agent_state_with_invalid_local_location() {
        let node_index_result = Graph::node_index_by_agent_state(AgentState {
            location: CoordXY::with_u8(2, 3).unwrap(),
            local_location: CellLocalLocation::North,
            heading_vector: VectorXY { x: 0, y: 0 },
        });
        assert!(node_index_result.is_err());
        assert_eq!(node_index_result.err(), Some(Error::InvalidLocation));
    }
    #[test]
    fn edge() {
        let g = Graph {
            maze: Maze::load_from_str(MAZE_STR),
        };
        let edge = g.edge(NodeIndex::new(0).unwrap(), NodeIndex::new(1).unwrap());
        assert!(edge.is_some());
        let edge = edge.unwrap();
        assert_eq!(edge.from.value, 0);
        assert_eq!(edge.to.value, 1);
        assert_eq!(edge.cost, 1);

        let edge = g.edge(
            NodeIndex::new(0).unwrap(),
            NodeIndex::new(WIDTH as NodeIndexValue).unwrap(),
        );
        assert!(edge.is_none());
    }
    #[test]
    fn edge_with_invalid_to() {
        let g = Graph {
            maze: Maze::load_from_str(MAZE_STR),
        };
        let edge = g.edge(NodeIndex::new(0).unwrap(), NodeIndex::new(3).unwrap());
        assert!(edge.is_none());
    }
    #[test]
    fn neighbors() {
        let g = Graph {
            maze: Maze::load_from_str(MAZE_STR),
        };
        let n = g.neighbors(NodeIndex::new(0).unwrap());
        assert_eq!(n.len(), 1);
        let edge = n.get(0).unwrap();
        assert_eq!(edge.from.value, 0);
        assert_eq!(edge.to.value, 1);
        assert_eq!(edge.cost, 1);

        let n = g.neighbors(NodeIndex::new(WIDTH as NodeIndexValue + 2).unwrap());
        assert_eq!(n.len(), 4);
    }
}
