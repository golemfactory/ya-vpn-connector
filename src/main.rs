pub mod cli_opts;
mod tap_codec;
mod packet_conv;

use crate::cli_opts::CliOptions;
use crate::tap_codec::{AnyPacket, AnyPacketCodec};
use actix::io::SinkWrite;
use actix::prelude::*;
use actix_codec::Framed;
use awc::ws;
use awc::{error::WsProtocolError, BoxedSocket};
use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use futures_util::stream::StreamExt;
use std::net::IpAddr;
use structopt::StructOpt;
use tun::{AsyncDevice};
use crate::packet_conv::{packet_ether_to_ip_slice, packet_ip_wrap_to_ether};

type WsFramedSink = SplitSink<Framed<BoxedSocket, ws::Codec>, ws::Message>;
type WsFramedStream = SplitStream<Framed<BoxedSocket, ws::Codec>>;
type TunFramedSink = SplitSink<tokio_util::codec::Framed<AsyncDevice, AnyPacketCodec>, AnyPacket>;
type TunFramedStream = SplitStream<tokio_util::codec::Framed<AsyncDevice, AnyPacketCodec>>;
type TapFramedSink = SplitSink<tokio_util::codec::Framed<AsyncDevice, AnyPacketCodec>, AnyPacket>;
type TapFramedStream = SplitStream<tokio_util::codec::Framed<AsyncDevice, AnyPacketCodec>>;

pub struct VpnWebSocket {
    ws_sink: SinkWrite<ws::Message, WsFramedSink>,
    tap_sink: Option<SinkWrite<AnyPacket, TapFramedSink>>,
    tun_sink: Option<SinkWrite<AnyPacket, TunFramedSink>>,
}

impl VpnWebSocket {
    pub fn start(
        ws_sink: WsFramedSink,
        ws_stream: WsFramedStream,
        tun_sink: Option<TunFramedSink>,
        tun_stream: Option<TunFramedStream>,
        tap_sink: Option<TapFramedSink>,
        tap_stream: Option<TapFramedStream>,
    ) -> Addr<Self> {
        VpnWebSocket::create(|ctx| {
            ctx.add_stream(ws_stream);
            if tap_sink.is_some() && tun_sink.is_some() {
                panic!("Pass TUN or TAP arguments, but not both");
            }

            if let (Some(tap_sink), Some(tap_stream)) = (tap_sink, tap_stream) {
                ctx.add_stream(tap_stream);
                VpnWebSocket {
                    ws_sink: SinkWrite::new(ws_sink, ctx),
                    tun_sink: None,
                    tap_sink: Some(SinkWrite::new(tap_sink, ctx)),
                }
            } else if let (Some(tun_sink), Some(tun_stream)) = (tun_sink, tun_stream) {
                ctx.add_stream(tun_stream);
                VpnWebSocket {
                    ws_sink: SinkWrite::new(ws_sink, ctx),
                    tun_sink: Some(SinkWrite::new(tun_sink, ctx)),
                    tap_sink: None,
                }
            } else {
                panic!("Pass TUN or TAP arguments");
            }
        })
    }
}

impl Actor for VpnWebSocket {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("VPN WebSocket: VPN connection started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        log::info!("VPN WebSocket: VPN connection stopped");
        ctx.stop();
    }
}

impl io::WriteHandler<WsProtocolError> for VpnWebSocket {}
impl io::WriteHandler<std::io::Error> for VpnWebSocket {}


