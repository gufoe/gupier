use crate::gameplay::*;
use crate::world::*;
use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use crate::com::Channel;
use crate::server::Player;
use crate::util;
use bimap::BiMap;
use nalgebra::geometry::Isometry3;
use nalgebra::base::Vector3;

pub const PL_RAD: f32 = 0.5;

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
}

impl LitioPlayer {
    fn new(color: (f32, f32, f32)) -> LitioPlayer {
        LitioPlayer {
            color,
            life: 100,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LitioBullet {
    pub damage: i16,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LitioBox {
    pub color: (f32, f32, f32),
    pub dim: (f32, f32, f32),
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LitioThing {
    Box(LitioBox),
    Player(LitioPlayer),
    Bullet(LitioBullet),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LitioThingUpdate {
    pub iso: Isometry3<f32>,
    pub thing: LitioThing,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LitioUpdate {
    pub things: HashMap<usize, LitioThingUpdate>,
    pub pl2thing: BiMap<usize, usize>,
}

#[allow(dead_code)]
impl LitioUpdate {
    pub fn new() -> LitioUpdate {
        LitioUpdate {
            things: HashMap::new(),
            pl2thing: BiMap::new(),
        }
    }
}

#[allow(dead_code)]
pub struct Host {
    // from logic to physycs
    world: World,
    things: HashMap<usize, LitioThing>,
    last_input: HashMap<usize, LitioPlayerInput>,
    pl2thing: BiMap<usize, usize>,
}

impl Host {
    pub fn new() -> Host {
        Host {
            world: World::new(),
            things: HashMap::new(),
            last_input: HashMap::new(),
            pl2thing: BiMap::new(),
        }
    }

    pub fn gen_update(&self) -> LitioUpdate {
        let things = self.things.iter().map(|(id, thing)| {
            (*id, LitioThingUpdate {
                thing: thing.clone(),
                iso: self.world.get_iso(*id),
            })
        }).collect();

        LitioUpdate {
            pl2thing: self.pl2thing.clone(),
            things,
        }
    }

    #[allow(dead_code)]
    fn init_map(&mut self) {
        use nphysics3d::object::BodyStatus;
        use crate::util::rand_float as rf;

        let lbox = LitioBox {
            color: util::rand_color(),
            dim: (100.0, 0.1, 100.0),
        };
        let w = &mut self.world;

        let node = w.add_cube(lbox.dim);
        self.things.insert(node, LitioThing::Box(lbox));
        w.get_body_mut(node).set_status(BodyStatus::Static);
        w.set_pos(node, &(0.0, -0.1, 0.0));

        for a in 0..2 {
            for b in 0..50 {
                for c in 0..2 {
                    let lbox = LitioBox {
                        color: util::rand_color(),
                        dim: (rf(0.5,0.9), 1.0, rf(0.3,1.2)),
                    };
                    let node = w.add_cube(lbox.dim);
                    w.set_pos(node, &((a*2) as f32, (b*2) as f32, (c*2) as f32));
                    self.things.insert(node, LitioThing::Box(lbox));
                }
            }
        }
    }
}


impl GameplayHost for Host {
    fn init(&mut self, _ch: &mut Channel, players: &HashMap::<usize, Player>) {
        self.init_map();
        players.values().for_each(|player| {
            let pl = LitioPlayer::new(player.color);
            let node = self.world.add_ball(1.0);
            self.world.set_pos(node, &(0.5, 60.0, 1.0));

            self.pl2thing.insert(player.id, node);
            self.things.insert(node, LitioThing::Player(pl));
        });
        println!("[s] gameplay initiated");
    }
    fn on_packet(&mut self, _ch: &mut Channel, player_id: usize, tx: &[u8]) {

        let tx: Tx = bincode::deserialize(tx).unwrap_or(Tx::Unknown);
        // println!("[s] rec packet {:?}", tx);
        match tx {
            Tx::Input(x) => {
                self.last_input.insert(*self.pl2thing.get_by_left(&player_id).unwrap(), x);
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

        let w = &mut self.world;

        self.last_input.iter().for_each(|(id, input)| {
            // println!("input: {:#?}", input);
            let pl = w.get_rigid_mut(*id);
            // pl.set_angular_velocity(cur_vel * 0.01 + input.acc * 10.0);
            if input.acc.magnitude() > 0.01 {
                pl.apply_force(0, &Force3::torque(input.acc.normalize()*2.0), ForceType::VelocityChange, true );
            }
        });

        self.world.update_physics();
        let update = self.gen_update();
        players.values().for_each(|p| ch.send_ro(p.addr, Tx::Update(update.clone())));
    }
}
