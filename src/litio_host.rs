use crate::gameplay::*;
use crate::world::*;
use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use crate::com::Channel;
use crate::server::Player;
use crate::util;
use nalgebra::geometry::Isometry3;
use nalgebra::base::Vector3;
use util::uid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LitioPlayerInput {
    pub acc: Vector3<f32>,
    pub look_at: Vector3<f32>,
    pub look_at_rot: Vector3<f32>,
}

impl LitioPlayerInput {
    pub fn new() -> Self {
        Self {
            acc: Vector3::zeros(),
            look_at: Vector3::zeros(),
            look_at_rot: Vector3::zeros(),
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub enum Tx {
    Unknown,
    Input(LitioPlayerInput),
    Update(LitioUpdate),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LitioPlayer {
    pub color: (f32, f32, f32),
    pub life: i16,
    // #[serde(skip_serializing)]
    pub ph_node: usize,
}

impl LitioPlayer {
    fn new(color: (f32, f32, f32), ph_node: usize) -> LitioPlayer {
        LitioPlayer {
            color,
            life: 100,
            ph_node,
        }
    }
}


#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LitioBox {
    pub color: (f32, f32, f32),
    pub dim: (f32, f32, f32),
    pub ph_node: usize,
}



#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LitioThing {
    Box(LitioBox),
    Player(LitioPlayer),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LitioThingUpdate {
    pub iso: Isometry3<f32>,
    pub thing: LitioThing,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LitioUpdate {
    pub things: HashMap<usize, LitioThingUpdate>,
}

#[allow(dead_code)]
impl LitioUpdate {
    pub fn new() -> LitioUpdate {
        LitioUpdate {
            things: HashMap::new(),
        }
    }
}

#[allow(dead_code)]
pub struct Host {
    // from logic to physycs
    world: World,
    catcher: usize,
    time: usize,
    things: HashMap<usize, LitioThing>,
    id2pl: HashMap<usize, usize>,
    last_input: HashMap<usize, LitioPlayerInput>,
}

impl Host {
    pub fn new() -> Host {
        Host {
            world: World::new(),
            things: HashMap::new(),
            catcher: 0,
            time: 0,
            id2pl: HashMap::new(),
            last_input: HashMap::new(),
        }
    }

    pub fn gen_update(&self) -> LitioUpdate {
        let w = &self.world;
        let things = self.things.iter().map(|(id, thing)| {
            (*id, LitioThingUpdate {
                thing: thing.clone(),
                iso: match thing {
                    LitioThing::Box(x) => w.get_iso(x.ph_node),
                    LitioThing::Player(x) => w.get_iso(x.ph_node),
                },
            })
        }).collect();

        LitioUpdate {
            things,
        }
    }

    #[allow(dead_code)]
    fn init_map(&mut self) {
        use nphysics3d::object::BodyStatus;
        use crate::util::rand_float as rf;

        let w = &mut self.world;

        let dim = (100.0, 0.1, 100.0);
        let lbox = LitioBox {
            color: util::rand_color(),
            dim,
            ph_node: w.add_cube(dim),
        };
        w.get_body_mut(lbox.ph_node).set_status(BodyStatus::Static);
        w.set_pos(lbox.ph_node, &(0.0, -0.1, 0.0));
        self.things.insert(uid(), LitioThing::Box(lbox));

        for a in 0..2 {
            for b in 0..30 {
                for c in 0..2 {
                    let dim = (rf(0.5,0.9), 1.0, rf(0.3,1.2));
                    let lbox = LitioBox {
                        color: util::rand_color(),
                        dim,
                        ph_node: w.add_cube(dim)
                    };
                    w.set_pos(lbox.ph_node, &((a*2) as f32, (b*2) as f32, (c*2) as f32));
                    self.things.insert(uid(), LitioThing::Box(lbox));
                }
            }
        }
    }

    fn pl(&mut self, id: usize) -> &mut LitioPlayer {
        match self.things.get_mut(&id).expect("cannot find player") {
            LitioThing::Player(p) => p,
            _ => panic!("thing is not player"),
        }
    }

    fn elect_catcher(&mut self, players: &HashMap::<usize, Player>) {
        let ids: Vec<usize> = players.keys().cloned().collect();
        self.catcher = util::pick(&ids);
    }

    fn toggle_colors(&mut self, id: usize) {
        if let LitioThing::Player(x) = self.things.get_mut(&id).expect(&format!("cannot find player to toggle_colors for {}", id)) {
            let g = x.color.1;
            x.color.1 = x.color.0;
            x.color.0 = g;
        }
    }
}


impl GameplayHost for Host {
    fn init(&mut self, _ch: &mut Channel, players: &HashMap::<usize, Player>) {
        players.iter().for_each(|(id, _player)| {
            let pl = LitioPlayer::new(
                (0.0, 1.0, 0.0),
                self.world.add_ball(1.0),
            );
            self.id2pl.insert(pl.ph_node, *id);
            self.world.set_pos(pl.ph_node, &(0.5, 60.0, 1.0));
            self.things.insert(*id, LitioThing::Player(pl));
        });
        self.init_map();
        self.elect_catcher(players);
        self.toggle_colors(self.catcher);
        println!("[s] gameplay initiated");
    }
    fn on_packet(&mut self, _ch: &mut Channel, player_id: usize, tx: &[u8]) {

        let tx: Tx = bincode::deserialize(tx).unwrap_or(Tx::Unknown);
        // println!("[s] rec packet {:?}", tx);
        match tx {
            Tx::Input(x) => {
                self.last_input.insert(player_id, x);
            },
            _ => {
                println!("[s] tx not handled: {:?}", tx);
            }
        }
    }
    fn update(&mut self, ch: &mut Channel, players: &HashMap::<usize, Player>) {
        use nphysics3d::algebra::*;
        use nphysics3d::algebra::ForceType;
        use nphysics3d::object::Body;

        self.time+= 1;

        let w = &mut self.world;
        let things = &mut self.things;

        self.last_input.iter().for_each(|(id, input)| {
            // println!("input: {:#?}", input);
            if let LitioThing::Player(pl) = things.get_mut(id).unwrap() {
                // pl.life = util::rand_usize(100) as i16;
                let body = w.get_rigid_mut(pl.ph_node);
                // pl.set_angular_velocity(cur_vel * 0.01 + input.acc * 10.0);
                if input.acc.magnitude() > 0.01 {
                    body.apply_force(0, &Force3::torque(input.acc.normalize()*2.0), ForceType::VelocityChange, true );
                }
            }

        });

        if self.time % 1000 == 0 {
            self.toggle_colors(self.catcher);
            self.elect_catcher(players);
            self.toggle_colors(self.catcher);
        }

        self.world.update_physics();
        for contact in self.world.geometrical_world.contact_events() {
            match contact {
                ncollide3d::pipeline::narrow_phase::ContactEvent::Started(h1, h2) => {
                    let h1 = self.world.col2id.get(h1).unwrap();
                    let h2 = self.world.col2id.get(h2).unwrap();
                    println!("contact {} {}, {:?}", h1, h2, self.id2pl);
                    if self.id2pl.contains_key(&h1) && self.id2pl.contains_key(&h2) {
                        println!("ok");
                        let id1 = self.id2pl.get(&h1).unwrap();
                        let id2 = self.id2pl.get(&h2).unwrap();

                        if let LitioThing::Player(p) = self.things.get_mut(&id1).unwrap() {
                             p.color.1-= 0.2
                        }
                        if let LitioThing::Player(p) = self.things.get_mut(&id2).unwrap() {
                             p.color.1-= 0.2
                        }
                    }
                },
                _ => {},
            }
        }

        // println!("send updates to {} players: {:#?}", players.len(), update);
        let update = self.gen_update();
        players.values().for_each(|p| ch.send_ro(p.addr, Tx::Update(update.clone())));
    }
}
