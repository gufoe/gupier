use laminar::{Config, Packet, Socket, SocketEvent};
use bincode::{deserialize, serialize};
use std::sync::{Arc, RwLock};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Channel {
    tx: crossbeam_channel::Sender<laminar::Packet>,
    rx: crossbeam_channel::Receiver<laminar::SocketEvent>,
    poll_thread: Arc<RwLock<std::thread::JoinHandle<()>>>,
}


impl<'a> Channel {
    #[allow(dead_code)]
    pub fn spawn(host: Option<String>) -> Channel {
        let mut socket = if host.is_none() {
            Socket::bind_with_config(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0), Config::default())
        } else {
            Socket::bind(host.unwrap())
        }.unwrap();

        Channel {
            tx: socket.get_packet_sender(),
            rx: socket.get_event_receiver(),
            poll_thread: Arc::new(RwLock::new(std::thread::spawn(move || socket.start_polling()))),
        }
    }

    #[allow(dead_code)]
    pub fn recv_all(&mut self) -> Vec<(SocketAddr, Vec<u8>)> {
        let mut ret = vec![];
        self.rx.try_iter().for_each(|packet| {
            match packet {
                SocketEvent::Connect(addr) => {
                    println!("[com] connected {}", addr);
                }
                SocketEvent::Timeout(addr) => {
                    println!("[com] timeout {}", addr);
                }
                SocketEvent::Packet(pkt) => {
                    ret.push((pkt.addr(), pkt.payload().to_vec()));
                }
            }
        });
        ret
    }

    #[allow(dead_code)]
    pub fn recv<T>(&mut self) -> Option<(SocketAddr, T)>
    where T: serde::de::DeserializeOwned {
        while let Ok(pkt) = self.rx.recv() {
            match pkt {
                SocketEvent::Connect(addr) => {
                    println!("[com] connected {}", addr);
                }
                SocketEvent::Timeout(addr) => {
                    println!("[com] timeout {}", addr);
                }
                SocketEvent::Packet(pkt) => {
                    // println!("[com] packet from {}", pkt.addr());
                    let data = deserialize(&pkt.payload());
                    if data.is_err() {
                        println!("[com] invalid packet");
                        continue;
                    }
                    return Some((pkt.addr(), data.unwrap()))
                }
            }
        }
        println!("[com] exit");
        None
    }

    #[allow(dead_code)]
    pub fn send_ro<T>(&mut self, addr: SocketAddr, x: T)
        where T: serde::Serialize {
        self.tx.send(Packet::reliable_ordered( addr, serialize(&x).unwrap(), None )).expect("cannot send packet");
    }

    #[allow(dead_code)]
    pub fn send_rs<T>(&mut self, addr: SocketAddr, x: T)
        where T: serde::Serialize {
        self.tx.send(Packet::reliable_sequenced( addr, serialize(&x).unwrap(), None )).expect("cannot send packet");
    }
}
