use smol::channel::{
    unbounded, Receiver, RecvError, SendError, Sender, TryRecvError, TrySendError,
};

/// Creates two interlocked `BiChannel` endpoints that send/receive to one another.
pub(crate) fn create_channel<TA, TB>() -> (BiChannel<TA, TB>, BiChannel<TB, TA>) {
    let (s1, r1) = unbounded();
    let (s2, r2) = unbounded();

    (BiChannel::new(s1, r2), BiChannel::new(s2, r1))
}

/// A bi-directional channel for sending/receiving
/// to/from the gui.
#[derive(Debug, Clone)]
pub(crate) struct BiChannel<TSend, TRecv> {
    s: Sender<TSend>,
    r: Receiver<TRecv>,
}

impl<TSend, TRecv> BiChannel<TSend, TRecv> {
    pub fn new(s: Sender<TSend>, r: Receiver<TRecv>) -> Self {
        Self { s, r }
    }

    pub async fn send(&self, item: TSend) -> Result<(), SendError<TSend>> {
        self.s.send(item).await
    }

    pub fn send_blocking(&self, item: TSend) -> Result<(), SendError<TSend>> {
        smol::block_on(async { self.s.send(item).await })
    }

    pub fn try_send(&self, item: TSend) -> Result<(), TrySendError<TSend>> {
        self.s.try_send(item)
    }

    pub async fn recv(&self) -> Result<TRecv, RecvError> {
        self.r.recv().await
    }

    pub fn try_recv(&self) -> Result<TRecv, TryRecvError> {
        self.r.try_recv()
    }
}
