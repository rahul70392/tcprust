use std::io;
use std::collections::HashMap;
use std::net::Ipv4Addr;

mod tcp;

struct Quad {
    src : (Ipv4Addr,u16),
    dst : (Ipv4Addr,u16),
}
fn main() -> io::Result<()>{
    let mut connections:HashMap<Quad,tcp::State> = Default::default();
    let nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun).expect("failed to create");
    let mut buf = [0u8;1504];
    loop{
        let nbytes = nic.recv(&mut buf[..])?;
        let _eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
        if eth_proto != 0x800 {
            //no ipv4
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                if iph.protocol() != 0x06 {
                    //not tcp
                    continue;
                }

                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + iph.slice().len()..]){
                    Ok(tcph) => {
                        let datai = 4 + iph.slice().len() + tcph.slice().len();
                        connections.entry(Quad {
                            src : (src, tcph.source_port()),
                            dst : (dst, tcph.destination_port()),
                        }).or_default().on_packet(iph, tcph, &buf[datai..]);
                        eprintln!(
                            "{} -> {} {}b of tcp to port {}",
                            src,
                            dst,
                            tcph.slice().len(),
                            tcph.destination_port()
                        );
                    }
                    Err(e) => {
                        eprintln!("ignoring weird tcp packet{:?}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("ignoring weird packet{:?}", e);
            }
        }
