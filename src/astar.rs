use ::goap::{WorldState, ActionPlanner, MAX_ACTIONS, MAX_ATOMS};
use ::bitset::BitSetU64;
use itertools::Itertools;
use itertools::multizip;
use std::collections::vec_deque;

#[derive(PartialEq, Eq, Clone, Copy)]
struct AStarNode {
    ws: WorldState, // State of the world at this node
    parentws: Option<WorldState>, // Where did we come from?
    g: i32, // The cost so far
    h: i32, // The heuristic for the remaining cost
    f: i32, // g+h combined
    action_name :&'static str // How did we get to this node?
}

pub struct AStarPlan {
    entries: vec_deque::VecDeque<(&'static str, WorldState)>,
    cost: i32
}

impl AStarPlan {
    pub fn new() -> Self {
        AStarPlan{entries: Default::default(), cost: 0}
    }

    pub fn iter(&self) -> vec_deque::Iter<(&'static str, WorldState)> {
        self.entries.iter()
    }

    pub fn cost(&self) -> i32 {
        self.cost
    }
}

const MAX_OPEN: usize = 1024;
const MAX_CLOSED: usize = 1024;

pub struct AStar {
    opened: Vec<AStarNode>,
    closed: Vec<AStarNode>
}

impl AStar {
    pub fn new() -> Self {
        AStar {
            opened: Vec::with_capacity(MAX_OPEN),
            closed: Vec::with_capacity(MAX_CLOSED)
        }
    }

    fn idx_in_opened(&self, ws: &WorldState) -> Option<usize> {
        self.opened.iter().position(|o| o.ws.values == ws.values)
    }

    fn idx_in_closed(&self, ws: &WorldState) -> Option<usize> {
        self.closed.iter().position(|o| o.ws.values == ws.values)
    }

    fn entry_in_closed(&self, ws: &WorldState) -> Option<&AStarNode> {
        self.closed.iter().filter(|c| c.ws.values == ws.values).nth(0)
    }

    fn calc_heuristic(from: &WorldState, to: &WorldState) -> i32 {
        let care = to.dontcare ^ BitSetU64::full();
        let diff = (from.values & care) ^ (to.values & care);
        let dist = diff.count_ones();
        dist as i32
    }

    fn clear(&mut self) {
        self.opened.clear();
        self.closed.clear();
    }

    fn reconstruct_plan(&self, ap: &ActionPlanner, goal_node: &AStarNode ) -> AStarPlan {
        let mut plan = AStarPlan::new();
        plan.cost = goal_node.f;
        let mut current_node = Some(goal_node);

        while let Some(node) = current_node {
            println!("{}", node.action_name);
            plan.entries.push_front((node.action_name, node.ws));
            if let Some(parentws) = node.parentws {
                let parentws = node.parentws.as_ref().unwrap();
                println!("idx_in_closed: {:?}", self.idx_in_closed(parentws));
                current_node = self.entry_in_closed(parentws);
            } else {
                current_node = None;
            }
        }
        plan
    }

    /* from: http://theory.stanford.edu/~amitp/GameProgramming/ImplementationNotes.html
    OPEN = priority queue containing START
    CLOSED = empty set
    while lowest rank in OPEN is not the GOAL:
      current = remove lowest rank item from OPEN
      add current to CLOSED
      for neighbors of current:
        cost = g(current) + movementcost(current, neighbor)
        if neighbor in OPEN and cost less than g(neighbor):
          remove neighbor from OPEN, because new path is better
        if neighbor in CLOSED and cost less than g(neighbor): **
          remove neighbor from CLOSED
        if neighbor not in OPEN and neighbor not in CLOSED:
          set g(neighbor) to cost
          add neighbor to OPEN
          set priority queue rank to g(neighbor) + h(neighbor)
          set neighbor's parent to current
     */


