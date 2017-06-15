use itertools::Itertools;
use itertools::multizip;

use bitset::BitSetU64;
use std::ops::IndexMut;
use std::fmt;

pub const MAX_ATOMS: usize = 64;
pub const MAX_ACTIONS: usize = 64;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct WorldState
{
    pub values: BitSetU64,
    pub dontcare: BitSetU64
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            values: BitSetU64::empty(),
            dontcare: BitSetU64::full()
        }
    }

    pub fn clear(&mut self) {
        *self = WorldState::new();
    }


    pub fn set(&mut self, ap: &mut ActionPlanner, atom_name: &'static str, value: bool) -> bool {
        match ap.idx_for_atom_name(atom_name) {
            Some(idx) => self.set_idx(idx, value),
            _ => false
        }
    }

    fn set_idx(&mut self, idx: usize, value: bool) -> bool {
        self.values.set(idx, value);
        self.dontcare.disable(idx);
        true
    }

    pub fn debug_fmt<'a, 'b>(&'a self, ap: &'b ActionPlanner) -> WorldStateFmt<'a, 'b>  {
        WorldStateFmt{ws: self, ap: ap}
    }
}

impl Default for WorldState {
    fn default() -> Self {
        WorldState::new()
    }
}

pub struct WorldStateFmt<'a, 'b> {
    ws: &'a WorldState,
    ap: &'b ActionPlanner
}

impl<'a, 'b> fmt::Debug for WorldStateFmt<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..MAX_ATOMS {
            if !self.ws.dontcare.get(i) {
                let string = self.ap.atm_names.get(i).unwrap();
                if self.ws.values.get(i) {
                    write!(f, "+{}\n", string).unwrap();
                } else {
                    write!(f, "-{}\n", string).unwrap();
                }
            }
        }
        write!(f, "\n")
    }
}

pub struct ActionPlanner {
    atm_names: [&'static str; MAX_ATOMS],
    num_atoms: usize,

    act_names: [&'static str; MAX_ACTIONS],
    act_pre: [WorldState; MAX_ACTIONS],
    act_post: [WorldState; MAX_ACTIONS],
    act_costs: [i32; MAX_ACTIONS],
    num_actions: usize,
}


impl ActionPlanner {
    pub fn new() -> Self {
        ActionPlanner {
            atm_names: [Default::default(); MAX_ATOMS],
            num_atoms: 0,
            act_names: [Default::default(); MAX_ACTIONS],
            act_pre: [Default::default(); MAX_ACTIONS],
            act_post: [Default::default(); MAX_ACTIONS],
            act_costs: [0; MAX_ACTIONS],
            num_actions: 0,
        }
    }

    pub fn clear(&mut self) {
        *self = ActionPlanner::new();
    }

    pub fn idx_for_atom_name(&mut self, atom_name: &'static str) -> Option<usize> {
        let find_index = self.atm_names[0..self.num_atoms].iter()
            .position(|s| *s == atom_name);
        match find_index {
            Some(_) => find_index,
            None if self.num_atoms < MAX_ATOMS => {
                let idx = self.num_atoms;
                *self.atm_names.index_mut(self.num_atoms) = atom_name;
                self.num_atoms += 1;
                Some(idx)
            },
            None => None
        }
    }

    pub fn idx_for_action_name(&mut self, act_name: &'static str) -> Option<usize> {
        let find_index = self.act_names[0..self.num_actions].iter()
            .position(|s| *s == act_name);
        match find_index {
            Some(_) => find_index,
            None if self.num_actions < MAX_ACTIONS => {
                let idx = self.num_actions;
                *self.act_names.index_mut(idx) = act_name;
                *self.act_costs.index_mut(idx) = 1;
                self.num_actions += 1;
                Some(idx)
            },
            None => None
        }
    }

    pub fn set_pre(&mut self, action_name: &'static str, atom_name: &'static str, value: bool) -> bool {
        match (self.idx_for_action_name(action_name), self.idx_for_atom_name(atom_name)) {
            (Some(actidx), Some(atmidx)) => self.act_pre.index_mut(actidx).set_idx(atmidx, value),
            _ => false
        }
    }

    pub fn set_post(&mut self, action_name: &'static str, atom_name: &'static str, value: bool) -> bool {
        match (self.idx_for_action_name(action_name), self.idx_for_atom_name(atom_name)) {
            (Some(actidx), Some(atmidx)) => self.act_post.index_mut(actidx).set_idx(atmidx, value),
            _ => false
        }
    }

    pub fn set_cost(&mut self, action_name: &'static str, cost: i32) -> bool {
        if let Some(idx) = self.idx_for_action_name(action_name) {
            *self.act_costs.index_mut(idx) = cost;
            true
        } else {
            false
        }
    }

    pub fn act_pre(&self) -> &[WorldState] {
        &self.act_pre[..self.num_actions]
    }

    pub fn name(&self, i: usize) -> &'static str {
        self.act_names[..self.num_actions].get(i).unwrap()
    }

    pub fn cost(&self, i: usize) -> i32 {
        *self.act_costs[..self.num_actions].get(i).unwrap()
    }

    pub fn post(&self, i: usize) -> &WorldState {
        self.act_post[..self.num_actions].get(i).unwrap()
    }

    pub fn actions(&self) -> (&[&'static str], &[WorldState], &[i32], &[WorldState]) {
        (&self.act_names[..self.num_actions],
         &self.act_pre[..self.num_actions],
         &self.act_costs[..self.num_actions],
         &self.act_post[..self.num_actions]
        )
    }

}

impl Default for ActionPlanner {
    fn default() -> Self {
        ActionPlanner::new()
    }
}

impl fmt::Debug for ActionPlanner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        for (name, pre, cost, post) in multizip(self.actions()) {
            write!(f, "{} - {}\n", name, cost).unwrap();
            for i in 0..MAX_ATOMS {
                if !pre.dontcare.get(i) {
                    write!(f, "  {}=={}\n", self.atm_names.get(i).unwrap(), pre.values.get(i)).unwrap();
                }
            }
            for i in 0..MAX_ATOMS {
                if !post.dontcare.get(i) {
                    write!(f, "  {}:={}\n", self.atm_names.get(i).unwrap(), post.values.get(i)).unwrap();

                }
            }
        }
        write!(f, "\n")
    }
}


