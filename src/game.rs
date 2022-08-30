use std::collections::VecDeque;

use yewdux::prelude::*;

// #[derive(Default, Clone, PartialEq, Eq, Store)]
#[derive(Clone, PartialEq, Store)]
pub struct State {
    tick_interval_ms: u16,
    pub tick: Tick,
    pub grid: Grid,
    spawn_rates: SpawnRates,
}

type Tick = u32;

#[derive(Default, Clone, PartialEq)]
pub struct Grid([Option<Lane>; 5]);

#[derive(Default, Clone, PartialEq)]
pub struct Lane([Field; 9]);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Field(pub VecDeque<Entity>);

#[derive(Debug, Clone, PartialEq)]
pub enum Entity {
    Zombie(LastMovedTick),
    Turret,
    Bullet(LastMovedTick),
    Collision,
}

type LastMovedTick = Tick;

#[derive(Clone, PartialEq)]
struct SpawnRates {
    zombies_each_x_ticks: Tick,
    bullets_each_x_ticks: Tick,
}

impl Entity {
    //todo: maybe should be moved to Lane? (as Entity should probabbly not know about the lane logic and fields...)
    fn move_zombie(
        entity: Entity,
        current_tick: Tick,
        mut current_field: Field,
        mut opt_prev_field: Option<Field>,
    ) -> (Field, Option<Field>) {
        match opt_prev_field {
            None => {
                /* prev is end of lane */
                //todo:
                //end of lane, hit player
                current_field.pop_front();
                return (current_field, None);
            }
            Some(Field(mut prev_field)) => {
                match prev_field.front() {
                    None | Some(Entity::Collision) => {
                        /* prev is empty */
                        current_field.pop_front();
                        prev_field.push_back(Entity::Zombie(current_tick));
                        return (current_field, Some(Field(prev_field)));
                    }
                    Some(Entity::Bullet(_)) => {
                        /* zombie walks into the bullet */
                        current_field.pop_front();
                        prev_field.pop_front();
                        prev_field.push_front(Entity::Collision);
                        return (current_field, Some(Field(prev_field)));
                    }
                    Some(Entity::Turret) => {
                        /* zombie walks over turret */
                        current_field.pop_front();
                        prev_field.pop_front();
                        prev_field.push_back(Entity::Zombie(current_tick));
                        return (current_field, Some(Field(prev_field)));
                    }
                    _ => {
                        //todo:
                        return (current_field, Some(Field(prev_field)));
                    }
                }
            }
        }
    }
}

// inital state
impl std::default::Default for State {
    fn default() -> Self {
        // State::new()
        State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 1,
                bullets_each_x_ticks: 2,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::from([Entity::Turret])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    // Field(VecDeque::new()),
                    // Field(VecDeque::new()),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                ])),
                None,
                None,
            ]),
        }
    }
}

impl State {
    pub fn new() -> State {
        State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 1,
                bullets_each_x_ticks: 1,
            },
            // lane: Lane(Default::default()), //needs #[derive(Default)] on Field
            // lane: Lane(std::array::from_fn(|_| Field(VecDeque::new()))), //creates an array by repeatedly calling the closure

