#![allow(dead_code)]
use crate::node::{Node, NodeType, Position};
use crate::sokoban::{Direction, Sokoban};
use log::debug;
use std::collections::HashMap;
use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

pub struct Solver {
    heuristics: HashMap<usize, HashMap<Position, usize>>,
    state_map: HashMap<u64, usize>,
    pub sokoban: Sokoban,
    original_player: Position,
    pub counter: usize,
}

impl Solver {
    pub fn new(level: String) -> Solver {
        let sokoban = Sokoban::new(&level);
        Solver {
            heuristics: Solver::build_heuristics(&sokoban),
            state_map: HashMap::new(),
            sokoban: Sokoban::new_reverse(&level),
            original_player: sokoban.player.unwrap(),
            counter: 0,
        }
    }

    fn get_heuristic(&self, goal_index: usize, box_index: usize) -> Option<&usize> {
        let current_goal = self.sokoban.goals[goal_index];
        self.heuristics[&box_index].get(&current_goal)
    }

    fn been_here(&mut self, depth: usize) -> bool {
        let current_hash = self.sokoban.get_hash();
        // debug!("been here? {} {} c:{}\n {:?}", depth, current_hash, self.state_map.contains_key(&current_hash), self.state_map);
        if self.state_map.contains_key(&current_hash) && depth >= self.state_map[&current_hash] {
            // debug!("been here!");
            return true;
        }
        self.state_map.insert(current_hash, depth);
        false
    }

    fn build_heuristics(sokoban_map: &Sokoban) -> HashMap<usize, HashMap<Position, usize>> {
        let mut heuristics = HashMap::new();
        for sbox in 0..sokoban_map.boxes.len() {
            heuristics.insert(
                sbox,
                Solver::heuristic_bfs(&sokoban_map, sokoban_map.boxes[sbox]),
            );
        }
        heuristics
    }

    /*
     * Gets a count of steps required to get to a box from any point in the
     * sokoban table by just taking into account walls, not other boxes. i.e.:
     *
     *  ######
     *  #12#6#
     *  #x1#5#
     *  #1234#
     *  ######
     */
    fn heuristic_bfs(sokoban_map: &Sokoban, goal: Position) -> HashMap<Position, usize> {
        let mut state = Vec::new();
        let mut queue = Vec::new();
        let mut distance = HashMap::new();

        queue.insert(0, goal);
        state.push(goal);
        distance.insert(goal, 0);
        while let Some(current) = queue.pop() {
            for adjacent in Node::loop_positions(&current).iter() {
                if state.contains(adjacent) {
                    continue;
                }

                state.push(*adjacent);
                if sokoban_map.get_ntype(&adjacent) != NodeType::Wall {
                    distance.insert(*adjacent, distance.get(&current).unwrap() + 1);
                    queue.insert(0, *adjacent);
                }
            }
        }

        distance
    }

    /*
     * A function to get the player on different zones to try to solve the world
     *
     * # # # # # #
     * #  @  # @ # two zones where the player could start
     * #     $   #
     * # # # # # #
     */
    fn player_zones(&self) -> Vec<Position> {
        let mut player_zones: Vec<Position> = Vec::new();
        let mut unvisited: Vec<Position> = self.sokoban.map.iter().map(|(key, _)| *key).collect();
        let mut queue = Vec::new();
        for pos in self.sokoban.map.keys() {
            if self.sokoban.get_ntype(pos) == NodeType::Empty {
                queue.insert(0, *pos);
                break;
            }
        }
        debug!("{}", self.sokoban);
        loop {
            // check if there is a box within reach, else, we are outside of the world
            let mut has_box = false;
            let mut last = None;
            while let Some(current) = queue.pop() {
                unvisited.remove_item(&current);
                for adjacent in Node::loop_positions(&current).iter() {
                    let node_type = self.sokoban.get_ntype(&adjacent);
                    if node_type.is_box() {
                        debug!("Found box {} {}", adjacent, node_type);
                        has_box = true;
                    }
                    if !unvisited.contains(adjacent) {
                        continue;
                    }

                    unvisited.remove_item(adjacent);
                    if node_type.can_move() {
                        debug!("inserting in queue {} with ntype {}", adjacent, node_type);
                        queue.insert(0, *adjacent);
                    }
                }
                last = Some(current);
            }

            if has_box && last.is_some() {
                player_zones.push(last.unwrap());
                debug!("Found player {}", last.unwrap());
            }

            if unvisited.is_empty() {
                break;
            }

            for pos in unvisited.iter() {
                if self.sokoban.get_ntype(&pos).can_move() {
                    queue.push(*pos);
                    break;
                }
            }

            if queue.is_empty() {
                break;
            }
        }

        player_zones
    }

    pub fn solve_sokoban(&mut self) -> bool {
        let solved = false;
        for player in self.player_zones().iter() {
            debug!("Trying player {}", player);
            self.sokoban.player = Some(player.clone());
            let solved = self.solve_dfs(0, 0, 0, &Direction::Up, usize::max_value(), 0);
            if solved {
                return solved;
            }
        }
        solved
    }

