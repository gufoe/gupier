use crate::gameplay::*;
use std::collections::HashMap;
use kiss3d::window::Window;
use kiss3d::event::WindowEvent;
use kiss3d::event::WindowEvent::*;
use crate::litio_host::*;
use kiss3d::scene::SceneNode;
use crate::com::Channel;
use na::geometry::Point3;
use std::net::SocketAddr;
use nalgebra::base::Vector3;

#[allow(dead_code)]
#[derive(Clone)]
pub struct GameState {
    player_id: usize,
    addr: SocketAddr,
    input: LitioPlayerInput
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            addr: "0.0.0.0:1".parse().unwrap(),
            player_id: 0,
            input: LitioPlayerInput::new(),
        }
    }
}

pub struct Client {
    _is_ready: bool,
    ps: GameState,
    nodes: HashMap<usize, SceneNode>,
    state: Option<LitioUpdate>,
}

impl Client {
    pub fn new() -> Client {
        Client {
            _is_ready: false,
            ps: GameState::new(),
            nodes: HashMap::new(),
            state: None,
        }
    }

    fn pl_thing_id(&self) -> usize {
        *self.state.as_ref().unwrap().pl2thing.get_by_left(&self.ps.player_id).unwrap()
    }

    fn pl_thing(&self) -> &LitioThingUpdate {
        self.state.as_ref().unwrap().things.get(&self.pl_thing_id()).unwrap()
    }

    fn is_ready(&self) -> bool {
        self.state.is_some()
    }
}





impl GameplayClient for Client {
    fn init(&mut self, id: usize, addr: SocketAddr) {
        println!("[c] init gameplay");
        self.ps.player_id = id;
        self.ps.addr = addr;
    }

    fn on_event(&mut self, _ch: &mut Channel, e: &WindowEvent, window: &mut Window) {
        use std::f64::consts::PI as PI;
        let w = window.width();
        let h = window.height();
        let input = &mut self.ps.input;

        match e {
            CursorPos(x, y, _mods) => {
                let dx = x - (w/2) as f64;
                let dy = y - (h/2) as f64;
                if dx.abs() > 0.0 {
                    input.look_at.x+= (dx / 200.0) as f32;
                }
                if dy.abs() > 0.0 {
                    input.look_at.y+= (dy / 200.0) as f32;
                    input.look_at.y = input.look_at.y.min((PI*0.49) as f32).max((-PI*0.49) as f32);
                }
                // println!("{:.2}, {:.2}", input.look_at.x, input.look_at.y);

                input.look_at_rot.x = - input.look_at.y.cos() * input.look_at.x.sin();
                input.look_at_rot.y = - input.look_at.y.sin();
                input.look_at_rot.z = input.look_at.y.cos() * input.look_at.x.cos();
                window.set_cursor_position((w/2) as f64, (h/2) as f64);
            },
            Key(_key, _action, _modifiers) => {
                // println!("{:?}", key);
            },
            _ => {},
        }
    }
    fn on_packet(&mut self, _ch: &mut Channel, tx: &[u8], window: &mut Window) {
        let tx = bincode::deserialize(tx).unwrap_or(Tx::Unknown);
        match tx {
            Tx::Update(update) => {

                update.things.iter().for_each(|(id, info)| {
                    let mut old_info = None;
                    if let Some(x) = &self.state {
                        old_info = x.things.get(id);
                    }
                    match &info.thing {
                        LitioThing::Box(p) => {
                            if !self.nodes.contains_key(id) {
                                println!("creating BOX");
                                let node = window.add_cube(p.dim.0 * 2.0, p.dim.1 * 2.0, p.dim.2 * 2.0);
                                self.nodes.insert(*id, node);
                            }
                            let node = self.nodes.get_mut(id).expect("AS7D6");
                            // node.set_local_transformation(info.iso);
                            let mut loc = info.iso.translation;
                            let rot = info.iso.rotation;
                            if let Some(x) = old_info {
                                loc.x = (loc.x + x.iso.translation.x) / 2.0;
                                loc.y = (loc.y + x.iso.translation.y) / 2.0;
                                loc.z = (loc.z + x.iso.translation.z) / 2.0;
                            }
                            node.set_local_translation(loc);
                            node.set_local_rotation(rot);
                            node.set_color(p.color.0, p.color.1, p.color.2);
                        },
                        LitioThing::Player(p) => {
                            if !self.nodes.contains_key(id) {
                                println!("creating PLAYER");
                                let mut node = window.add_sphere(PL_RAD*2.0);
                                node.set_texture_from_file(&std::path::Path::new("./tex.jpg"), "tex");
                                self.nodes.insert(*id, node);
                            }
                            let node = self.nodes.get_mut(id).expect("AS7D6");
                            let mut loc = info.iso.translation;
                            let rot = info.iso.rotation;
                            if let Some(x) = old_info {
                                loc.x = (loc.x + x.iso.translation.x) / 2.0;
                                loc.y = (loc.y + x.iso.translation.y) / 2.0;
                                loc.z = (loc.z + x.iso.translation.z) / 2.0;
                            }
                            node.set_local_translation(loc);
                            node.set_local_rotation(rot);
                            node.set_color(p.color.0, p.color.1, p.color.2);
                        },
                        _ => {},
                    }
                });

                self.state = Some(update);
            },
            _ => {
                println!("[c] tx not handled {:?}", tx);
            },
        }
    }