            // grid: Grid(Default::default()),
            // grid: Grid(std::array::from_fn(|_| { Some(Lane(Default::default())) } )),
            grid: Grid([None, None, Some(Lane(Default::default())), None, None]),
        }
    }

    pub fn next(&mut self) {
        self.tick += 1;
        self.grid = State::grid_next(self.grid.clone(), self.tick, &self.spawn_rates);
    }

    fn grid_next(mut grid: Grid, tick: u32, spawn_rates: &SpawnRates) -> Grid {
        for i in 0..grid.len() {
            let opt_lane = grid[i].clone();
            match opt_lane {
                Some(lane) => {
                    grid[i] = Some(State::lane_next(lane, tick, &spawn_rates));
                }
                None => {}
            }
        }

        //handle zombie spawn
        for i in 0..grid.len() {
            let opt_lane = grid[i].clone();
            match opt_lane {
                Some(mut lane) => {
                    let i_last = lane.len() -1;

                    if spawn_rates.zombies_each_x_ticks != 0 &&
                        ((tick % spawn_rates.zombies_each_x_ticks) == 0)
                    {
                        match lane.last() {
                            Some(Field(field)) => {
                                match field.front() {
                                    Some(Entity::Bullet(_)) => {
                                        let mut f_copy = field.clone();
                                        f_copy.pop_front();
                                        lane[i_last] = Field(f_copy);
                                        continue;
                                    }
                                    _ => {
                                        lane[i_last].push_back(Entity::Zombie(tick));
                                        grid[i] = Some(lane);
                                        continue;
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                }
                None => {}
            }
        }

        grid
    }

    fn lane_next(mut lane: Lane, tick: u32, spawn_rates: &SpawnRates) -> Lane {
        lane = State::remove_lane_collisions(lane);

        for i in 0..lane.len() {
            let Field(mut current_field) = lane[i].clone();
            let opt_prev_field = if i == 0 {
                None
            } else {
                Some(lane[i - 1].clone())
            };
            let i_next = i + 1;
            let opt_next_field = if i_next >= lane.len() {
                None
            } else {
                Some(lane[i_next].clone())
            };

            match current_field.front() {
                None => {
                    continue;
                }
                Some(Entity::Zombie(last_moved_tick)) => {
                    let (z_current, z_prev) = Entity::move_zombie(
                        Entity::Zombie(*last_moved_tick),
                        tick,
                        Field(current_field),
                        opt_prev_field,
                    );

                    lane[i] = z_current;
                    if i > 0 && z_prev != None {
                        lane[i - 1] = z_prev.unwrap();
                    }
                }
                Some(Entity::Bullet(last_moved_tick)) => {
                    let (b_current, b_next) = Lane::move_bullet(
                        *last_moved_tick,
                        tick,
                        Field(current_field),
                        opt_next_field,
                    );

                    lane[i] = b_current;
                    if i_next < lane.len() && b_next != None {
                        lane[i_next] = b_next.unwrap();
                    }
                }
                Some(Entity::Turret) => {
                    let (t_current, t_next) = Lane::process_turret(
                        tick,
                        spawn_rates,
                        Field(current_field),
                        opt_next_field,
                    );

                    lane[i] = t_current;
                    if i_next < lane.len() && t_next != None {
                        lane[i_next] = t_next.unwrap();
                    }
                }
                _ => {}
            }
        }

        lane
    }

    fn remove_lane_collisions(mut lane: Lane) -> Lane {
        for i in 0..lane.len() {
            let mut field = (&lane[i]).clone();

            match field.front() {
                Some(Entity::Collision) => {
                    field.pop_front();
                    lane[i] = field;
                }
                _ => {}
            }

        }

        return lane;
    }
}

impl Lane {
    fn move_bullet(
        entity_last_moved_tick: Tick,
        current_tick: Tick,
        mut current_field: Field,
        mut opt_next_field: Option<Field>,
    ) -> (Field, Option<Field>) {
        if entity_last_moved_tick == current_tick {
            return (current_field, opt_next_field);
        }

        match opt_next_field.clone() {
            Some(Field(mut next_field)) => {
                match next_field.front() {
                    None | Some(Entity::Bullet(_)) => {
                        current_field.pop_front();
                        next_field.push_back(Entity::Bullet(current_tick));

                        return (current_field, Some(Field(next_field)));
                    }
                    Some(Entity::Zombie(_)) => {
                        /* hit zombie */
                        current_field.pop_front();
                        next_field.pop_front();
                        next_field.push_front(Entity::Collision);
                        return (current_field, Some(Field(next_field)));
                    }
                    _ => {
                        //todo
                        (current_field, opt_next_field)
                    }
                }
            }
            None => {
                current_field.pop_front();
                return (current_field, opt_next_field);
            }
        }
    }

    fn process_turret(
        current_tick: Tick,
        spawn_rates: &SpawnRates,
        mut current_field: Field,
        mut opt_next_field: Option<Field>,
    ) -> (Field, Option<Field>) {
        if spawn_rates.bullets_each_x_ticks == 0 ||
            current_tick % spawn_rates.bullets_each_x_ticks != 0
        {
            return (current_field, opt_next_field);
        }

        match opt_next_field {
            None => {
                /* end of lane, do nothing? */
                return (current_field, opt_next_field);
            }
            Some(mut next_field) => {
                match next_field.front() {
                    None => {
                        /* next is empty */
                        next_field.push_back(Entity::Bullet(current_tick));
                        return (current_field, Some(next_field));
                    }
                    Some(Entity::Zombie(_)) => {
                        next_field.pop_front();
                        next_field.push_front(Entity::Collision);
                        return (current_field, Some(next_field));
                    }
                    _ => {
                        next_field.push_back(Entity::Bullet(current_tick));
                        return (current_field, Some(next_field));
                    }
                }
            }
        }
    }
}

// impl Grid {
//     pub fn len(&self) -> usize {
//         self.0.len()
//     }
// }
impl core::ops::Deref for Grid {
    type Target = [Option<Lane>; 5];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
// // DerefMut is optional
impl core::ops::DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl core::ops::Deref for Lane {
    type Target = [Field; 9];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for Lane {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl core::ops::Deref for Field {
    type Target = VecDeque<Entity>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for Field {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################
// ############################################################################

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_zombie_moves_left() {
        let mut state = State {
            tick: 0,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                ])),
                None,
                None,
            ]),
            // tick_interval_ms: 799,
            ..Default::default()
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(0)])),
            &third_lane.0[8]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(&Field(VecDeque::new()).len(), &third_lane.0[8].len());
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(1)])),
            &third_lane.0[7]
        );
    }

    #[test]
    fn two_zombies_move_left() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(0)])),
            &third_lane.0[8]
        );
        state.next();
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(&Field(VecDeque::new()).len(), &third_lane.0[8].len());
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(2)])),
            &third_lane.0[6]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(2)])),
            &third_lane.0[5]
        );
    }

    #[test]
    fn two_zombies_leaves_lane() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::from([Entity::Zombie(0)])),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(0)])),
            &third_lane.0[0]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(0)])),
            &third_lane.0[1]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(&true, &third_lane.0[1].is_empty());
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(1)])),
            &third_lane.0[0]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(&true, &third_lane.0[0].is_empty());
        assert_eq!(&true, &third_lane.0[1].is_empty());
    }

    #[test]
    fn one_bullet_move_right() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::from([Entity::Bullet(0)])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(0)])),
            &third_lane.0[0]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(
            &Field(VecDeque::new()).is_empty(),
            &third_lane.0[0].is_empty()
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(1)])),
            &third_lane.0[1]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(2)])),
            &third_lane.0[2]
        );
    }

    #[test]
    fn two_bullets_move_right() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::from([Entity::Bullet(0)])),
                    Field(VecDeque::from([Entity::Bullet(0)])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(0)])),
            &third_lane.0[0]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(0)])),
            &third_lane.0[1]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(&true, &third_lane.0[0].is_empty());
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(1)])),
            &third_lane.0[1]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(1)])),
            &third_lane.0[2]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(2)])),
            &third_lane.0[2]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(2)])),
            &third_lane.0[3]
        );
    }

    #[test]
    fn bullets_leave_the_lane() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::from([Entity::Bullet(0)])),
                    Field(VecDeque::from([Entity::Bullet(0)])),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(0)])),
            &third_lane.0[7]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(0)])),
            &third_lane.0[8]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(&true, &third_lane.0[7].is_empty());
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(1)])),
            &third_lane.0[8]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(&true, &third_lane.0[7].is_empty());
        assert_eq!(&true, &third_lane.0[8].is_empty());
    }

    #[test]
    fn bullet_hits_zombie() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::new()),
                    Field(VecDeque::from([Entity::Bullet(0)])),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(0)])),
            &third_lane.0[1]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(0)])),
            &third_lane.0[2]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(&true, &third_lane.0[0].is_empty());
        assert_eq!(&true, &third_lane.0[1].is_empty());
        assert_eq!(&true, &third_lane.0[2].is_empty());
        assert_eq!(&true, &third_lane.0[3].is_empty());
    }

    #[test]
    fn zombie_walks_into_bullet() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::from([Entity::Bullet(0)])),
                    Field(VecDeque::new()),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(0)])),
            &third_lane.0[0]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(0)])),
            &third_lane.0[2]
        );
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!(&true, &third_lane.0[0].is_empty());
        assert_eq!(&true, &third_lane.0[1].is_empty());
        assert_eq!(&true, &third_lane.0[2].is_empty());
        assert_eq!(&true, &third_lane.0[3].is_empty());
    }

    #[test]
    fn turret_spawns_bullet() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 1,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::from([Entity::Turret])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(&Field(VecDeque::from([Entity::Turret])), &third_lane.0[0]);
        assert_eq!(&true, &third_lane.0[1].is_empty());

        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();

        assert_eq!(&Field(VecDeque::from([Entity::Turret])), &third_lane.0[0]);
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(1)])),
            &third_lane.0[1]
        );
        assert_eq!(&true, &third_lane.0[2].is_empty());
        assert_eq!(&true, &third_lane.0[3].is_empty());
        assert_eq!(&true, &third_lane.0[4].is_empty());
    }

    #[test]
    fn turret_hits_zombie() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 1,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::from([Entity::Turret])),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(&Field(VecDeque::from([Entity::Turret])), &third_lane.0[0]);
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(0)])),
            &third_lane.0[1]
        );

        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();

        assert_eq!(&Field(VecDeque::from([Entity::Turret])), &third_lane.0[0]);
        assert_eq!(&true, &third_lane.0[1].is_empty());
        assert_eq!(&true, &third_lane.0[2].is_empty());
        assert_eq!(&true, &third_lane.0[3].is_empty());
        assert_eq!(&true, &third_lane.0[4].is_empty());
    }

    #[test]
    fn turret_spawns_bullet_every_2nd_tick() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 2,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::from([Entity::Turret])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(&Field(VecDeque::from([Entity::Turret])), &third_lane.0[0]);
        assert_eq!(&true, &third_lane.0[1].is_empty());

        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();

        assert_eq!(&Field(VecDeque::from([Entity::Turret])), &third_lane.0[0]);
        assert_eq!(&true, &third_lane.0[1].is_empty());

        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();

        assert_eq!(&Field(VecDeque::from([Entity::Turret])), &third_lane.0[0]);
        assert_eq!(
            &Field(VecDeque::from([Entity::Bullet(2)])),
            &third_lane.0[1]
        );
        assert_eq!(&true, &third_lane.0[2].is_empty());
        assert_eq!(&true, &third_lane.0[3].is_empty());
        assert_eq!(&true, &third_lane.0[4].is_empty());
    }

    #[test]
    fn zombie_spawns_every_2nd_tick() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 2,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(&true, &third_lane.0[0].is_empty());
        assert_eq!(&true, &third_lane.0[8].is_empty());

        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();

        assert_eq!(&true, &third_lane.0[0].is_empty());
        assert_eq!(&true, &third_lane.0[7].is_empty());
        assert_eq!(&true, &third_lane.0[8].is_empty());

        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();

        assert_eq!(&true, &third_lane.0[0].is_empty());
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(2)])),
            &third_lane.0[8]
        );

        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();

        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(3)])),
            &third_lane.0[7]
        );
        assert_eq!(&true, &third_lane.0[8].is_empty());

        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();

        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(4)])),
            &third_lane.0[6]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(4)])),
            &third_lane.0[8]
        );

    }

    #[test]
    fn zombie_hits_turret() {
        let mut state = State {
            tick: 0,
            tick_interval_ms: 700,
            spawn_rates: SpawnRates {
                zombies_each_x_ticks: 0,
                bullets_each_x_ticks: 0,
            },
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::from([Entity::Turret])),
                    Field(VecDeque::from([Entity::Zombie(0)])),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                ])),
                None,
                None,
            ]),
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(
            &Field(VecDeque::from([Entity::Turret])),
            &third_lane.0[0]
        );
        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(0)])),
            &third_lane.0[1]
        );

        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();


        assert_eq!(
            &Field(VecDeque::from([Entity::Zombie(1)])),
            &third_lane.0[0]
        );
        assert_eq!(&true, &third_lane.0[1].is_empty());
    }

}
