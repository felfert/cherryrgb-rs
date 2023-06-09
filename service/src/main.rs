use anyhow::{anyhow, Context, Result};
use cherryrgb::{self, CherryKeyboard, CustomKeyLeds, RpcAnimation, VirtKbd};
use clap::Parser;
use file_mode::ModePath;
use log::LevelFilter;
use nix::unistd::{chown, Group};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use systemd_journal_logger::{connected_to_journal, JournalLog};

mod service;
use service::Opt;
#[path = "../../src/common.rs"]
mod common;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

/// Handle a single connection from cherryrgb_ncli
/// Try to read command (and possible
/// serialized parameters) from stream, then
/// execute command and return the result.
fn handle_client(
    mut stream: UnixStream,
    keyboard: Arc<CherryKeyboard>,
    mutex: Arc<Mutex<u32>>,
) -> Result<()> {
    let mut msg = String::new();
    match stream.read_to_string(&mut msg) {
        Ok(res) => res,
        Err(err) => {
            log::error!("Errror while receiving cmd: {:?}", err);
            return Ok(());
        }
    };
    if msg.starts_with("debug=on") {
        log::set_max_level(LevelFilter::Debug);
        return Ok(());
    }
    if msg.starts_with("debug=off") {
        log::set_max_level(LevelFilter::Info);
        return Ok(());
    }
    // Not really useful at the moment, because
    // it just does logging, always returns an
    // empty Ok Result and is not really required
    // for LED-related operations.
    /*
    if msg.starts_with("fetch_device_state") {
        let _guard = mutex.lock().unwrap();
        match keyboard.fetch_device_state() {
            Ok(res) => res,
            Err(err) => {
                let emsg = format!("Fetching device state failed: {:?}", err);
                let _ = stream.write_all(emsg.as_bytes());
                _ = stream.flush();
                log::error!("{}", emsg);
                return Ok(());
            }
        }
        return Ok(());
    }
    */
    if msg.starts_with("reset_custom_colors") {
        let _guard = mutex.lock().unwrap();
        match keyboard.reset_custom_colors() {
            Ok(res) => res,
            Err(err) => {
                let emsg = format!("Errror in reset_custom_colors: {:?}", err);
                let _ = stream.write_all(format!("{}\n", emsg).as_bytes());
                log::error!("{}", emsg);
                return Ok(());
            }
        }
        return Ok(());
    }
    if let Some(stripped) = msg.strip_prefix("set_led_animation=") {
        let params = stripped;
        let args: RpcAnimation = match serde_json::from_str(params) {
            Ok(res) => res,
            Err(err) => {
                log::error!(
                    "Unable to deserialize params for set_led_animation {:?}",
                    err
                );
                return Ok(());
            }
        };
        let color = args.color.unwrap_or(rgb::RGB8::new(255, 255, 255).into());
        let _guard = mutex.lock().unwrap();
        match keyboard.set_led_animation(
            args.mode,
            args.brightness,
            args.speed,
            color,
            args.rainbow,
        ) {
            Ok(res) => res,
            Err(err) => {
                let emsg = format!("Errror in set_led_animation: {:?}", err);
                let _ = stream.write_all(emsg.as_bytes());
                log::error!("{}", emsg);
                return Ok(());
            }
        }
        return Ok(());
    }
    if let Some(stripped) = msg.strip_prefix("set_custom_colors=") {
        let params = stripped;
        let key_leds: CustomKeyLeds = match serde_json::from_str(params) {
            Ok(res) => res,
            Err(err) => {
                log::error!(
                    "Unable to deserialize params for set_custom_colors {:?}",
                    err
                );
                return Ok(());
            }
        };
        let _guard = mutex.lock().unwrap();
        match keyboard.set_custom_colors(key_leds) {
            Ok(res) => res,
            Err(err) => {
                let emsg = format!("Errror in set_set_custom_colors: {:?}", err);
                let _ = stream.write_all(emsg.as_bytes());
                log::error!("{}", emsg);
                return Ok(());
            }
        }
        return Ok(());
    }
    log::warn!("received invalid cmd: {:?}", msg.as_str().trim());
    Ok(())
}

