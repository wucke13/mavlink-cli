use std::collections::HashMap;
use std::io;
pub use std::mem::{discriminant, Discriminant};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mavlink::{common::*, MavConnection, MavHeader};

use futures::{future::Either, prelude::*};
use smol::{
    channel::{self, Receiver, Sender},
    lock::Mutex,
};

/// Returns the `MavMessageType` of a `MavMessage`
pub use discriminant as message_type;

pub type MavMessageType = Discriminant<MavMessage>;

/// A async adapter for a MAVLink connection
///
/// Offers high level functionality to interact with a MAVLink vehicle in an async fashion.
pub struct MavlinkConnectionHandler {
    conn: Arc<dyn MavConnection<mavlink::common::MavMessage> + Sync + Send>,
    subscriptions: Mutex<HashMap<MavMessageType, Vec<Sender<MavMessage>>>>,
    tx: Sender<(MavMessageType, Sender<MavMessage>)>,
    rx: Receiver<(MavMessageType, Sender<MavMessage>)>,
    last_heartbeat: Mutex<Option<Instant>>,
}

// TODO make this failable if no heartbeat is received
impl MavlinkConnectionHandler {
    /// Construct a new MavlinkConnectionHandler
    ///
    /// # Arguments
    ///
    /// * `address` - MAVLink connection `&str`. Equivalent to the `address` argument in
    /// [mavlink::connect](https://docs.rs/mavlink/*/mavlink/fn.connect.html)
    ///
    /// # Examples
    ///
    /// ```
    /// use mavlink::common::MavMessage;
    /// use mavlink_stub::message_type;
    ///
    /// let conn = MavlinkConnectionHandler::new("serial:/dev/ttyACM0:115200")?;
    /// ```
    pub fn new(address: &str) -> io::Result<Self> {
        let mut conn = mavlink::connect::<MavMessage>(address)?;
        conn.set_protocol_version(mavlink::MavlinkVersion::V1);
        let conn = Arc::from(conn);
        let (tx, rx) = channel::unbounded();
        let subscriptions = Mutex::new(HashMap::new());
        let last_heartbeat = Mutex::new(None);
        Ok(Self {
            conn,
            subscriptions,
            tx,
            rx,
            last_heartbeat,
        })
    }

    /// Says whethe
    pub async fn is_alive(&self) -> io::Result<()> {
        let message_type = message_type(&MavMessage::HEARTBEAT(Default::default()));
        let ttl = Duration::from_secs(1);
        let err = io::Error::new(
            io::ErrorKind::TimedOut,
            format!("did not receive a HEARTBEAT signal in {:?}", ttl),
        );
        futures::select_biased! {
            _ = self.request(message_type).fuse() => Ok(()),
            _ = smol::Timer::after(ttl).fuse() => Err(err),
        }
    }

    /// Subscribe to all new MavMessages of the given MavMessageType
    ///
    /// This returns a never-ending Stream of MavMessages.
    ///
    /// # Arguments
    ///
    /// * `message_type` - `MavMessageType` of the desired messages
    ///
    /// # Examples
    ///
    /// ```
    /// let message_type = message_type(&MavMessage::PARAM_VALUE(Default::default())));
    ///
    /// let stream = conn.subscribe(message_type).await;
    ///
    /// for message in smol::stream::block_on(stream) {
    ///     if let MavMessage::PARAM_VALUE(data) = message {
    ///         // do something with `data`
    ///     }
    /// }
    /// ```

    pub async fn subscribe(
        &self,
        message_type: MavMessageType,
    ) -> Pin<Box<dyn Stream<Item = MavMessage>>> {
        let (tx, rx) = channel::unbounded();
        self.tx.send((message_type, tx)).await.unwrap(); // this may never fail
        Box::pin(rx)
    }

    /// Awaits the next MavMessage of the given MavMessageType
    ///
    /// # Arguments
    ///
    /// * `message_type` - `MavMessageType` of the desired messages
    ///
    /// # Examples
    ///
    /// ```
    /// let message_type = message_type(&MavMessage::PARAM_VALUE(Default::default())));
    ///
    /// if let MavMessage::PARAM_VALUE(data) = conn.request().await {
    ///     // do something with `data`
    /// }
    /// ```

    pub async fn request(&self, message_type: MavMessageType) -> MavMessage {
        let (tx, rx) = channel::unbounded();
        self.tx.send((message_type, tx)).await.unwrap(); //this may never fail
        rx.recv().map(|m| m.expect("Oh no!")).await
    }

    /// Send a `MavMessage` to the vehicle
    ///
    /// # Arguments
    ///
    /// * `message` - `MavMessage` to send
    ///
    /// # Examples
    ///
    /// ```
    /// let header = MavHeader::default();
    ///
    /// let message = MavMessage::PARAM_REQUEST_LIST(PARAM_REQUEST_LIST_DATA {
    ///     target_component: 0,
    ///     target_system: 0,
    /// };
    ///
    /// conn.send(&header, &message)?;
    /// ```
    pub fn send(&self, header: &MavHeader, message: &MavMessage) -> io::Result<()> {
        self.conn.send(header, message)
    }

    /// Send a `MavMessage` to the vehicle
    ///
    /// # Arguments
    ///
    /// * `message` - `MavMessage` to send
    ///
    /// # Examples
    ///
    /// ```
    /// let message = MavMessage::PARAM_REQUEST_LIST(PARAM_REQUEST_LIST_DATA {
    ///     target_component: 0,
    ///     target_system: 0,
    /// };
    ///
    /// conn.send_default(&message)?;
    /// ```
    pub fn send_default(&self, message: &MavMessage) -> io::Result<()> {
        self.conn.send_default(message)
    }

    /// Returns the `Instant` from the last received HEARTBEAT
    async fn last_heartbeat(&self) -> Option<Instant> {
        let time = self.last_heartbeat.lock().await;
        (*time).clone()
    }

    /// Starts the eventloop of MavlinkConnectionHandler
    ///
    /// May only be called once, will block on subsequent calls.
    /// Must be called in order for the MavlinkConnectionHandler to work.
    pub async fn main_loop(&self) -> ! {
        let mut map = self.subscriptions.lock().await;

        let operations = self.rx.clone().map(Either::Left);
        let messages = smol::stream::repeat_with(|| self.conn.recv()).map(Either::Right);
        let mut combined = stream::select(operations, messages);

        loop {
            match combined.next().await.unwrap() {
                Either::Left((message_type, backchannel)) => {
                    let subs = map.entry(message_type).or_insert(Vec::with_capacity(1));
                    subs.push(backchannel);
                }
                Either::Right(Ok((_, MavMessage::HEARTBEAT(_)))) => {
                    *self.last_heartbeat.lock().await = Some(Instant::now());
                }
                Either::Right(Ok((_header, msg))) => {
                    map.entry(discriminant(&msg))
                        .or_insert(Vec::new())
                        .retain(|backchannel| match backchannel.is_closed() {
                            true => false,
                            false => {
                                smol::block_on(backchannel.send(msg.clone()))
                                    .expect("unable to do this");
                                true
                            }
                        });
                }
                _ => {}
            }
        }
    }
}
