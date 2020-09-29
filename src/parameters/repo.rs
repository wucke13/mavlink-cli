use crate::mavlink_stub::MavlinkConnectionHandler;
use std::fs::

struct ParameterRepo {
    conn: Arc<MavlinkConnectionHandler>
}

impl ParameterRepo {
    /// Creates new repo and returns instantly
    pub fn new(conn:Arc<MavlinkConnectionHandler>)->Self{
        unimplemented!();
    }



    pub async fn main_loop(&self) -> ! {
        unimplemented!();
    }
}