#[cfg(test)]
mod tests {

    use super::ActionPlanner;

    #[test]
    fn it_works() {
        let mut planner = ActionPlanner::new();
        assert_eq!(Some(0), planner.idx_for_atom_name("atom_0"));
        assert_eq!(Some(1), planner.idx_for_atom_name("atom_1"));
        assert_eq!(Some(2), planner.idx_for_atom_name("atom_2"));
        assert_eq!(Some(3), planner.idx_for_atom_name("atom_3"));
        assert_eq!(Some(4), planner.idx_for_atom_name("atom_4"));
        assert_eq!(Some(5), planner.idx_for_atom_name("atom_5"));
        assert_eq!(Some(6), planner.idx_for_atom_name("atom_6"));
        assert_eq!(Some(7), planner.idx_for_atom_name("atom_7"));
        assert_eq!(Some(8), planner.idx_for_atom_name("atom_8"));
        assert_eq!(Some(9), planner.idx_for_atom_name("atom_9"));
        assert_eq!(Some(10), planner.idx_for_atom_name("atom_10"));
        assert_eq!(Some(11), planner.idx_for_atom_name("atom_11"));
        assert_eq!(Some(12), planner.idx_for_atom_name("atom_12"));
        assert_eq!(Some(13), planner.idx_for_atom_name("atom_13"));
        assert_eq!(Some(14), planner.idx_for_atom_name("atom_14"));
        assert_eq!(Some(15), planner.idx_for_atom_name("atom_15"));
        assert_eq!(Some(16), planner.idx_for_atom_name("atom_16"));
        assert_eq!(Some(17), planner.idx_for_atom_name("atom_17"));
        assert_eq!(Some(18), planner.idx_for_atom_name("atom_18"));
        assert_eq!(Some(19), planner.idx_for_atom_name("atom_19"));
        assert_eq!(Some(20), planner.idx_for_atom_name("atom_20"));
        assert_eq!(Some(21), planner.idx_for_atom_name("atom_21"));
        assert_eq!(Some(22), planner.idx_for_atom_name("atom_22"));
        assert_eq!(Some(23), planner.idx_for_atom_name("atom_23"));
        assert_eq!(Some(24), planner.idx_for_atom_name("atom_24"));
        assert_eq!(Some(25), planner.idx_for_atom_name("atom_25"));
        assert_eq!(Some(26), planner.idx_for_atom_name("atom_26"));
        assert_eq!(Some(27), planner.idx_for_atom_name("atom_27"));
        assert_eq!(Some(28), planner.idx_for_atom_name("atom_28"));
        assert_eq!(Some(29), planner.idx_for_atom_name("atom_29"));
        assert_eq!(Some(30), planner.idx_for_atom_name("atom_30"));
        assert_eq!(Some(31), planner.idx_for_atom_name("atom_31"));
        assert_eq!(Some(32), planner.idx_for_atom_name("atom_32"));
        assert_eq!(Some(33), planner.idx_for_atom_name("atom_33"));
        assert_eq!(Some(34), planner.idx_for_atom_name("atom_34"));
        assert_eq!(Some(35), planner.idx_for_atom_name("atom_35"));
        assert_eq!(Some(36), planner.idx_for_atom_name("atom_36"));
        assert_eq!(Some(37), planner.idx_for_atom_name("atom_37"));
        assert_eq!(Some(38), planner.idx_for_atom_name("atom_38"));
        assert_eq!(Some(39), planner.idx_for_atom_name("atom_39"));
        assert_eq!(Some(40), planner.idx_for_atom_name("atom_40"));
        assert_eq!(Some(41), planner.idx_for_atom_name("atom_41"));
        assert_eq!(Some(42), planner.idx_for_atom_name("atom_42"));
        assert_eq!(Some(43), planner.idx_for_atom_name("atom_43"));
        assert_eq!(Some(44), planner.idx_for_atom_name("atom_44"));
        assert_eq!(Some(45), planner.idx_for_atom_name("atom_45"));
        assert_eq!(Some(46), planner.idx_for_atom_name("atom_46"));
        assert_eq!(Some(47), planner.idx_for_atom_name("atom_47"));
        assert_eq!(Some(48), planner.idx_for_atom_name("atom_48"));
        assert_eq!(Some(49), planner.idx_for_atom_name("atom_49"));
        assert_eq!(Some(50), planner.idx_for_atom_name("atom_50"));
        assert_eq!(Some(51), planner.idx_for_atom_name("atom_51"));
        assert_eq!(Some(52), planner.idx_for_atom_name("atom_52"));
        assert_eq!(Some(53), planner.idx_for_atom_name("atom_53"));
        assert_eq!(Some(54), planner.idx_for_atom_name("atom_54"));
        assert_eq!(Some(55), planner.idx_for_atom_name("atom_55"));
        assert_eq!(Some(56), planner.idx_for_atom_name("atom_56"));
        assert_eq!(Some(57), planner.idx_for_atom_name("atom_57"));
        assert_eq!(Some(58), planner.idx_for_atom_name("atom_58"));
        assert_eq!(Some(59), planner.idx_for_atom_name("atom_59"));
        assert_eq!(Some(60), planner.idx_for_atom_name("atom_60"));
        assert_eq!(Some(61), planner.idx_for_atom_name("atom_61"));
        assert_eq!(Some(62), planner.idx_for_atom_name("atom_62"));
        assert_eq!(Some(63), planner.idx_for_atom_name("atom_63"));
        assert_eq!(None, planner.idx_for_atom_name("atom_64"));
        assert_eq!(None, planner.idx_for_atom_name("atom_65"));
        assert_eq!(None, planner.idx_for_atom_name("atom_66"));
        assert_eq!(None, planner.idx_for_atom_name("atom_67"));
        assert_eq!(None, planner.idx_for_atom_name("atom_68"));
        assert_eq!(None, planner.idx_for_atom_name("atom_69"));


        assert_eq!(Some(0), planner.idx_for_atom_name("atom_0"));
    }
}