    fn update(&mut self, ch: &mut Channel, window: &mut Window, cam: &mut kiss3d::camera::FirstPerson) {
        if self.is_ready() {
            // window.set_cursor_grab(true);
            window.hide_cursor(true);

            use kiss3d::event::Key;
            use kiss3d::event::Action::*;
            use std::f64::consts::PI as PI;
            let a = self.ps.input.look_at.x - PI as f32 / 2.0;
            let mut v = Vector3::zeros();
            if window.get_key(Key::W) == Press {
                v+= Vector3::new(-a.sin(), 0.0, a.cos());
            }
            if window.get_key(Key::S) == Press {
                let a = a + PI as f32;
                v+= Vector3::new(-a.sin(), 0.0, a.cos());
            }
            if window.get_key(Key::A) == Press {
                let a = a - 0.5 * PI as f32;
                v+= Vector3::new(-a.sin(), 0.0, a.cos());
            }
            if window.get_key(Key::D) == Press {
                let a = a + 0.5 * PI as f32;
                v+= Vector3::new(-a.sin(), 0.0, a.cos());
            }
            self.ps.input.acc = v;

            let lar = self.ps.input.look_at_rot;
            let me = self.pl_thing();
            let pos = me.iso.translation;
            // println!("[c] pos {:?}", pos);

            // At center
            // cam.look_at(
            //     Point3::new(pos.x, pos.y, pos.z),
            //     Point3::new(
            //         pos.x + lar.x,
            //         pos.y + lar.y,
            //         pos.z + lar.z,
            //     )
            // );

            // Look over
            cam.look_at(
                Point3::new(
                    pos.x - lar.x * 6.0,
                    pos.y - lar.y * 6.0 + 5.0,
                    pos.z - lar.z * 6.0,
                ),
                Point3::new(
                    pos.x,
                    pos.y + 1.0,
                    pos.z,
                )
            );


            ch.send_rs(self.ps.addr, Tx::Input(LitioPlayerInput {
                acc: self.ps.input.acc,
                look_at: self.ps.input.look_at,
                look_at_rot: self.ps.input.look_at_rot,
            }));
            // cam.look_at(
            //     Point3::new(pos.x, pos.y, pos.z),
            //     Point3::new(0.0, 0.0, 0.0),
            // );
        } else {
            println!("[c] downloading game state");
        }
    }
}
