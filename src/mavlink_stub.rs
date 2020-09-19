use std::collections::HashMap;
pub use std::mem::{discriminant, Discriminant};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use mavlink::{common::*, MavConnection};

use futures::{future::Either, prelude::*};
use smol::{
    channel::{self, Receiver, Sender},
    lock::Mutex,
};

pub use discriminant as message_type;

pub type MavMessageType = Discriminant<MavMessage>;

#[derive(Debug)]
pub enum Operation {
    Subscribe {
        message: MavMessageType,
        backchannel: Sender<MavMessage>,
    },
    Send(MavMessage),
}

struct Mailbag {
    pub backchannel: Sender<MavMessage>,
    pub reccuring: bool,
}

pub struct MavlinkConnectionHandler {
    conn: Arc<dyn MavConnection<mavlink::common::MavMessage> + Sync + Send>,
    subscriptions: Mutex<HashMap<MavMessageType, Vec<Mailbag>>>,
    tx: Sender<Operation>,
    rx: Receiver<Operation>,
    last_heartbeat: Mutex<Option<Instant>>,
}

impl MavlinkConnectionHandler {
    pub fn new(conn_str: &str) -> Self {
        let mut conn = mavlink::connect::<MavMessage>(conn_str).expect("Oh no");
        conn.set_protocol_version(mavlink::MavlinkVersion::V1);
        let conn = Arc::from(conn);
        let (tx, rx) = channel::unbounded();
        let subscriptions = Mutex::new(HashMap::new());
        let last_heartbeat = Mutex::new(None);
        Self {
            conn,
            subscriptions,
            tx,
            rx,
            last_heartbeat,
        }
    }

    pub async fn subscribe(
        &self,
        message: MavMessageType,
    ) -> Pin<Box<dyn Stream<Item = MavMessage>>> {
        let (tx, rx) = channel::unbounded();
        self.tx
            .send(Operation::Subscribe {
                message,
                backchannel: tx,
            })
            .await
            .expect("unable to send");
        Box::pin(rx)
    }

    pub async fn request<T: Sized>(&self, message: MavMessageType) -> MavMessage {
        let (tx, rx) = channel::unbounded();
        self.tx
            .send(Operation::Subscribe {
                message,
                backchannel: tx,
            })
            .await
            .expect("unable to send");
        rx.recv().map(|m| m.expect("Oh no!")).await
    }

    pub async fn send(&self, message: MavMessage) {
        self.tx
            .send(Operation::Send(message))
            .await
            .expect("unable to send");
    }

    pub async fn last_heartbeat(&self) -> Option<Instant> {
        let time = self.last_heartbeat.lock().await;
        (*time).clone()
    }

    pub async fn spin(&self) -> ! {
        let mut map = self.subscriptions.lock().await;

        let operation = self.rx.clone().map(Either::Left);

        let message = smol::stream::repeat_with(|| {
            let conn = self.conn.clone();
            let res = conn.recv();
            res
        })
        .map(Either::Right);

        let mut combined = stream::select(operation, message);

        loop {
            let pi = combined.next().await.unwrap();
            match pi {
                Either::Left(Operation::Subscribe {
                    message,
                    backchannel,
                }) => {
                    let subs = map.entry(message).or_insert(Vec::with_capacity(1));
                    subs.push(Mailbag {
                        backchannel,
                        reccuring: true,
                    });
                }
                Either::Left(Operation::Send(message)) => {
                    let header = mavlink::MavHeader::default();
                    self.conn
                        .send(&header, &message)
                        .unwrap_or_else(|_| panic!("fuck here"));
                }
                Either::Right(Ok((_, MavMessage::HEARTBEAT(_)))) => {
                    *self.last_heartbeat.lock().await = Some(Instant::now());
                }
                Either::Right(Ok((_header, msg))) => {
                    map.entry(discriminant(&msg))
                        .or_insert(Vec::new())
                        .retain(|sub| {
                            smol::block_on(sub.backchannel.send(msg.clone()))
                                .expect("unable to do this");
                            sub.reccuring && !sub.backchannel.is_closed()
                        });
                }
                _ => {}
            }
        }
    }
}
