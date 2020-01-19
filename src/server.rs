use crate::com::Channel;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::util;
use crate::gameplay::GameplayHost;
use std::net::SocketAddr;


#[derive(Clone, Debug, Serialize, Deserialize)]

#[allow(dead_code)]
pub struct Player {
    pub id: usize,
    pub addr: SocketAddr,
    pub color: (f32, f32, f32),
}

impl Player {
    pub fn new(id: usize, addr: SocketAddr, color: (f32, f32, f32)) -> Player {
        Player {
            id,
            addr,
            color,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub enum Packet {
    Join((f32, f32, f32)),
    Joined(usize),
    GameStarted,
    Control,
}

#[allow(dead_code)]
pub struct LobbyConfig {
    min_players: usize,
    port: u16,
}

#[allow(dead_code)]
pub enum LobbyState {
    Setup,
    Pending,
    Gameplay,
    Finished,
}

#[allow(dead_code)]
struct Lobby<G> {
    players: HashMap<usize, Player>,
    sock2pl: HashMap<SocketAddr, usize>,
    state: LobbyState,
    config: LobbyConfig,
    ch: Channel,
    gameplay: G
}


impl<G> Lobby<G>
where G: GameplayHost {
    fn spawn(config: LobbyConfig, gameplay: G) -> Lobby<G> {

        Lobby {
            players: HashMap::new(),
            sock2pl: HashMap::new(),
            state: LobbyState::Setup,
            ch: Channel::spawn(Some(format!("0.0.0.0:{}", config.port))),
            config,
            gameplay,
        }
    }

    fn wait_players(&mut self) {
        self.state = LobbyState::Pending;
        while self.players.len() < self.config.min_players {
            println!("[s] waiting players");
            let (addr, data) = self.ch.recv().unwrap();
            match data {
                Packet::Join(color) => {
                    println!("[s] new client {} {:?}!!", addr, color);
                    let id = self.players.len()+1;
                    self.players.insert(id, Player::new(id, addr, color));
                    self.sock2pl.insert(addr, id);
                    self.broadcast(Packet::Joined(id));
                },
                _ => {},
            }
        }
    }

    fn run_gameplay(&mut self) {
        self.state = LobbyState::Gameplay;
        println!("[s] game started");
        self.gameplay.init(&mut self.ch, &self.players);
        self.broadcast(Packet::GameStarted);

        let interval_ms = 1000/60;
        loop {
            let t_start = util::now();

            let p = self.ch.recv_all();
            p.iter().for_each(|packet| {
                self.gameplay.on_packet(&mut self.ch, self.sock2pl[&packet.0], &packet.1);
            });


            self.gameplay.update(&mut self.ch, &self.players);

            let t_used = ((util::now() - t_start) * 1000.0) as u64;
            // println!("[s] update took {} ms", t_used);
            if t_used < interval_ms {
                util::sleep(interval_ms - t_used);
            }
        }
    }

    #[allow(dead_code)]
    fn broadcast<T: serde::Serialize>(&mut self, data: T) {
        let ch = &mut self.ch;
        self.players.values().for_each(|pl| {
            ch.send_rs(pl.addr.clone(), &data);
        });
    }
}

pub fn serve<G>(port: u16, gameplay: G)
where G: GameplayHost {
    let mut lobby = Lobby::spawn(LobbyConfig {
        port,
        min_players: 1,
    }, gameplay);
    lobby.wait_players();
    lobby.run_gameplay();
}