    fn generator_test(
        match_length: u8,
        previous_goal_index: u8,
        previous_box_index: u8,
        previous_direction: &Direction,
    ) {
        let mut depth_permutation = || {
            let mut current_goal_index;
            for i in 0..match_length {
                current_goal_index = (previous_goal_index + i) % match_length;
                let mut current_box_index;
                for j in 0..match_length {
                    current_box_index = (previous_box_index + j) % match_length;

                    let mut current_direction = previous_direction.clone();
                    for _dir in 0u8..4 {
                        yield (
                            current_goal_index,
                            current_box_index,
                            current_direction.clone(),
                        );
                        current_direction = current_direction.next().unwrap();
                    }
                }
            }
        };

        while let GeneratorState::Yielded((goal_index, box_index, direction)) =
            Pin::new(&mut depth_permutation).resume()
        {
            println!(
                "goal {}, box {}, direction {:?}",
                goal_index, box_index, direction
            );
        }
    }

    fn is_solved(&mut self) -> bool {
        self.sokoban.is_resolved() && self.sokoban.can_reach(&self.original_player)
    }

    fn solve_dfs(
        &mut self,
        start_cost: usize,
        goal_index: usize,
        box_index: usize,
        previous_direction: &Direction,
        cost_limit: usize,
        depth: usize,
    ) -> bool {
        if self.is_solved() {
            return true;
        }

        if self.been_here(depth) {
            return false;
        }

        // box and goal length are the same, so we can use the match_length for both
        let match_length = self.sokoban.boxes.len();
        let mut minimum_cost;
        let mut current_box_index;
        for j in 0..match_length {
            current_box_index = (box_index + j) % match_length;
            let mut current_goal_index;

            for i in 0..match_length {
                // second loop over goals
                current_goal_index = (goal_index + i) % match_length;

                let mut current_direction = previous_direction.clone();
                let mut is_blocked = true;
                for _dir in 0..4 {
                    // Avoid trying to go to a goal where a box is set
                    // check if minimum cost is greater than cost limit
                    let heuristic_value = self.get_heuristic(current_goal_index, current_box_index);
                    let heuristic_value = match heuristic_value {
                        Some(value) => value,
                        None => continue,
                    };
                    minimum_cost = start_cost + heuristic_value;
                    if minimum_cost > cost_limit {
                        break;
                    }

                    // try to move box, if we can, count, and issue DFS again
                    if self.sokoban.move_box(current_box_index, &current_direction) {
                        is_blocked = false;
                        self.counter += 1;
                        // debug!("depth: {}, state: {}\n {}", depth, self.sokoban.get_hash(), self.sokoban.print_level());
                        let solved = self.solve_dfs(
                            start_cost + 1,
                            current_goal_index,
                            current_box_index,
                            &current_direction,
                            cost_limit,
                            depth + 1,
                        );

                        self.sokoban
                            .undo_move_box(current_box_index, &current_direction);
                        // debug!("counter: {}, {}", self.counter, &self.sokoban);

                        if solved == true {
                            // add path solution
                            // add box swaps
                            return true;
                        }
                    }

                    current_direction = current_direction.next().unwrap();
                }

                if is_blocked {
                    // check if the box is in a whole
                    // check if it wasn't able to move because of walls, compared to boxes that
                    // could be moved in the future.
                    //
                    // If can't move because of boxes, break the current loop
                    // If can't move because of walls, cut the options tree entirely by returning false
                    if self.should_cut_tree(box_index) {
                        return false;
                    }

                    break;
                }
            }
        }

        false
    }

    /*
     * in case of a blockage, i.e. a box can't be moved
     * check the reason why.
     *
     * - If box can't move because there is another box blocking the way, we should keep looking that tree
     * - If box can't move but it's on a goal, keep looking that tree
     * - If box can't move because there are walls, stop looking that tree.
     */
    fn should_cut_tree(&self, box_index: usize) -> bool {
        let box_position = self.sokoban.boxes[box_index];
        let box_ntype = self.sokoban.get_ntype(&box_position);

        if box_ntype == NodeType::BoxOnWhole {
            return false;
        }
        
        let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        for dir in directions.iter() {
            let future_result = self.sokoban.get_future_position(&box_position, &dir);
            if future_result.err().is_some() {
                continue;
            }

            let (box_future, player_future) = future_result.ok().unwrap();
            let box_future = self.sokoban.get_ntype(&box_future);
            let player_future = self.sokoban.get_ntype(&player_future);
            if player_future == NodeType::Wall || box_future == NodeType::Wall {
                continue
            }

            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_env_logger;

    #[test]
    fn test_build_heuristics() {
        let sokoban_level = String::from("0506111111120101130101140001111111");
        let solver = Solver::new(sokoban_level);
        println!("{}", solver.sokoban);
        println!("{:?}", solver.heuristics);
    }

    #[test]
    fn test_sokoban_with_multiple_player_zones() {
        pretty_env_logger::init();
        let sokoban_level =
            String::from("080711111111200601110020101011011001101113230101020100111110");
        let solver = Solver::new(sokoban_level);

        assert_eq!(solver.player_zones().len(), 2);

        let sokoban_level = String::from("0706111110104010122210133311100001100001111111");
        let solver = Solver::new(sokoban_level);

        assert_eq!(solver.player_zones().len(), 2);
    }

    #[test]
    fn test_sokoban_solver() {
        let sokoban_level = String::from("0706111100102100100111154001100301100111111100");
        let mut solver = Solver::new(sokoban_level);
        assert_eq!(solver.solve_sokoban(), true);
    }
}
