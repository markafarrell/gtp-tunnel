extern crate gtp;

extern crate clap;
use clap::{Arg, App, SubCommand};

use std::sync::Mutex;
use std::sync::Arc;
use std::sync::mpsc::{self, TryRecvError};

use std::{
    thread,
    time
};

use std::net::{
    IpAddr,
    Ipv4Addr,
    Ipv6Addr
};

use gtp::gtp_v1::listener_statistics::Statistics;
use gtp::gtp_v1::gtp_listener::GtpListener;
use gtp::gtp_v1::ip_listener::IpListener;

fn main() {

    let matches = App::new("My Super Program")
        .version("0.1")
        .author("Mark Farrell. <mark.andrew.farrell@gmail.com>")
        .about("Creates a GTP tunnel.")
        .arg(Arg::with_name("peer")
            .short("p")
            .long("peer")
            .help("IP Address of Peer")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("i_teid")
            .short("i")
            .long("i_teid")
            .help("TEID of incoming GTP packets")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("o_teid")
            .short("o")
            .long("o_teid")
            .help("TEID of outgoing GTP packets")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("src_ip")
            .short("s")
            .long("src_ip")
            .help("Source IP address of packets to be encapsulated in the tunnel.")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("dest_ip")
            .short("d")
            .long("dest_ip")
            .help("Destination IP address of packets to be encapsulated in the tunnel.")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("interface")
            .long("interface")
            .help("Interface to capture IP packets for encapsulation.")
            .takes_value(true)
            .required(false))
        .get_matches();
     

    let peer = matches.value_of("peer").unwrap().parse::<IpAddr>().expect("Peer IP address is invalid");

    let o_teid = matches.value_of("o_teid").unwrap().parse::<u32>().expect("o_teid is invalid");
    let i_teid = matches.value_of("i_teid").unwrap().parse::<u32>().expect("i_teid is invalid");

    // let filter = matches.value_of("filter").unwrap();

    let src_ip = matches.value_of("src_ip");
    let dest_ip = matches.value_of("dest_ip");

    let mut src_ip_address: Option<IpAddr> = None;
    let mut dest_ip_address: Option<IpAddr> = None;

    if src_ip.is_some() && dest_ip.is_some() {
        // Can only specify one of these options
        println!("Only one of src_ip and dest_ip can be specified.");
        return ();
    }
    else if let Some(src_ip) = src_ip {
        src_ip_address = Some(src_ip.parse::<IpAddr>().expect("Src IP address is invalid"));
    }
    else if let Some(dest_ip) = dest_ip {
        dest_ip_address = Some(dest_ip.parse::<IpAddr>().expect("Src IP address is invalid"));
    }
    else {
        println!("Either src_ip or dest_ip must be specified.");
        return ();
    }

    let interface_name = matches.value_of("interface").unwrap();

    println!("Starting Tunnel with {} for o_teid: {} i_teid: {} capturing packets from {}", peer, o_teid, i_teid, interface_name);

    let s = Statistics::new();
    let m = Mutex::new(s);
    let arc = Arc::new(m);

    let gtp_listener_thread = if let Some(g) = GtpListener::new(
        i_teid,
        o_teid,
        arc.clone()
    ) {
        thread::spawn(move || {
            println!("Spawned GTP Listener");
            g.listen();
        })
    }
    else {
        panic!("Could not start GTP listener");
    };

    let ip_listener_thread = if let Some(i) = IpListener::new(
        peer,
        o_teid,
        arc.clone(),
        interface_name,
        src_ip_address,
        dest_ip_address        
    ) {
        thread::spawn(move || {
            println!("Spawned IP Listener");
            i.listen();
        })
    }
    else {
        panic!("Could not start IP listener");
    };

    let (tx, rx) = mpsc::channel();
    
    let stats_thread = thread::spawn(move || {
        println!("Spawned Statistics Thread");
        loop {
            println!("{}", arc.clone().lock().unwrap());
            thread::sleep(time::Duration::from_millis(5000));
            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    println!("Terminating.");
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    });

    gtp_listener_thread.join().unwrap();
    ip_listener_thread.join().unwrap();

    let _ = tx.send(());

    stats_thread.join().unwrap();
}
