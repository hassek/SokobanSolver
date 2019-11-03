use std::cmp::Ordering;
use std::fmt;

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position { x, y }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Hash, Debug)]
pub enum NodeType {
    Empty = 0,
    Wall = 1,
    Whole = 2,
    Box = 3,
    Player = 4,
    BoxOnWhole = 5,
    PlayerOnWhole = 6,
}

impl NodeType {
    pub fn is_player(&self) -> bool {
        [NodeType::Player, NodeType::PlayerOnWhole].contains(self)
    }

    pub fn is_box(&self) -> bool {
        [NodeType::Box, NodeType::BoxOnWhole].contains(self)
    }

    pub fn is_whole(&self) -> bool {
        [NodeType::Whole, NodeType::BoxOnWhole, NodeType::PlayerOnWhole].contains(self)
    }

    pub fn can_move(&self) -> bool {
        ![NodeType::Box, NodeType::BoxOnWhole, NodeType::Wall].contains(self)
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            NodeType::Empty => ' ',
            NodeType::Wall => '#',
            NodeType::Whole => '.',
            NodeType::Box => '$',
            NodeType::Player => '@',
            NodeType::BoxOnWhole => '*',
            NodeType::PlayerOnWhole => '+',
        };
        write!(f, "{}", printable)
    }
}

impl NodeType {
    pub fn build(node_type: u32) -> Result<NodeType, &'static str> {
        match node_type {
            0 => Ok(NodeType::Empty),
            1 => Ok(NodeType::Wall),
            2 => Ok(NodeType::Whole),
            3 => Ok(NodeType::Box),
            4 => Ok(NodeType::Player),
            5 => Ok(NodeType::BoxOnWhole),
            6 => Ok(NodeType::PlayerOnWhole),
            _ => Err("No match for NodeType"),
        }
    }

    // Reverse type to play reverse sokoban (pulling instead of pushing)
    pub fn reverse_build(node_type: u32) -> Result<NodeType, &'static str> {
        match node_type {
            0 => Ok(NodeType::Empty),
            1 => Ok(NodeType::Wall),
            2 => Ok(NodeType::Box),
            3 => Ok(NodeType::Whole),
            4 => Ok(NodeType::Player),
            5 => Ok(NodeType::BoxOnWhole),
            6 => Ok(NodeType::Box),
            _ => Err("No match for NodeType"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub ntype: NodeType,
    pub position: Position,
}

impl Node {
    pub fn loop_positions(pos: &Position) -> Vec<Position> {
        let mut arr = Vec::new();
        if pos.x > 0 {
            arr.push(Position {
                x: pos.x - 1,
                y: pos.y,
            });
        }

        arr.push(Position {
            x: pos.x + 1,
            y: pos.y,
        });

        if pos.y > 0 {
            arr.push(Position {
                x: pos.x,
                y: pos.y - 1,
            });
        }
        arr.push(Position {
                x: pos.x,
                y: pos.y + 1,
            });
        arr
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} | {}", self.ntype, self.position)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.ntype == other.ntype
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ntype.partial_cmp(&other.ntype)
    }
}

// XXX need to implement hashes properly
// impl Hash for Node {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         let hash = 31;
//         if self.ntype == NodeType.Player {
//             hash = 31 * hash;
//         } else if self.ntype == NodeType.PlayerOnWhole {
//             hash = (31 * hash) + 2;
//         } else {
//             hash = 31 * hash + self.ntype.hash();
//         }

//         ( 31 * hash ) + self.position.hash()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordering() {
        let node1 = Node {ntype: NodeType::Empty, position: Position::new(0, 0) };
        let node2 = Node {ntype: NodeType::Wall, position: Position::new(0, 1) };
        let node3 = Node {ntype: NodeType::Empty, position: Position::new(0, 2) };

        assert_eq!(node1 != node2, true);
        assert_eq!(node1 < node2, true);
        assert_eq!(node1 == node3, true);
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(1, 1);
        let pos3 = Position::new(1, 1);

        assert_eq!(pos1 == pos2, false);
        assert_eq!(pos2 == pos3, true);
    }

    #[test]
    fn test_node_type_build() {
        let ntype = NodeType::build(2).unwrap();
        assert_eq!(ntype, NodeType::Whole);
    }
}
