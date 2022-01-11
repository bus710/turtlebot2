#![allow(unused)]

use std::ops::{Shl, Shr};

use anyhow::{anyhow, Result};
use thiserror::Error;

use derivative::*;

pub enum CommandId {
    BaseControl = 1,
    Sound = 3,
    SoundSequence = 4,
    RequestExtra = 9,
    GeneralPurposeOutput = 12,
    SetControllerGain = 13,
    GetControllerGain = 14,
}

// These can be used to set the length of command
// Total length = ID + Size + Payload + CRC
pub const CMD_LEN_BASE_CONTROL: u8 = 7;
pub const CMD_LEN_SOUND: u8 = 6;
pub const CMD_LEN_SOUND_SEQUENCE: u8 = 4;
pub const CMD_LEN_REQUEST_EXTRA: u8 = 5;
pub const CMD_LEN_GENERAL_PURPOSE_OUTPUT: u8 = 5;
pub const CMD_LEN_SET_CONTROLLER_GAIN: u8 = 16;
pub const CMD_LEN_GET_CONTROLLER_GAIN: u8 = 4;

// These can be used to set the size of payload
pub const CMD_SIZE_BASE_CONTROL: u8 = 4;
pub const CMD_SIZE_SOUND: u8 = 3;
pub const CMD_SIZE_SOUND_SEQUENCE: u8 = 1;
pub const CMD_SIZE_REQUEST_EXTRA: u8 = 2;
pub const CMD_SIZE_GENERAL_PURPOSE_OUTPUT: u8 = 2;
pub const CMD_SIZE_SET_CONTROLLER_GAIN: u8 = 13;
pub const CMD_SIZE_GET_CONTROLLER_GAIN: u8 = 1;

fn generate_crc(cmd: &[u8]) -> u8 {
    if cmd.len() < 5 {
        return 0;
    }
    let payload = cmd[5..].to_vec();
    let mut acc = 0;
    for (_, c) in payload.iter().enumerate() {
        acc = acc ^ *c;
    }
    acc
}

pub fn base_control_command(speed: u16, radius: u16) -> Result<Vec<u8>> {
    let mut cmd: Vec<u8> = Vec::new();
    cmd.push(0xaa);
    cmd.push(0x55);
    cmd.push(CMD_LEN_BASE_CONTROL);
    cmd.push(CommandId::BaseControl as u8);
    cmd.push(CMD_SIZE_BASE_CONTROL);
    cmd.push((speed & 0xff) as u8);
    cmd.push((speed & 0xff00).shr(8) as u8);
    cmd.push((radius & 0xff) as u8);
    cmd.push((radius & 0xff00).shr(8) as u8);
    let crc = generate_crc(&cmd.clone());
    cmd.push(crc);
    Ok(cmd)
}

pub fn sound_command(freq: u8, amp: u8, duration: u8) -> Result<Vec<u8>> {
    if freq == 0 || amp == 0 || duration == 0 {
        return Err(anyhow!(""));
    }
    let tmp: u16 = (1 / (freq * amp)) as u16;

    let mut cmd: Vec<u8> = Vec::new();
    cmd.push(0xaa);
    cmd.push(0x55);
    cmd.push(CMD_LEN_SOUND);
    cmd.push(CommandId::Sound as u8);
    cmd.push(CMD_SIZE_SOUND);
    cmd.push((tmp & 0xff) as u8);
    cmd.push((tmp & 0xff00).shr(8) as u8);
    cmd.push(duration);
    let crc = generate_crc(&cmd.clone());
    cmd.push(crc);
    Ok(cmd)
}

pub fn sound_sequence_command(seq: u8) -> Result<Vec<u8>> {
    let mut cmd: Vec<u8> = Vec::new();
    cmd.push(0xaa);
    cmd.push(0x55);
    cmd.push(CMD_LEN_SOUND_SEQUENCE);
    cmd.push(CommandId::SoundSequence as u8);
    cmd.push(CMD_SIZE_SOUND_SEQUENCE);
    cmd.push(seq);
    let crc = generate_crc(&cmd.clone());
    cmd.push(crc);
    Ok(cmd)
}

