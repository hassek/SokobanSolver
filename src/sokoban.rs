use crate::node::{Node, NodeType, Position};
use log::debug;
use std::char;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Iterator for Direction {
    type Item = Direction;

    fn next(&mut self) -> Option<Direction> {
        match self {
            Direction::Up => Some(Direction::Down),
            Direction::Down => Some(Direction::Left),
            Direction::Left => Some(Direction::Right),
            Direction::Right => Some(Direction::Up),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Sokoban {
    pub width: usize,
    pub height: usize,
    pub map: HashMap<Position, NodeType>,
    player_reachable: Option<Vec<Vec<u8>>>,
    pub player: Option<Position>,
    pub goals: Vec<Position>,
    pub boxes: Vec<Position>,
}

impl Sokoban {
    pub fn new(level: &String) -> Sokoban {
        Sokoban::build(&NodeType::build, level)
    }

    pub fn can_reach(&mut self, position: &Position) -> bool {
        self.get_hash();
        self.player_reachable.as_ref().unwrap()[position.x][position.y] == 1
    }

    pub fn new_reverse(level: &String) -> Sokoban {
        Sokoban::build(&NodeType::reverse_build, level)
    }

    pub fn get_hash(&mut self) -> u64 {
        // This is a quick fix, ideally we want to do one of these 2 things:
        // 1.- Separate the player_reachable state matrix into a history so we can recover it
        // 2.- Make the changes stateless, so we pass a copy of sokoban to the next depth DFS level
        // this way, we remove the need to do undo_move_box
        let mut player_reachable = self.init_player_reachable();
        self.build_player_reachable(&self.player.unwrap(), &mut player_reachable);
        self.player_reachable = Some(player_reachable);
        let mut hasher = DefaultHasher::new();
        Hash::hash_slice(self.player_reachable.as_ref().unwrap(), &mut hasher);
        hasher.finish()
    }

    fn parse_level(level: &str) -> (usize, usize, &str) {
        (
            (&level[0..2]).parse::<usize>().unwrap(),
            (&level[2..4]).parse::<usize>().unwrap(),
            &level[4..],
        )
    }

    pub fn print_level(&self) -> String { 
        let mut level = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let ntype = self.get_ntype(&Position{ x, y }) as usize;
                level.push_str(&ntype.to_string());
            }
        }
        format!("{:02}{:02}{}", self.height, self.width, level)
    }

    fn build(func: &dyn Fn(u32) -> Result<NodeType, &'static str>, level: &str) -> Sokoban {
        let (height, width, level) = Sokoban::parse_level(level);

        let mut map = HashMap::new();
        let mut level = level.chars();
        let mut player: Option<Position> = None;
        let mut boxes = vec![];
        let mut goals = vec![];
        for y in 0..height {
            for x in 0..width {
                let mut node_type = func(level.next().unwrap().to_digit(10).unwrap()).unwrap();
                if node_type.is_player() {
                    player = Some(Position { x, y });
                    if node_type == NodeType::Player {
                        node_type = NodeType::Empty;
                    } else {
                        node_type = NodeType::Whole;
                    }
                }
                if node_type.is_box() {
                    boxes.push(Position { x, y });
                    if node_type == NodeType::Box {
                        node_type = NodeType::Empty;
                    } else {
                        node_type = NodeType::Whole;
                    }
                }
                if node_type.is_whole() {
                    goals.push(Position { x, y });
                }

                map.insert(Position { x, y }, node_type);
            }
        }
        Sokoban {
            height,
            width,
            map,
            player_reachable: None,
            player,
            boxes,
            goals,
        }
    }

    pub fn get_ntype(&self, position: &Position) -> NodeType {
        let ntype = self.map.get(position);
        if ntype.is_none() {
            return NodeType::Wall;
        }

        let mut ntype = *ntype.unwrap();
        if self.boxes.contains(position) {
            if ntype == NodeType::Whole {
                ntype = NodeType::BoxOnWhole;
            } else {
                ntype = NodeType::Box;
            }
        };

        if self.player.is_some() && *position == self.player.unwrap() {
            if ntype == NodeType::Whole {
                ntype = NodeType::PlayerOnWhole;
            } else {
                ntype = NodeType::Player;
            }
        }
        ntype
    }

    fn init_player_reachable(&self) -> Vec<Vec<u8>> {
        let mut reachable = vec![vec![0 as u8; self.height]; self.width];
        for y in 0..self.height {
            for x in 0..self.width {

                let ntype = self.get_ntype(&Position{ x, y });
                if ntype.is_box() {
                    reachable[x][y] = ntype as u8;
                }
            }
        }
        reachable
    }

    fn build_player_reachable(&self, current: &Position, reachable: &mut Vec<Vec<u8>>) {
        // XXX If we can push a box, we can move, this may be buggy, need an edge case unit test!

        //If player has alredy been here, we dont care about getting here again.
        if reachable[current.x][current.y] == 1 {
            return;
        }

        if self.get_ntype(current).can_move() {
            reachable[current.x][current.y] = 1;

            for adjacent in Node::loop_positions(&current).iter() {
                self.build_player_reachable(&adjacent, reachable);
            }
        }
    }

    pub fn is_resolved(&mut self) -> bool {
        let boxes: HashSet<_> = self.boxes.iter().collect();
        let goals: HashSet<_> = self.goals.iter().collect();
        boxes == goals
    }

    pub fn move_box(&mut self, box_index: usize, direction: &Direction) -> bool {
        // debug!("Trying {:?} on box {}", direction, box_index);

        // if !&box_node.ntype.is_box() {
        //     return false;
        // }

        let box_position = self.boxes[box_index];
        let mut box_position = box_position.clone();
        let mut player_position = box_position.clone();

        match direction {
            Direction::Up => {
                if player_position.y < 2 {
                    return false;
                }
                box_position.y -= 1;
                player_position.y -= 2;
            }
            Direction::Down => {
                box_position.y += 1;
                player_position.y += 2;
            }
            Direction::Left => {
                if player_position.x < 2 {
                    return false;
                }
                box_position.x -= 1;
                player_position.x -= 2;
            }
            Direction::Right => {
                box_position.x += 1;
                player_position.x += 2;
            }
        };

        let box_future = self.get_ntype(&box_position);
        let player_future = self.get_ntype(&player_position);
        if box_future.can_move() && player_future.can_move() && self.can_reach(&box_position) {
            self.boxes[box_index] = box_position;
            self.player = Some(player_position);
            debug!(
                "moved box: {} player: {}, {:?} {} {:?}",
                box_position, player_position, direction, &self, &self.boxes
            );
            return true;
        }
        false
    }

    pub fn undo_move_box(&mut self, box_index: usize, direction: &Direction) {
        // println!("Undoing {:?} on box {} {}", direction, box_index, self);
        let mut box_position = self.boxes[box_index];
        self.player = Some(box_position);
        match direction {
            Direction::Up => {
                box_position.y += 1;
            }
            Direction::Down => {
                box_position.y -= 1;
            }
            Direction::Left => {
                box_position.x += 1;
            }
            Direction::Right => {
                box_position.x -= 1;
            }
        };

        self.boxes[box_index] = box_position;
    }
}

impl fmt::Display for Sokoban {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut grid_display = String::new();

        for y in 0..self.height {
            grid_display.push('\n');
            grid_display.push(char::from_digit((y % 10) as u32, 10).unwrap());
            for x in 0..self.width {
                let position = Position { x, y };
                grid_display.push_str(&format!("{}", self.get_ntype(&position)));
                grid_display.push_str(" ");
            }
        }
        write!(f, "{}", grid_display)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_env_logger;

    #[test]
    fn test_build_sokoban_world() {
        let sokoban_level =
            String::from("080711111111200001110320101011011401101113230101000100111110");
        let sokoban = Sokoban::new(&sokoban_level);
        println!("{}", sokoban);
    }

    #[test]
    fn test_reverse_build_sokoban_world() {
        let sokoban_level =
            String::from("080711111111200001110620101011011001101113230101000100111110");
        Sokoban::new_reverse(&sokoban_level);
        // println!("{}", sokoban);
    }

    #[test]
    fn test_no_box_move_if_invalid_move() {}

    #[test]
    fn test_box_move() {}

    #[test]
    fn test_hash_sokoban() {
        let sokoban_level1 = String::from("0506111111120101100101140301111111");
        let sokoban_level2 = String::from("0506111111120101104101100301111111");
        let sokoban_level3 = String::from("0506111111120101100101100341111111");
        let mut sokoban = Sokoban::new(&sokoban_level1);
        let mut sokoban2 = Sokoban::new(&sokoban_level2);
        let mut sokoban3 = Sokoban::new(&sokoban_level3);
        debug!("{}", sokoban);
        debug!("{}", sokoban2);
        debug!("{}", sokoban3);
        assert_eq!(sokoban.get_hash(), sokoban2.get_hash());
        assert_eq!(sokoban.get_hash(), sokoban2.get_hash());
        assert_ne!(sokoban.get_hash(), sokoban3.get_hash());

        //moved box: (5, 5)
        let sokoban_level1 = String::from("08081111111110300001100022011111101100010310000104100001001000011110");
        let sokoban_level2 = String::from("08081111111110300001100022011111141100010310000100100001001000011110");
        let mut sokoban1 = Sokoban::new(&sokoban_level1);
        let mut sokoban2 = Sokoban::new(&sokoban_level2);
        println!("{}", sokoban1);
        println!("{}", sokoban2);
        assert_ne!(sokoban1.get_hash(), sokoban2.get_hash());

        let sokoban_level1 = String::from("07070111100114011110300011325201100011111001000111100");
        let sokoban_level2 = String::from("07070111100110011110340011055201100011111001000111100");
        let mut sokoban1 = Sokoban::new(&sokoban_level1);
        let mut sokoban2 = Sokoban::new(&sokoban_level2);
        sokoban1.get_hash();
        sokoban2.get_hash();
        println!("{:?}", sokoban1.player_reachable);
        println!("{:?}", sokoban2.player_reachable);
        assert_ne!(sokoban1.get_hash(), sokoban2.get_hash());
    }

    #[test]
    fn test_sokoban_is_resolved() {
        let sokoban_level = String::from("0506111111122101133101140001111111");
        let mut sokoban = Sokoban::new(&sokoban_level);
        println!("{}", sokoban);

        assert_eq!(sokoban.is_resolved(), false);

        let sokoban_level = String::from("0506111111155101100101140001111111");
        let mut sokoban = Sokoban::new(&sokoban_level);
        println!("{}", sokoban);
        assert_eq!(sokoban.is_resolved(), true);
    }

    #[test]
    fn test_sokoban_print_level() {
        let sokoban_level = String::from("0506111111122101133101140001111111");
        let sokoban = Sokoban::new(&sokoban_level);
        assert_eq!(sokoban.print_level(), sokoban_level);
    }

}
