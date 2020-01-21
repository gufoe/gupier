use crate::com::Channel;
use kiss3d::window::Window;
use kiss3d::light::Light;
use na::geometry::Point3;
use std::net::{SocketAddr, ToSocketAddrs};
use kiss3d::camera::FirstPerson;
use crate::gameplay::*;
use crate::server::Packet;
use crate::util;

#[derive(Clone)]
pub enum ClientState {
    Menu,
    Joining,
    Joined,
    Playing,
}

#[derive(Clone)]
struct Game<G> {
    player_id: usize,
    ch: Channel,
    addr: SocketAddr,
    gameplay: G
}

impl<G> Game<G> {
    fn send_ro<T: serde::Serialize>(&mut self, data: T) {
        self.ch.send_ro(self.addr, data);
    }
}


#[allow(dead_code)]
struct Client<G> {
    state: ClientState,
    window: Window,
    game: Option<Game<G>>,
    cam: FirstPerson,
}


impl<G> Client<G>
where G: GameplayClient  {
    fn new() -> Client<G> {
        let mut window = Window::new("Kiss3d: cube");
        window.set_light(Light::StickToCamera);
        // let cam = kiss3d::camera::FirstPerson::new(Point3::new(10.0, 10.0, 10.0), Point3::new(0.0, 0.0, 0.0));
        let cam = kiss3d::camera::FirstPerson::new_with_frustrum(
            -80.0,
            0.2,
            1000.0,
            Point3::new(10.0, 10.0, 10.0),
            Point3::new(0.0, 0.0, 0.0),
        );
        Client {
            state: ClientState::Menu,
            window,
            cam,
            game: None,
        }
    }
    fn connect(&mut self, host: String, gameplay: G) {
        self.game = Some(Game {
            player_id: 0,
            ch: Channel::spawn(None),
            addr: host.to_socket_addrs().expect("Invalid host").next().expect("Cannot resolve hostname"),
            gameplay,
        })
    }
    fn join(&mut self) {
        self.state = ClientState::Joining;
        let game = self.game.as_mut().expect("game not ready ASD7YH");

        println!("[c] joining lobby");
        game.send_ro(Packet::Join(util::rand_color()));

        loop {
            match game.ch.recv() {
                Some((_, Packet::Joined(id))) => {
                    println!("[c] joined {}", id);
                    game.player_id = id;
                    return
                },
                _ => {},
            }
        }
    }

    fn wait_start(&mut self) {
        self.state = ClientState::Joined;
        let game = self.game.as_mut().expect("game not ready ASD7YH");

        println!("[c] wait game start");
        loop {
            match game.ch.recv() {
                Some((_, Packet::GameStarted)) => {
                    return
                },
                _ => {},
            }
        }
    }

    fn run(&mut self) {
        println!("[c] game started");
        self.state = ClientState::Playing;
        let game = self.game.as_mut().expect("game not ready ASD7YH");
        let window = &mut self.window;

        game.gameplay.init(game.player_id, game.addr);


        loop {
            let p = game.ch.recv_all();

            p.iter().for_each(|packet| {
                game.gameplay.on_packet(&mut game.ch, &packet.1, window);
            });

            window.events().iter().for_each(|e| {
                game.gameplay.on_event(&mut game.ch, &e.value, window);
            });

            game.gameplay.update(&mut game.ch, window, &mut self.cam);

            if !window.render_with_camera(&mut self.cam) {
                return
            }

        }
    }
}

pub fn connect<G>(host: String, gameplay: G)
where G: GameplayClient {
    let mut client = Client::new();
    client.connect(host, gameplay);
    client.join();
    client.wait_start();
    client.run();
}
