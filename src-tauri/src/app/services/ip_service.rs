use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::net::UdpSocket;

use tracing::error;

fn get_local_ip() -> Result<IpAddr, std::io::Error> {
    // Create a UDP socket and bind it to a local address (typically 0.0.0.0:0).
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // Connect the socket to a remote address. This will not initiate any network traffic.
    socket.connect("8.8.8.8:80")?;

    // Get the local socket address to retrieve the local IP address.
    let local_socket_addr = socket.local_addr()?;

    // Extract the IP address from the socket address.
    let local_ip = local_socket_addr.ip();

    Ok(local_ip)
}

#[tauri::command]
pub fn get_ipv4() -> Result<String, String> {
    let res = get_local_ip();
    match res {
        Ok(addr) => Ok(addr.to_string()),
        Err(e) => {
            error!("Error getting local IP address");
            Err(e.to_string())
        }
    }
}