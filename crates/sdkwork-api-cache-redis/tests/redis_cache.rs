use sdkwork_api_cache_core::{CacheStore, CacheTag, DistributedLockStore};
use sdkwork_api_cache_redis::RedisCacheStore;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn redis_cache_round_trips_values_respects_ttl_and_invalidates_tags() {
    let server = FakeRedisServer::start();
    let cache = RedisCacheStore::connect(&server.url_with_db(2))
        .await
        .expect("redis cache store");

    cache
        .put(
            "routing",
            "candidate-a",
            b"provider-a".to_vec(),
            Some(30),
            &[CacheTag::new("policy:default")],
        )
        .await
        .unwrap();
    cache
        .put(
            "routing",
            "candidate-b",
            b"provider-b".to_vec(),
            None,
            &[CacheTag::new("policy:default")],
        )
        .await
        .unwrap();

    let immediate = cache
        .get("routing", "candidate-a")
        .await
        .unwrap()
        .expect("immediate cache entry");
    assert_eq!(immediate.value(), b"provider-a");

    sleep(Duration::from_millis(45)).await;
    let expired = cache.get("routing", "candidate-a").await.unwrap();
    assert!(expired.is_none());

    let removed = cache
        .invalidate_tag("routing", "policy:default")
        .await
        .unwrap();
    assert_eq!(removed, 1);
    assert!(cache.get("routing", "candidate-b").await.unwrap().is_none());
}

#[tokio::test]
async fn redis_cache_locking_requires_matching_owner_to_release() {
    let server = FakeRedisServer::start();
    let cache = RedisCacheStore::connect(&server.url_with_db(3))
        .await
        .expect("redis cache store");

    assert!(cache
        .try_acquire_lock("catalog-refresh", "worker-a", 50)
        .await
        .unwrap());
    assert!(!cache
        .try_acquire_lock("catalog-refresh", "worker-b", 50)
        .await
        .unwrap());
    assert!(!cache
        .release_lock("catalog-refresh", "worker-b")
        .await
        .unwrap());
    assert!(cache
        .release_lock("catalog-refresh", "worker-a")
        .await
        .unwrap());
}

#[derive(Default)]
struct FakeRedisState {
    databases: HashMap<u32, FakeRedisDatabase>,
}

#[derive(Default)]
struct FakeRedisDatabase {
    strings: HashMap<Vec<u8>, FakeRedisStringValue>,
    sets: HashMap<Vec<u8>, HashSet<Vec<u8>>>,
}

struct FakeRedisStringValue {
    value: Vec<u8>,
    expires_at: Option<Instant>,
}

struct FakeRedisServer {
    address: String,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl FakeRedisServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake redis listener");
        listener
            .set_nonblocking(true)
            .expect("set nonblocking listener");
        let address = listener.local_addr().unwrap().to_string();
        let stop = Arc::new(AtomicBool::new(false));
        let state = Arc::new(Mutex::new(FakeRedisState::default()));
        let thread_stop = stop.clone();
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream.set_nonblocking(false).expect("set blocking stream");
                        handle_fake_redis_connection(stream, state.clone());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("fake redis accept failed: {error}"),
                }
            }
        });

        Self {
            address,
            stop,
            thread: Some(thread),
        }
    }

    fn url_with_db(&self, db: u32) -> String {
        format!("redis://{}/{db}", self.address)
    }
}

impl Drop for FakeRedisServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(&self.address);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("join fake redis thread");
        }
    }
}

fn handle_fake_redis_connection(stream: TcpStream, state: Arc<Mutex<FakeRedisState>>) {
    let mut stream = stream;
    stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .expect("set read timeout");
    stream
        .set_write_timeout(Some(Duration::from_millis(250)))
        .expect("set write timeout");
    let mut selected_db = 0_u32;

    loop {
        let command = match read_resp_array(&mut stream) {
            Ok(Some(command)) => command,
            Ok(None) => break,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::UnexpectedEof
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(error) => panic!("fake redis command read failed: {error}"),
        };
        let response = execute_fake_redis_command(&state, &mut selected_db, &command);
        write_resp_value(&mut stream, response).expect("write fake redis response");
    }
}

