pub mod netbuf;
pub mod virtio_net;

pub use netbuf::NetBuf;
pub use virtio_net::{
    enqueue_rx_frame, enqueue_rx_frame_on, inject_rx_frame_for_test, poll_rx, VirtioNet,
};
