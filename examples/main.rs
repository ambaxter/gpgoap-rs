extern crate gpgoap;

use gpgoap::{WorldState, WorldStateFmt, ActionPlanner, AStarPlan, AStar};

fn main() {
    let mut ap = ActionPlanner::new();
    ap.set_pre("scout", "armedwithgun", true);
    ap.set_post("scout", "enemyvisible", true);

    ap.set_pre("approach", "enemyvisible", true);
    ap.set_post("approach", "nearenemy", true);

    ap.set_pre("aim", "enemyvisible", true);
    ap.set_pre("aim", "weaponloaded", true);
    ap.set_post("aim", "enemylinedup", true);

    ap.set_pre("shoot", "enemylinedup", true);
    ap.set_post("shoot", "enemyalive", false);

    ap.set_pre("load", "armedwithgun", true);
    ap.set_post("load", "weaponloaded", true);

    ap.set_pre("detonatebomb", "armedwithbomb", true);
    ap.set_pre("detonatebomb", "nearenemy", true);
    ap.set_post("detonatebomb", "alive", false);
    ap.set_post("detonatebomb", "enemyalive", false);

    ap.set_pre("flee", "enemyvisible", true);
    ap.set_post("flee", "nearenemy", false);


    //ap.set_cost("detonatebomb", 5);

    print!("Planner:\n {:?}", &ap);

    let mut fr = WorldState::new();
    fr.set(&mut ap, "enemyvisible", false);
    fr.set(&mut ap, "armedwithgun", true);
    fr.set(&mut ap, "weaponloaded", false);
    fr.set(&mut ap, "enemylinedup", false);
    fr.set(&mut ap, "enemyalive", true);
    fr.set(&mut ap, "armedwithbomb", false);
    fr.set(&mut ap, "nearenemy", false);
    fr.set(&mut ap, "alive", true);

    print!("From: \n{:?}", fr.debug_fmt(&ap));


    let mut goal = WorldState::new();
    goal.set(&mut ap, "enemyalive", false);
    goal.set(&mut ap, "alive", true);

    //print!("Goal: \n{:?}", goal.debug_fmt(&ap));

    let mut astar = AStar::new();
    if let Some(plan) = astar.plan(&ap, &fr, &goal) {
        println!("Plan Cost: {}", plan.cost());
        for (i, &(plan, result_state)) in plan.iter().enumerate() {
            println!("{}: {}\n{:?}", i, plan, result_state.debug_fmt(&ap));
        }
    }

}