pub fn request_extra_command(hw_ver: bool, fw_ver: bool, udid: bool) -> Result<Vec<u8>> {
    let mut tmp: u8 = 0;
    tmp |= hw_ver as u8;
    tmp |= (fw_ver as u8).shl(1);
    tmp |= (udid as u8).shl(7);

    let mut cmd: Vec<u8> = Vec::new();
    cmd.push(0xaa);
    cmd.push(0x55);
    cmd.push(CMD_LEN_REQUEST_EXTRA);
    cmd.push(CommandId::RequestExtra as u8);
    cmd.push(CMD_SIZE_REQUEST_EXTRA);
    cmd.push(tmp);
    let crc = generate_crc(&cmd.clone());
    cmd.push(crc);
    Ok(cmd)
}

pub fn general_purpose_output_command(
    d_out_ch0: bool,
    d_out_ch1: bool,
    d_out_ch2: bool,
    d_out_ch3: bool,
    power_3v3: bool,
    power_5v0: bool,
    power_12v5a: bool,
    power_12v1a5: bool,
    red_led1: bool,
    red_led2: bool,
    green_led1: bool,
    green_led2: bool,
) -> Result<Vec<u8>> {
    let mut tmp0: u8 = 0;
    tmp0 |= d_out_ch0 as u8;
    tmp0 |= (d_out_ch1 as u8).shl(1);
    tmp0 |= (d_out_ch2 as u8).shl(2);
    tmp0 |= (d_out_ch3 as u8).shl(3);
    tmp0 |= (power_3v3 as u8).shl(4);
    tmp0 |= (power_5v0 as u8).shl(5);
    tmp0 |= (power_12v5a as u8).shl(6);
    tmp0 |= (power_12v1a5 as u8).shl(7);
    let mut tmp1: u8 = 0;
    tmp1 |= red_led1 as u8;
    tmp1 |= (green_led1 as u8).shl(1);
    tmp1 |= (red_led2 as u8).shl(2);
    tmp1 |= (green_led2 as u8).shl(3);

    let mut cmd: Vec<u8> = Vec::new();
    cmd.push(0xaa);
    cmd.push(0x55);
    cmd.push(CMD_LEN_REQUEST_EXTRA);
    cmd.push(CommandId::RequestExtra as u8);
    cmd.push(CMD_SIZE_REQUEST_EXTRA);
    cmd.push(tmp0);
    cmd.push(tmp1);
    let crc = generate_crc(&cmd.clone());
    cmd.push(crc);
    Ok(cmd)
}

pub fn set_controller_gain(is_user_configured: bool, p: u32, i: f32, d: u32) -> Result<Vec<u8>> {
    let mut pp = if p == 0 { 1000 } else { p * 1000 };
    let mut ii = if i < 0.1 || i > 32000.0 {
        (0.1 * 1000.0) as u32
    } else {
        (i * 1000.0) as u32
    };
    let mut dd = if d == 0 { 2 * 1000 } else { d * 1000 };

    let mut cmd: Vec<u8> = Vec::new();
    cmd.push(0xaa);
    cmd.push(0x55);
    cmd.push(CMD_LEN_SET_CONTROLLER_GAIN);
    cmd.push(CommandId::SetControllerGain as u8);
    cmd.push(CMD_SIZE_SET_CONTROLLER_GAIN);
    cmd.push(is_user_configured as u8);
    cmd.push((pp & 0x000000ff) as u8);
    cmd.push((pp & 0x0000ff00).shr(8) as u8);
    cmd.push((pp & 0x00ff0000).shr(16) as u8);
    cmd.push((pp & 0xff000000).shr(24) as u8);
    cmd.push((ii & 0x000000ff) as u8);
    cmd.push((ii & 0x0000ff00).shr(8) as u8);
    cmd.push((ii & 0x00ff0000).shr(16) as u8);
    cmd.push((ii & 0xff000000).shr(24) as u8);
    cmd.push((dd & 0x000000ff) as u8);
    cmd.push((dd & 0x0000ff00).shr(8) as u8);
    cmd.push((dd & 0x00ff0000).shr(16) as u8);
    cmd.push((dd & 0xff000000).shr(24) as u8);
    let crc = generate_crc(&cmd.clone());
    cmd.push(crc);
    Ok(cmd)
}

pub fn get_controller_gain() -> Result<Vec<u8>> {
    let mut cmd: Vec<u8> = Vec::new();
    cmd.push(0xaa);
    cmd.push(0x55);
    cmd.push(CMD_LEN_GET_CONTROLLER_GAIN);
    cmd.push(CommandId::GetControllerGain as u8);
    cmd.push(CMD_SIZE_GET_CONTROLLER_GAIN);
    cmd.push(0xff);
    let crc = generate_crc(&cmd.clone());
    cmd.push(crc);
    Ok(cmd)
}
