use std::convert::TryFrom;

use anyhow::{anyhow, Context, Result};
use cherryrgb::{
    self,
    legacy::{self, LegacyKeyboard},
    read_color_profile, rgb, Brightness, CherryKeyboard, CustomKeyLeds, Keyboard,
};
use clap::Parser;

mod cli;
use cli::{CliCommand, Opt};
mod common;
mod profile;
mod state;

fn main() -> Result<()> {
    let opt = Opt::parse();

    // Allow the usual hex specifiation (starting with 0x) for the product-id
    let pid = common::get_u16_from_string(opt.product_id);

    // Search / init usb keyboard
    let devices = cherryrgb::find_devices(pid).context("Failed to find any cherry keyboard")?;

    if devices.len() > 1 {
        for (index, &dev) in devices.iter().enumerate() {
            println!("{}) VEN_ID={}, PROD_ID={}", index, dev.0, dev.1);
        }
        return Err(anyhow!(
            "More than one keyboard found, please provide --product-id"
        ));
    }

    let (vendor_id, product_id) = devices.first().unwrap().to_owned();

    let loglevel = if opt.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(loglevel)?;

    /* Fun begins */

    match Keyboard::open(vendor_id, product_id).context("Failed to create keyboard")? {
        Keyboard::Default(keyboard) => run(&keyboard, opt.command, opt.brightness),
        // Legacy protocol keyboards (e.g. MX BOARD 6.0 RGB, PID 0x00B8) speak a
        // completely different framebuffer streaming protocol
        Keyboard::Legacy(keyboard) => run_legacy(&keyboard, opt.command, opt.brightness),
    }
}

fn run(keyboard: &CherryKeyboard, command: CliCommand, brightness: Brightness) -> Result<()> {
    keyboard
        .fetch_device_state()
        .context("Fetching device state failed")?;

    match command {
        CliCommand::CustomColors(args) => {
            keyboard.reset_custom_colors()?;
            let mut keys = CustomKeyLeds::new();

            for (index, color) in args.colors.into_iter().enumerate() {
                keys.set_led(index, color)?;
            }

            keyboard.set_custom_colors(keys)?;
        }
        CliCommand::ColorProfileFile(args) => {
            let json = profile::read_profile_json(&args.file_path)?;
            let colors_from_file =
                read_color_profile(&json).context("reading colors from color file")?;

            if args.keep_existing {
                let keys = state::load()?
                    .modify_from(colors_from_file)
                    .context("assembling custom key leds")?;
                keyboard.set_custom_colors(keys.clone())?;
                state::save(keys)?;
            } else {
                let keys = CustomKeyLeds::try_from(colors_from_file)
                    .context("assembling custom key leds")?;
                keyboard.set_custom_colors(keys.clone())?;
                state::save(keys)?;
            }
        }
        CliCommand::Animation(args) => {
            let color = args.color.unwrap_or(rgb::RGB8::new(255, 255, 255).into());

            log::info!(
                "Setting: mode={:?} brightness={:?} speed={:?} color={:?}",
                args.mode,
                brightness,
                args.speed,
                color
            );

            keyboard
                .set_led_animation(args.mode, brightness, args.speed, color, args.rainbow)
                .context("Failed to set led animation")?;
        }
    }

    Ok(())
}

fn run_legacy(
    keyboard: &LegacyKeyboard,
    command: CliCommand,
    brightness: Brightness,
) -> Result<()> {
    let mut frame = [legacy::KeyColor::default(); legacy::TOTAL_KEYS];

    match command {
        CliCommand::Animation(args) => {
            let color = args.color.unwrap_or(rgb::RGB8::new(255, 255, 255).into());

            log::info!(
                "Setting: mode={:?} brightness={:?} speed={:?} color={:?}",
                args.mode,
                brightness,
                args.speed,
                color
            );

            keyboard
                .stream_animation(args.mode, brightness, args.speed, color, args.rainbow)
                .context("Failed to stream animation")?;
        }
        CliCommand::CustomColors(args) => {
            let bright = legacy::brightness_byte(brightness);
            for (key, color) in frame.iter_mut().zip(args.colors) {
                *key = legacy::KeyColor::new(bright, color);
            }

            keyboard
                .stream_static(&frame)
                .context("Failed to stream colors")?;
        }
        CliCommand::ColorProfileFile(args) => {
            let json = profile::read_profile_json(&args.file_path)?;
            let colors_from_file =
                read_color_profile(&json).context("reading colors from color file")?;

            let bright = legacy::brightness_byte(brightness);
            for key in colors_from_file {
                if let Some(slot) = frame.get_mut(key.key_index) {
                    *slot = legacy::KeyColor::new(bright, key.rgb_value);
                }
            }

            keyboard
                .stream_static(&frame)
                .context("Failed to stream colors")?;
        }
    }

    Ok(())
}
