#[macro_use]
extern crate log;

use crossbeam_channel::{unbounded, Receiver, SendError, Sender};
use std::{
    io::{self, BufRead, BufReader, BufWriter, Error, ErrorKind, Write},
    net::{Shutdown, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub mod stratum_data;

/// command send to the stratum server
#[derive(Debug)]
pub enum StratumCmd {
    Login {},
    SubmitShare { share: stratum_data::Share },
    KeepAlive { miner_id: String },
    Shutdown {},
}

/// something received from the stratum server
#[derive(Debug, Clone, PartialEq)]
pub enum StratumAction {
    Job {
        miner_id: String,
        seed_hash: String,
        blob: String,
        job_id: String,
        target: String,
    },
    Error {
        err: String,
    },
    Ok,
    KeepAliveOk,
}

pub struct StratumClient {
    command_sender: Sender<StratumCmd>,
    send_thread: thread::JoinHandle<()>,
    rcv_thread: thread::JoinHandle<()>,
    keep_alive_thread: thread::JoinHandle<()>,
    tcp_stream_hnd: TcpStream,
    tick_tx: Sender<()>,
}

/// All operation in the client are async
impl StratumClient {
    pub fn login(
        pool_conf: PoolConfig,
        err_receiver: Sender<Error>,
        action_rcv: Sender<StratumAction>,
    ) -> io::Result<StratumClient> {
        info!("connecting to address: {}", pool_conf.user);

        let (tcp_stream_hnd, reader, writer) = StratumClient::connect_tcp(&pool_conf.url)?;

        let miner_id = Arc::new(Mutex::new(Option::None));
        let (command_sender, command_receiver) = unbounded();

        let send_thread = StratumClient::start_send_thread(
            writer,
            command_receiver,
            pool_conf,
            err_receiver.clone(),
        )?;
        let rcv_thread = StratumClient::start_receive_thread(
            reader,
            action_rcv,
            miner_id.clone(),
            err_receiver,
        )?;
        let (keep_alive_thread, tick_tx) =
            StratumClient::start_keep_alive_thread(command_sender.clone(), miner_id)?;

        command_sender
            .send(StratumCmd::Login {})
            .expect("login command send");

        Ok(StratumClient {
            command_sender,
            send_thread,
            rcv_thread,
            keep_alive_thread,
            tcp_stream_hnd,
            tick_tx,
        })
    }

    fn connect_tcp(
        pool_address: &str,
    ) -> io::Result<(TcpStream, BufReader<TcpStream>, BufWriter<TcpStream>)> {
        let stream = TcpStream::connect(pool_address)?;
        stream.set_read_timeout(None)?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;

        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream.try_clone()?);

        Ok((stream, reader, writer))
    }

    fn start_send_thread(
        writer: BufWriter<TcpStream>,
        command_rcv: Receiver<StratumCmd>,
        pool_conf: PoolConfig,
        err_receiver: Sender<Error>,
    ) -> io::Result<thread::JoinHandle<()>> {
        thread::Builder::new()
            .name("Stratum send thread".to_string())
            .spawn(move || {
                let result = handle_stratum_send(&command_rcv, writer, &pool_conf);
                if result.is_err() {
                    err_receiver
                        .send(result.expect_err("result error send thread"))
                        .expect("sending error in send thread");
                }
                info!("stratum send thread ended");
            })
    }

    fn start_receive_thread(
        reader: BufReader<TcpStream>,
        action_rcv: Sender<StratumAction>,
        miner_id: Arc<Mutex<Option<String>>>,
        err_receiver: Sender<Error>,
    ) -> io::Result<thread::JoinHandle<()>> {
        thread::Builder::new()
            .name("Stratum receive thread".to_string())
            .spawn(move || {
                let result = handle_stratum_receive(reader, &action_rcv, &miner_id);
                if result.is_err() {
                    err_receiver
                        .send(result.expect_err("result error recv thread"))
                        .expect("sending error in recv thread");
                }
                info!("stratum receive thread ended");
            })
    }

    fn start_keep_alive_thread(
        cmd_alive: Sender<StratumCmd>,
        alive_miner_id: Arc<Mutex<Option<String>>>,
    ) -> io::Result<(thread::JoinHandle<()>, Sender<()>)> {
        let (stop_sndr, stop_rcvr) = unbounded();

        let (tick_rcv, _) = start_tick_thread(Duration::from_secs(60), stop_rcvr);
        Ok((
            thread::Builder::new()
                .name("keep alive thread".to_string())
                .spawn(move || {
                    loop {
                        let tick_result = tick_rcv.recv();
                        if tick_result.is_err() || tick_result.expect("tick result") == Tick::Stop {
                            break;
                        } //else: normal tick, loop around

                        let miner_id_guard = &*alive_miner_id.lock().expect("miner_id lock");
                        if miner_id_guard.is_some() {
                            let miner_id = miner_id_guard.clone().expect("miner_id clone");
                            cmd_alive
                                .send(StratumCmd::KeepAlive { miner_id })
                                .expect("KeepAlive send failed");
                        }
                    }
                    info!("keep alive thread ended");
                })?,
            stop_sndr,
        ))
    }

    /// Returns a new channel for sending commands to the stratum client
    pub fn new_cmd_channel(&self) -> Sender<StratumCmd> {
        self.command_sender.clone()
    }

    /// Stops the StratumClient, ending all communication with the server end.
    pub fn stop(self) {
        info!("stopping stratum client");

        //stop send thread
        self.command_sender
            .send(StratumCmd::Shutdown {})
            .expect("shutdown command send");

        //stop receive thread
        let shutdown_result = self.tcp_stream_hnd.shutdown(Shutdown::Both);
        if shutdown_result.is_err() {
            info!("TcpStream shutdown failed {:?}", shutdown_result);
        } else {
            info!("TcpStream shutdown ok");
        }

        //stop keep alive thread (via stopping tick thread)
        self.tick_tx.send(()).expect("ending tick thread");
        self.send_thread.join().expect("join send thread");
        self.rcv_thread.join().expect("join rcv thread");
        self.keep_alive_thread.join().expect("keep alive thread");
    }
}

#[derive(Debug, PartialEq)]
pub enum Tick {
    Tick,
    Stop,
}

pub fn start_tick_thread(
    interval: Duration,
    stop_rcv: Receiver<()>,
) -> (Receiver<Tick>, thread::JoinHandle<()>) {
    let (sndr, rcvr) = unbounded();
    let hnd = thread::Builder::new()
        .name("tick thread".to_string())
        .spawn(move || {
            loop {
                let result = stop_rcv.recv_timeout(interval);
                if result.is_err() {
                    //err means timeout reached and not a "normal" shutdown
                    let send_result = sndr.send(Tick::Tick);
                    if send_result.is_err() {
                        info!("sending tick signal failed {:?}", send_result);
                    }
                } else {
                    //shutdown received, end everything
                    let stop_send_result = sndr.send(Tick::Stop);
                    if stop_send_result.is_err() {
                        info!("sending tick stop signal failed, {:?}", stop_send_result);
                    }
                    break;
                }
            }
        })
        .expect("tick thread handle");
    (rcvr, hnd)
}

pub fn submit_share(
    tx: &Sender<StratumCmd>,
    share: stratum_data::Share,
) -> Result<(), SendError<StratumCmd>> {
    info!("submitting share: {:?}", share);
    tx.send(StratumCmd::SubmitShare { share })
}

fn handle_stratum_send(
    rx: &Receiver<StratumCmd>,
    mut writer: BufWriter<TcpStream>,
    pool_conf: &PoolConfig,
) -> Result<(), Error> {
    loop {
        match rx.recv().expect("stratum receiver") {
            StratumCmd::Login {} => do_stratum_login(&mut writer, pool_conf)?,
            StratumCmd::SubmitShare { share } => do_stratum_submit_share(&mut writer, share)?,
            StratumCmd::KeepAlive { miner_id } => do_stratum_keep_alive(&mut writer, miner_id)?,
            StratumCmd::Shutdown {} => {
                info!("stopping stratum send thread");
                break;
            }
        }
    }
    Ok(())
}

fn do_stratum_keep_alive(writer: &mut BufWriter<TcpStream>, miner_id: String) -> Result<(), Error> {
    let keep_alive_req = stratum_data::KeepAliveRequest {
        id: 1,
        method: "keepalived".to_string(),
        params: stratum_data::KeepAliveParams { id: miner_id },
    };

    let json = serde_json::to_string(&keep_alive_req).expect("marshaling keep alive json");
    writeln!(writer, "{}", json)?;
    writer.flush()?;
    Ok(())
}

fn do_stratum_submit_share(
    writer: &mut BufWriter<TcpStream>,
    share: stratum_data::Share,
) -> Result<(), Error> {
    let submit_req = stratum_data::SubmitRequest {
        id: 1,
        method: "submit".to_string(),
        params: stratum_data::SubmitParams {
            id: share.miner_id,
            job_id: share.job_id,
            nonce: share.nonce,
            result: share.hash,
        },
    };
    let json = serde_json::to_string(&submit_req).expect("marshaling submit json");
    writeln!(writer, "{}", json)?;
    writer.flush()?;
    Ok(())
}

fn do_stratum_login(
    writer: &mut BufWriter<TcpStream>,
    pool_conf: &PoolConfig,
) -> Result<(), Error> {
    let login_req = stratum_data::LoginRequest {
        id: 1,
        method: "login".to_string(),
        params: stratum_data::LoginParams {
            login: pool_conf.user.clone(),
            pass: pool_conf.pass.clone(),
        },
    };
    let json = serde_json::to_string(&login_req).expect("marshaling login json");
    writeln!(writer, "{}", json)?;
    writer.flush()?;
    Ok(())
}

fn handle_stratum_receive(
    mut reader: BufReader<TcpStream>,
    rcv: &Sender<StratumAction>,
    miner_id: &Arc<Mutex<Option<String>>>,
) -> Result<(), Error> {
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    //that means EOF in the TCPStream was reached
                    return Err(Error::new(ErrorKind::Other, "connection terminated"));
                }
                parse_line_dispatch_result(&line, rcv, miner_id);
            }
            Err(e) => {
                //read_line fails (maybe connection lost, dispatch err to channel)
                //=> Terminate loop
                return Err(e);
            }
        };
    }
}

