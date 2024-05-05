pub use accept::{accept_inet, accept_unix};
pub use bind::{bind_inet, bind_unix};
pub use connect::{connect_inet, connect_unix};
pub use listen::listen;

#[cfg(feature = "alloc")]
pub use socket::{get_inet_sock_name, get_unix_sock_name, recvmsg, sendmsg, socket};
#[cfg(not(feature = "alloc"))]
pub use socket::{get_inet_sock_name, get_unix_sock_name, socket};

mod accept;
mod bind;
mod connect;
mod listen;
mod socket;
#[cfg(all(test, feature = "alloc"))]
mod test;