fn execute_fake_redis_command(
    state: &Arc<Mutex<FakeRedisState>>,
    selected_db: &mut u32,
    command: &[Vec<u8>],
) -> RespValue {
    let name = String::from_utf8_lossy(&command[0]).to_ascii_uppercase();
    let mut state = state.lock().expect("fake redis state");
    let database = state.databases.entry(*selected_db).or_default();
    purge_expired_strings(database);

    match name.as_str() {
        "PING" => RespValue::Simple("PONG".to_owned()),
        "AUTH" => RespValue::Simple("OK".to_owned()),
        "SELECT" => {
            *selected_db = String::from_utf8_lossy(&command[1]).parse().unwrap();
            RespValue::Simple("OK".to_owned())
        }
        "GET" => {
            let key = &command[1];
            RespValue::Bulk(database.strings.get(key).map(|value| value.value.clone()))
        }
        "SET" => {
            let key = command[1].clone();
            let value = command[2].clone();
            let mut ttl_ms = None;
            let mut nx = false;
            let mut index = 3;
            while index < command.len() {
                match String::from_utf8_lossy(&command[index]).to_ascii_uppercase().as_str() {
                    "PX" => {
                        ttl_ms = Some(
                            String::from_utf8_lossy(&command[index + 1])
                                .parse::<u64>()
                                .unwrap(),
                        );
                        index += 2;
                    }
                    "NX" => {
                        nx = true;
                        index += 1;
                    }
                    other => panic!("unsupported fake redis SET option: {other}"),
                }
            }

            if nx && database.strings.contains_key(&key) {
                return RespValue::Bulk(None);
            }

            database.strings.insert(
                key,
                FakeRedisStringValue {
                    value,
                    expires_at: ttl_ms.map(|ttl_ms| Instant::now() + Duration::from_millis(ttl_ms)),
                },
            );
            RespValue::Simple("OK".to_owned())
        }
        "DEL" => {
            let mut removed = 0_i64;
            for key in &command[1..] {
                if database.strings.remove(key).is_some() {
                    removed += 1;
                }
                if database.sets.remove(key).is_some() {
                    removed += 1;
                }
            }
            RespValue::Integer(removed)
        }
        "SADD" => {
            let members = database.sets.entry(command[1].clone()).or_default();
            let mut added = 0_i64;
            for member in &command[2..] {
                if members.insert(member.clone()) {
                    added += 1;
                }
            }
            RespValue::Integer(added)
        }
        "SMEMBERS" => RespValue::Array(
            database
                .sets
                .get(&command[1])
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect(),
        ),
        "SREM" => {
            let mut removed = 0_i64;
            if let Some(members) = database.sets.get_mut(&command[1]) {
                for member in &command[2..] {
                    if members.remove(member) {
                        removed += 1;
                    }
                }
                if members.is_empty() {
                    database.sets.remove(&command[1]);
                }
            }
            RespValue::Integer(removed)
        }
        other => panic!("unsupported fake redis command: {other}"),
    }
}

fn purge_expired_strings(database: &mut FakeRedisDatabase) {
    let now = Instant::now();
    database
        .strings
        .retain(|_, value| value.expires_at.map(|expires_at| expires_at > now).unwrap_or(true));
}

fn read_resp_array(stream: &mut TcpStream) -> std::io::Result<Option<Vec<Vec<u8>>>> {
    let mut marker = [0_u8; 1];
    match stream.read_exact(&mut marker) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    if marker[0] != b'*' {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected RESP array",
        ));
    }
    let count = read_resp_line(stream)?
        .parse::<usize>()
        .expect("array length");
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bulk_marker = [0_u8; 1];
        stream.read_exact(&mut bulk_marker)?;
        if bulk_marker[0] != b'$' {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "expected RESP bulk string",
            ));
        }
        let length = read_resp_line(stream)?
            .parse::<usize>()
            .expect("bulk string length");
        let mut value = vec![0_u8; length];
        stream.read_exact(&mut value)?;
        let mut crlf = [0_u8; 2];
        stream.read_exact(&mut crlf)?;
        values.push(value);
    }
    Ok(Some(values))
}

fn read_resp_line(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte)?;
        if byte[0] == b'\r' {
            let mut newline = [0_u8; 1];
            stream.read_exact(&mut newline)?;
            if newline[0] != b'\n' {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid RESP line ending",
                ));
            }
            break;
        }
        bytes.push(byte[0]);
    }
    Ok(String::from_utf8(bytes).expect("utf8 resp line"))
}

enum RespValue {
    Simple(String),
    Integer(i64),
    Bulk(Option<Vec<u8>>),
    Array(Vec<Vec<u8>>),
}

fn write_resp_value(stream: &mut TcpStream, value: RespValue) -> std::io::Result<()> {
    match value {
        RespValue::Simple(value) => {
            stream.write_all(format!("+{value}\r\n").as_bytes())?;
        }
        RespValue::Integer(value) => {
            stream.write_all(format!(":{value}\r\n").as_bytes())?;
        }
        RespValue::Bulk(Some(value)) => {
            stream.write_all(format!("${}\r\n", value.len()).as_bytes())?;
            stream.write_all(&value)?;
            stream.write_all(b"\r\n")?;
        }
        RespValue::Bulk(None) => {
            stream.write_all(b"$-1\r\n")?;
        }
        RespValue::Array(values) => {
            stream.write_all(format!("*{}\r\n", values.len()).as_bytes())?;
            for value in values {
                stream.write_all(format!("${}\r\n", value.len()).as_bytes())?;
                stream.write_all(&value)?;
                stream.write_all(b"\r\n")?;
            }
        }
    }
    stream.flush()?;
    Ok(())
}