    pub fn plan(&mut self, ap: &ActionPlanner, start: &WorldState, goal: &WorldState) -> Option<AStarPlan> {
        self.clear();
        let mut plan = AStarPlan::new();
        let h = Self::calc_heuristic(start, goal);
        let n0 = AStarNode{
            ws: *start,
            parentws: None,
            g: 0,
            h: h,
            f: 0 + h,
            action_name: "root"
        };
        self.opened.push(n0);

        loop {
            if self.opened.len() == 0 {
                println!("Did not find a path.");
                return None;
            }
            let (lowest_idx, _) = self.opened.iter()
                    .enumerate()
                    .min_by_key(|&(_, node)| node.f)
                    .unwrap();
            let cur = match self.opened.len() {
                1 => self.opened.first().unwrap().clone(),
                _ => self.opened.swap_remove(lowest_idx)
            };

            let care = goal.dontcare ^ BitSetU64::full();
            let val_match = cur.ws.values & care == goal.values & care;
            if val_match {
                println!("Reconstructing plan!");
                return Some(self.reconstruct_plan(ap, &cur));
            }
            self.closed.push(cur);
            if self.closed.len() == MAX_CLOSED {
                println!("Closed set overflow");
                return None;
            }

            for(name, act_cost, to_ws) in StateTransIter::new(&cur.ws, multizip(ap.actions())) {
                let cost = cur.g + act_cost;
                let idx_o = match self.idx_in_opened(&to_ws) {
                    Some(idx) if cost < self.opened.get(idx).unwrap().g => {
                        if self.opened.len() > 1 {
                            self.opened.swap_remove(idx);
                        } else {
                            self.opened.clear();
                        }
                        None
                    },
                    Some(idx) => Some(idx),
                    None => None
                };
                let idx_c = match self.idx_in_closed(&to_ws) {
                    Some(idx) if cost < self.closed.get(idx).unwrap().g => {
                        if self.closed.len() > 1 {
                            self.closed.swap_remove(idx);
                        } else {
                            self.closed.clear();
                        }
                        None
                    },
                    Some(idx) => Some(idx),
                    None => None
                };
                if idx_o.is_none() && idx_c.is_none() {
                    let g = cost;
                    let h = Self::calc_heuristic(&to_ws, goal);
                    let f = g + h;
                    let nb = AStarNode {
                        ws: to_ws,
                        g: g,
                        h: h,
                        f: f,
                        action_name: name,
                        parentws: Some(cur.ws)
                    };
                    self.opened.push(nb);
                }
                if self.opened.len() == MAX_OPEN {
                    println!("Opened set overflow!");
                    return None;
                }
            }
        }
        Some(plan)

    }


}

struct StateTransIter<'a, 'b, I>
    where I: Iterator<Item = (&'b&'static str, &'b WorldState, &'b i32, &'b WorldState)> {
    from: &'a WorldState,
    action_iter: I
}

impl<'a, 'b, I> StateTransIter<'a, 'b, I>
    where I: Iterator<Item = (&'b&'static str, &'b WorldState, &'b i32, &'b WorldState)> {

    pub fn new(from: &'a WorldState, action_iter: I) -> StateTransIter<'a, 'b, I> {
        StateTransIter{from: from, action_iter: action_iter}
    }
}

impl<'a, 'b, I> Iterator for StateTransIter<'a, 'b, I>
    where I: Iterator<Item = (&'b&'static str, &'b WorldState, &'b i32, &'b WorldState)> {
    type Item = (&'static str, i32, WorldState);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((name, pre, cost, post)) = self.action_iter.next() {
            let care = pre.dontcare ^ BitSetU64::full();
            let met = (pre.values & care) == (self.from.values & care);
            if met {
                let unaffected = post.dontcare;
                let affected = unaffected ^ BitSetU64::full();
                let mut next = *self.from;
                next.values = (self.from.values & unaffected) | (post.values & affected);
                next.dontcare &= unaffected;
                return Some((*name, *cost, next));
            }
        }
        None
    }
}