fn is_known_ok(
    result: Result<stratum_data::OkResponse, serde_json::Error>,
) -> Option<StratumAction> {
    if let Ok(unwrapped) = result {
        if unwrapped.result.status == "OK" && unwrapped.result.id.is_none() {
            return Some(StratumAction::Ok);
        } else if unwrapped.result.status == "KEEPALIVED" && unwrapped.result.id.is_none() {
            return Some(StratumAction::KeepAliveOk);
        }
    }
    None
}

//TODO Refactor this method (it is very ugly) - its probably better to use generic value parsing and not using struct for every case
pub fn parse_line_dispatch_result(
    line: &str,
    rcv: &Sender<StratumAction>,
    miner_id_mutx: &Arc<Mutex<Option<String>>>,
) {
    let action;

    let error: Result<stratum_data::ErrorResult, serde_json::Error> = serde_json::from_str(line);
    if let Ok(error) = error {
        let stratum_data::ErrorResult { error: err_details } = error;
        action = StratumAction::Error {
            err: format!(
                "error received: {} (code {}, raw json {})",
                err_details.message, err_details.code, line
            ),
        };
    } else {
        let ok_result: Result<stratum_data::OkResponse, serde_json::Error> =
            serde_json::from_str(line);
        let known_ok = is_known_ok(ok_result);
        if let Some(known_ok) = known_ok {
            action = known_ok;
        } else {
            let result: Result<stratum_data::Method, serde_json::Error> =
                serde_json::from_str(line);
            if let Ok(result) = result {
                let stratum_data::Method { method } = result;
                match method.as_ref() {
                    "job" => action = parse_job(line, miner_id_mutx),
                    _ => {
                        action = StratumAction::Error {
                            err: format!("unknown method received: {}", method),
                        }
                    }
                };
            } else {
                //try parsing intial job
                let initial: Result<stratum_data::LoginResponse, serde_json::Error> =
                    serde_json::from_str(line);
                match initial {
                    Ok(stratum_data::LoginResponse {
                        result:
                            stratum_data::LoginResult {
                                status,
                                job:
                                    stratum_data::Job {
                                        seed_hash,
                                        blob,
                                        job_id,
                                        target,
                                    },
                                id: miner_id,
                            },
                        ..
                    }) => {
                        if status == "OK" {
                            action = StratumAction::Job {
                                miner_id: miner_id.clone(),
                                seed_hash,
                                blob,
                                job_id,
                                target,
                            };
                            let mut miner_id_guard = miner_id_mutx.lock().expect("miner_id lock");
                            *miner_id_guard = Option::Some(miner_id);
                        } else {
                            action = StratumAction::Error {
                                err: format!("Not OK initial job received, status was {}", status),
                            }
                        }
                    }
                    Err(e) => {
                        action = StratumAction::Error {
                            err: format!("{:?}, json received {}", e, line),
                        }
                    }
                }
            }
        }
    }

    let send_result = rcv.send(action);
    if send_result.is_err() {
        info!("sending action to receiver failed (receiver probably already terminated), trying next receiver");
    }
}

fn parse_job(line: &str, miner_id_mutx: &Arc<Mutex<Option<String>>>) -> StratumAction {
    let result: Result<stratum_data::JobResponse, serde_json::Error> = serde_json::from_str(line);
    let miner_id_guard = &*miner_id_mutx.lock().expect("miner_id lock");

    if miner_id_guard.is_none() {
        return StratumAction::Error{err: "miner_id not available for first mining job (login failed previously, this is a bug)".to_string()};
    }
    let miner_id = miner_id_guard.clone().expect("miner_id clone");

    match result {
        Ok(stratum_data::JobResponse {
            params:
                stratum_data::Job {
                    seed_hash,
                    blob,
                    job_id,
                    target,
                },
        }) => StratumAction::Job {
            miner_id,
            seed_hash,
            blob,
            job_id,
            target,
        },
        _ => StratumAction::Error {
            err: "Error parsing job response".to_string(),
        },
    }
}