fn socket_server(
    opt: Arc<Opt>,
    keep_running: Arc<AtomicBool>,
    keyboard: Arc<CherryKeyboard>,
    mutex: Arc<Mutex<u32>>,
) -> Result<()> {
    log::debug!("Listening on {}", opt.socket_path);
    let listener = UnixListener::bind(opt.socket_path.clone())?;
    let mode = u32::from_str_radix(&opt.socket_mode, 8).unwrap();
    let spath = Path::new(opt.socket_path.as_str());
    spath.set_mode(mode).unwrap();
    let group = Group::from_name(opt.socket_group.as_str())
        .unwrap()
        .unwrap();
    chown(spath, None, Some(group.gid)).unwrap();

    // accept connections and process them, spawning a new thread for each one
    log::debug!("Accept-loop");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // connection succeeded
                if keep_running.load(Ordering::SeqCst) {
                    log::debug!("Got connection on {}", opt.socket_path);
                    let keyboard_clone = Arc::clone(&keyboard);
                    let mutex_clone = Arc::clone(&mutex);
                    let tb = thread::Builder::new().name("handle_client".into());
                    tb.spawn(|| handle_client(stream, keyboard_clone, mutex_clone))
                        .unwrap();
                } else {
                    let _ = std::fs::remove_file(opt.socket_path.clone());
                    break;
                }
            }
            Err(err) => {
                log::error!("stream error err={:?}", err);
                // connection failed
                break;
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    if connected_to_journal() {
        // If the output streams of this process are directly connected to the
        // systemd journal log directly to the journal to preserve structured
        // log entries (e.g. proper multiline messages, metadata fields, etc.)
        JournalLog::default()
            .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
            .with_syslog_identifier("cherryrgb".to_string())
            .install()
            .unwrap();
    } else {
        simple_logger::init()?;
    }
    if opt.debug {
        log::set_max_level(LevelFilter::Debug);
    } else {
        log::set_max_level(LevelFilter::Info);
    }
    log::info!("{} {} starting", NAME, VERSION);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let aopt = Arc::new(opt.clone());
    // Mutex for accessing CherryKeyboard
    let amutex = Arc::new(Mutex::new(0));

    // Allow the usual hex specifiation (starting with 0x) for the product-id
    let pid = common::get_u16_from_string(opt.product_id);

    // Search / init usb keyboard
    let devices = match cherryrgb::find_devices(pid) {
        Err(_err) => {
            panic!("Failed to find any cherry keyboard");
        }
        Ok(devices) => devices,
    };

    if devices.len() > 1 {
        for (index, &dev) in devices.iter().enumerate() {
            println!("{}) VEN_ID={}, PROD_ID={}", index, dev.0, dev.1);
        }
        return Err(anyhow!(
            "More than one keyboard found, please provide --product-id"
        ));
    }

    let (vendor_id, product_id) = devices.first().unwrap().to_owned();
    let keyboard =
        CherryKeyboard::new(vendor_id, product_id).context("Failed to create keyboard")?;
    let mut vkb = VirtKbd::new();

    let aopt_clone = Arc::clone(&aopt);
    let akeyboard = Arc::new(keyboard);
    let akeyboard_clone = Arc::clone(&akeyboard);
    let server_running = Arc::clone(&running);
    let driver_running = Arc::clone(&running);
    let amutex_clone1 = Arc::clone(&amutex);
    let amutex_clone2 = Arc::clone(&amutex);
    let tb = thread::Builder::new().name("socket_server".into());
    let th = tb
        .spawn(|| socket_server(aopt_clone, server_running, akeyboard_clone, amutex_clone1))
        .unwrap();
    log::debug!("Entering driver loop");
    while driver_running.load(Ordering::SeqCst) {
        {
            let _guard = amutex_clone2.lock().unwrap();
            if let Err(err) = Arc::clone(&akeyboard).forward_filtered_keys(&mut vkb) {
                log::error!("Failed to forward key events, err={}", err);
                break;
            }
        }
        // Without this sleep, sometimes the mutex appears to be still locked
        // in the handle_client() above.
        thread::sleep(time::Duration::from_millis(100));
    }
    running.store(false, Ordering::SeqCst);
    // This triggers a break in the socket_server accept loop
    let _ = UnixStream::connect(opt.socket_path);
    _ = th.join();

    Ok(())
}