impl StreamHandler<Result<ws::Frame, WsProtocolError>> for VpnWebSocket {
    fn handle(&mut self, msg: Result<ws::Frame, WsProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Frame::Text(_text)) => {
                log::error!("VPN WebSocket: Received text frame");
                ctx.stop();
            }
            Ok(ws::Frame::Binary(bytes)) => {
                if let Some(tun_sink) = self.tun_sink.as_mut() {
                    //log::trace!("Received Binary packet, sending to TUN...");

                    match packet_ether_to_ip_slice(bytes) {
                        Ok(ip_slice) => {
                            log::trace!("IP packet: {:?}", ip_slice);
                            if let Err(err) = tun_sink.write(AnyPacket::new_from_bytes(ip_slice)) {
                                log::error!("Error sending packet: {:?}", err);
                            }
                        }
                        Err(e) => {
                            log::error!("Error unwrapping packet: {:?}", e);
                        }
                    }
                } else {
                    //log::trace!("Received Binary packet, sending to TAP...");
                    if let Err(err) = self
                        .tap_sink
                        .as_mut()
                        .expect("tap sink has to be here")
                        .write(AnyPacket::new(bytes.to_vec()))
                    {
                        log::error!("Error sending packet to TUN: {:?}", err);
                        ctx.stop();
                    }
                }
            }
            Ok(ws::Frame::Ping(msg)) => {
                log::debug!("Received Ping Message, replying with pong...");
                if let Err(err) = self.ws_sink.write(ws::Message::Pong(msg)) {
                    log::error!("Error replying with pong: {:?}", err);
                    ctx.stop();
                }
            }
            Ok(ws::Frame::Pong(_)) => {
                log::debug!("Received Pong Message");
            }
            Ok(ws::Frame::Close(reason)) => {
                //ctx.close(reason);
                log::info!("Received Close Message: {:?}", reason);
                ctx.stop();
            }
            Ok(ws::Frame::Continuation(_)) => {
                //ignore
            }
            Err(err) => {
                log::error!("VPN WebSocket: protocol error: {:?}", err);
                ctx.stop();
            }
        }
    }
}

impl StreamHandler<Result<AnyPacket, std::io::Error>> for VpnWebSocket {
    fn handle(&mut self, msg: Result<AnyPacket, std::io::Error>, ctx: &mut Self::Context) {
        //self.heartbeat = Instant::now();
        match msg {
            Ok(packet) => {
                //log::trace!(
                //    "Received packet from TUN {:#?}",
                //    packet::ip::Packet::unchecked(packet.get_bytes())
                //);
                match packet_ip_wrap_to_ether(&packet.get_bytes(), None, None) {
                    Ok(ether_packet) => {
                        if let Err(err) = self
                            .ws_sink
                            .write(ws::Message::Binary(Bytes::from(ether_packet)))
                        {
                            log::error!("Error sending packet: {:?}", err);
                        }
                    }
                    Err(e) => {
                        log::error!("Error wrapping packet: {:?}", e);
                    }
                }
            }
            Err(err) => {
                log::error!("Tun io error: {:?}", err);
                ctx.stop();
            }
        }
    }
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG").unwrap_or("info".to_string()),
    );
    env_logger::init();
    //let routes_to_delete_later = iptables_route_to_interface("eth0", "vpn0")?;
    //iptables_cleanup(routes_to_delete_later)?;
    let opt: CliOptions = CliOptions::from_args();
    let app_key = std::env::var("YAGNA_APPKEY").expect("YAGNA_APPKEY not set");
    let (_tx, _rx) = std::sync::mpsc::channel::<bytes::Bytes>();
    //let connector = awc::Connector::new().ssl(ssl).finish();
    let (_req, ws_socket) = awc::Client::default()
        .ws(opt.websocket_address)
        .header("Authorization", format!("Bearer {app_key}"))
        .connect()
        .await?;

    let (ws_sink, ws_stream) = ws_socket.split();

    let addr = opt.vpn_network_addr.parse::<IpAddr>()?;
    let mask = opt.vpn_network_mask.parse::<IpAddr>()?;
    let mut config = tun::Configuration::default();
    let vpn_layer = match opt.vpn_layer.as_str() {
        "tun" => tun::Layer::L3,
        "tap" => tun::Layer::L2,
        _ => panic!("Invalid vpn layer"),
    };
    config
        .layer(vpn_layer)
        .address(addr)
        .netmask(mask)
        .name(opt.vpn_interface_name)
        .mtu(opt.vpn_mtu as i32)
        .up();

    let dev = tun::create_as_async(&config).unwrap();

    let ws_actor = if opt.vpn_layer == "tap" {
        let (tap_sink, tap_stream) =
            tokio_util::codec::Framed::new(dev, AnyPacketCodec::new()).split();
        VpnWebSocket::start(
            ws_sink,
            ws_stream,
            None,
            None,
            Some(tap_sink),
            Some(tap_stream),
        )
    } else {
        let (tun_sink, tun_stream) =    tokio_util::codec::Framed::new(dev, AnyPacketCodec::new()).split();
        VpnWebSocket::start(
            ws_sink,
            ws_stream,
            Some(tun_sink),
            Some(tun_stream),
            None,
            None,
        )
    };
    loop {
        if !ws_actor.connected() {
            log::info!("Actor stopped, exiting");
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs_f64(0.5)).await;
    }
    Ok(())
}
