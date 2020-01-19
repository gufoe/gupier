use crate::com::Channel;
use kiss3d::window::Window;
use kiss3d::event::WindowEvent;
use std::collections::HashMap;
use crate::server::Player;
use std::net::SocketAddr;


pub trait GameplayHost {
    fn init(&mut self, ch: &mut Channel, players: &HashMap::<usize, Player>);
    fn update(&mut self, ch: &mut Channel, players: &HashMap::<usize, Player>);
    fn on_packet(&mut self, ch: &mut Channel, sender: usize, tx: &[u8]);
}

pub trait GameplayClient {
    fn init(&mut self, id: usize, addr: SocketAddr);
    fn on_event(&mut self, ch: &mut Channel, e: &WindowEvent, window: &mut Window);
    fn update(&mut self, ch: &mut Channel, window: &mut Window, cam: &mut kiss3d::camera::FirstPerson);
    fn on_packet(&mut self, ch: &mut Channel, tx: &[u8], window: &mut Window);
}
