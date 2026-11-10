use crate::timecode::Sink;
use rosc::{OscMessage, OscPacket, OscType};
use selah_domain::{ChaseSource, CommandOrigin, PresentationCommand};
use std::net::UdpSocket;
pub fn parse(message: &OscMessage) -> Option<PresentationCommand> {
    let no_args = message.args.is_empty();
    match message.addr.as_str() {
        "/selah/next" if no_args => Some(PresentationCommand::Next),
        "/selah/prev" if no_args => Some(PresentationCommand::Prev),
        "/selah/clear" if no_args => Some(PresentationCommand::Clear),
        "/selah/chase/arm" => match message.args.as_slice() {
            [OscType::String(source)] => Some(PresentationCommand::ChaseArm {
                source: match source.as_str() {
                    "none" => ChaseSource::None,
                    "mtc" => ChaseSource::Mtc,
                    "ltc" => ChaseSource::Ltc,
                    _ => return None,
                },
            }),
            _ => None,
        },
        path if no_args => {
            let parts: Vec<_> = path.strip_prefix("/selah/go/")?.split('/').collect();
            if parts.len() != 2
                || parts[0].is_empty()
                || parts[1].is_empty()
                || !parts[1].bytes().all(|c| c.is_ascii_digit())
            {
                return None;
            }
            Some(PresentationCommand::Go {
                song_id: parts[0].into(),
                block_index: parts[1].parse().ok()?,
            })
        }
        _ => None,
    }
}
pub fn bind(port: u16) -> Result<UdpSocket, String> {
    let socket = UdpSocket::bind(("0.0.0.0", port)).map_err(|e| e.to_string())?;
    socket.set_nonblocking(true).map_err(|e| e.to_string())?;
    Ok(socket)
}
pub fn poll(socket: &UdpSocket, sink: &Sink) -> Result<(), String> {
    let mut data = [0u8; 65536];
    // Bound work per tick so a UDP flood cannot prevent shutdown/clock progress.
    for _ in 0..64 {
        match socket.recv(&mut data) {
            Ok(n) => {
                if let Ok(([], packet)) = rosc::decoder::decode_udp(&data[..n]) {
                    emit(packet, sink, 0);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => return Err(format!("OSC: {e}")),
        }
    }
    Ok(())
}
fn emit(packet: OscPacket, sink: &Sink, depth: usize) {
    if depth > 16 {
        return;
    }
    match packet {
        OscPacket::Message(message) => {
            if let Some(command) = parse(&message) {
                sink(command, CommandOrigin::Osc);
            }
        }
        // Only immediate bundles: never execute a future timetag early.
        OscPacket::Bundle(bundle) if bundle.timetag == (0, 1).into() => {
            for packet in bundle.content {
                emit(packet, sink, depth + 1);
            }
        }
        _ => {}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn udp_emits_command_and_origin() {
        let receiver = bind(0).unwrap();
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let packet = OscPacket::Message(OscMessage {
            addr: "/selah/next".into(),
            args: vec![],
        });
        let port = receiver.local_addr().unwrap().port();
        sender
            .send_to(
                &rosc::encoder::encode(&packet).unwrap(),
                ("127.0.0.1", port),
            )
            .unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let sink: Sink = std::sync::Arc::new(move |command, origin| {
            tx.send((command, origin)).unwrap();
        });
        for _ in 0..100 {
            poll(&receiver, &sink).unwrap();
            if let Ok(event) = rx.try_recv() {
                assert_eq!(event, (PresentationCommand::Next, CommandOrigin::Osc));
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        panic!("OSC event was not received");
    }
    #[test]
    fn routes_are_strict() {
        let message = |addr: &str| OscMessage {
            addr: addr.into(),
            args: vec![],
        };
        assert_eq!(
            parse(&message("/selah/next")),
            Some(PresentationCommand::Next)
        );
        assert_eq!(
            parse(&message("/selah/go/song/2")),
            Some(PresentationCommand::Go {
                song_id: "song".into(),
                block_index: 2
            })
        );
        for path in [
            "/selah/go/song/-1",
            "/selah/go//1",
            "/selah/go/song/1/x",
            "/next",
        ] {
            assert!(parse(&message(path)).is_none());
        }
        assert_eq!(
            parse(&OscMessage {
                addr: "/selah/chase/arm".into(),
                args: vec![OscType::String("ltc".into())]
            }),
            Some(PresentationCommand::ChaseArm {
                source: ChaseSource::Ltc
            })
        );
    }
}
