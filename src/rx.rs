#![allow(unused)]

use std::ops::{Shl, Shr};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Error, Result};
use thiserror::Error;

use derivative::*;

#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum FeedbackId {
    BasicSensor = 1,
    DockingIR = 3,
    InertialSensor = 4,
    Cliff = 5,
    Current = 6,
    HardwareVersion = 10,
    FirmwareVersion = 11,
    RawDataOf3AxisGyro = 13,
    GeneralPurposeInput = 16,
    UniqueDeviceId = 19,
    ControllerInfo = 21,
}

// These can be used to get the size of payload
// Gyro sensor size can be 14 or 20 bytes
pub const FDB_SIZE_BASIC_SENSOR_DATA: u8 = 15;
pub const FDB_SIZE_DOCKING_IR: u8 = 3;
pub const FDB_SIZE_INERTIAL_SENSOR: u8 = 7;
pub const FDB_SIZE_CLIFF: u8 = 6;
pub const FDB_SIZE_CURRENT: u8 = 2;
pub const FDB_SIZE_HARDWARE_VERSION: u8 = 4;
pub const FDB_SIZE_FIRMWARE_VERSION: u8 = 4;
pub const FDB_SIZE_RAW_DATA_3_AXIS_GYRO_A: u8 = 14;
pub const FDB_SIZE_RAW_DATA_3_AXIS_GYRO_B: u8 = 20;
pub const FDB_SIZE_GENERAL_PURPOSE_OUTPUT: u8 = 16;
pub const FDB_SIZE_UNIQUE_DEVICE_IDENTIFIER: u8 = 12;
pub const FDB_SIZE_CONTROLLER_INFO: u8 = 13;

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Feedback {
    #[derivative(Default(value = "0"))]
    pub epoch_time_stamp: u128,
    pub basic_sensor: BasicSensor,
    pub docking_ir: DockingIR,
    pub inertial_sensor: InertialSensor,
    pub cliff: Cliff,
    pub current: Current,
    pub hardware_version: HardwareVersion,
    pub firmware_version: FirmwareVersion,
    pub gyro: Gyro,
    pub general_purpose_input: GeneralPurposeInput,
    pub unique_device_id: UniqueDeviceId,
    pub controller_info: ControllerInfo,
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct BasicSensor {
    pub valid: bool,
    pub time_stamp: u16,
    pub bumper: u8,
    pub wheel_drop: u8,
    pub cliff: u8,
    pub left_encoder: u16,
    pub right_encoder: u16,
    pub left_pwm: u8,
    pub right_pwm: u8,
    pub button: u8,
    pub charger: u8,
    pub battery: u8,
    pub overcurrent_flags: u8,
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct DockingIR {
    pub valid: bool,
    pub right_signal: u8,
    pub central_signal: u8,
    pub left_signal: u8,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct InertialSensor {
    pub valid: bool,
    pub angle: u16,
    pub angle_rate: u16,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Cliff {
    pub valid: bool,
    pub right_cliff_sensor: u16,
    pub central_cliff_sensor: u16,
    pub left_cliff_sensor: u16,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Current {
    pub valid: bool,
    pub left_motor: u8,
    pub right_motor: u8,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct HardwareVersion {
    pub valid: bool,
    pub patch: u8,
    pub minor: u8,
    pub major: u8,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct FirmwareVersion {
    pub valid: bool,
    pub patch: u8,
    pub minor: u8,
    pub major: u8,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct Gyro {
    pub valid: bool,
    pub frame_id: u8,
    pub followed_data_length: u8,
    pub raw_gyro_data: [RawGyro; 3],
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct RawGyro {
    pub valid: bool,
    pub x: u16,
    pub y: u16,
    pub z: u16,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct GeneralPurposeInput {
    pub valid: bool,
    pub d_ch0: u16,
    pub a_ch0: u16,
    pub a_ch1: u16,
    pub a_ch2: u16,
    pub a_ch3: u16,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct UniqueDeviceId {
    pub valid: bool,
    pub udid0: u32,
    pub udid1: u32,
    pub udid2: u32,
}
#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct ControllerInfo {
    pub valid: bool,
    pub is_user_configured: u8,
    pub p_gain: u32,
    pub i_gain: u32,
    pub d_gain: u32,
}

impl Feedback {
    pub fn new() -> Feedback {
        Feedback {
            epoch_time_stamp: 0,
            basic_sensor: BasicSensor::default(),
            docking_ir: DockingIR::default(),
            inertial_sensor: InertialSensor::default(),
            cliff: Cliff::default(),
            current: Current::default(),
            hardware_version: HardwareVersion::default(),
            firmware_version: FirmwareVersion::default(),
            gyro: Gyro::default(),
            general_purpose_input: GeneralPurposeInput::default(),
            unique_device_id: UniqueDeviceId::default(),
            controller_info: ControllerInfo::default(),
        }
    }
}

// decode buffer => packets => feedbacks
pub fn decode(buffer: &[u8], mut residue: &[u8]) -> Result<(Vec<Feedback>, Vec<u8>)> {
    // Check if the length if enough.
    // The min length is 70.
    let len = buffer.len();
    if len < 81 {
        return Err(anyhow!("Not enough data"));
    }

    let mut feedbacks = Vec::new();
    let mut new_residue = Vec::new();

    // Search for the preambles (0xaa, 0x55)
    let headers = search_header(&buffer[..len])?; // Headers not found

    // If the first preambles set is not located at 0 of the buffer, there is a broken packet
    // The residue from previous iteration and the broken packet
    // should be used to make a complete packet
    if headers[0] != 0 && residue.len() != 0 {
        let broken_packet = &buffer[..headers[0]];
        let merged_packet = merge_residue(&residue, broken_packet)?; // Merged failed
        let correct_crc = check_crc(&merged_packet);
        if correct_crc {
            let f = format_feedback(&merged_packet)?; // Formatting failed
            feedbacks.push(f);
        }
    }

    // Divide packets by header found
    let packets = divide_packet(&buffer[..len], &headers)?; // Packets not found
    for (i, packet) in packets.iter().enumerate() {
        // Check CRC and set the residue to pass to next iteration.
        let correct_crc = check_crc(&packet.clone());
        if correct_crc {
            let f = format_feedback(packet)?; // Formatting failed
            feedbacks.push(f);
            new_residue = [0u8; 0].to_vec(); // Clear so don't pass to the next iteration
        } else {
            new_residue = packet.clone(); // Pass to the next iteration
        }
    }
    return Ok((feedbacks, new_residue));
}

fn search_header(buffer: &[u8]) -> Result<Vec<usize>> {
    let mut h = Vec::new();
    let buf = buffer.iter();
    // Need to skip the first byte since this loop accesses [index-1]
    for (i, c) in buf.enumerate().skip(1) {
        if buffer[i - 1] == 0xaa && buffer[i] == 0x55 {
            h.push(i - 1);
        }
    }
    Ok(h)
}

fn merge_residue(residue: &[u8], broken_packet: &[u8]) -> Result<Vec<u8>> {
    let mut a = residue.clone().to_vec();
    let b = broken_packet.to_vec();
    a.extend(b);
    Ok(a)
}

fn divide_packet(buffer: &[u8], headers: &[usize]) -> Result<Vec<Vec<u8>>> {
    let mut packets = Vec::new();
    let mut start = 0;
    let mut end = 0;
    for (i, c) in headers.iter().enumerate() {
        // Use the indixes from h
        // Or use bytes until the end if currently last part
        if i + 1 != headers.len() {
            start = headers[i];
            end = headers[i + 1];
        } else {
            start = headers[i];
            end = buffer.len(); // Until the end => residue
        }
        let b = buffer[start..end].to_vec().clone();
        packets.push(b);
    }
    Ok(packets)
}

fn check_crc(packet: &Vec<u8>) -> bool {
    // Broken packet, most likely.
    let packet_len = packet.len();
    if packet_len < 81 {
        return false;
    }

    let last = packet.len() - 1;
    let checksum = packet.as_slice()[last];
    let mut acc: u8 = 0;

    for (i, c) in packet.iter().enumerate().skip(2) {
        if i == last {
            break;
        }
        acc = acc ^ c;
    }
    acc == checksum
}

fn format_feedback(packet: &Vec<u8>) -> Result<Feedback> {
    let total_len = packet[2].clone();
    let mut exit_count = 0;
    let mut index: u8 = 3; // assign the index of first ID of a feedback
    let mut f = Feedback::new();

    f.epoch_time_stamp = get_epoch_ms();

    loop {
        // There are only 11 entries in FeedBackId.
        exit_count += 1;
        if exit_count > 11 {
            break;
        }
        // To prevent Our of Range access
        if index >= total_len {
            break;
        }

        let id = num::FromPrimitive::from_u8(packet[index as usize]);
        match id {
            Some(FeedbackId::BasicSensor) => {
                f.basic_sensor.valid = true;
                f.basic_sensor.time_stamp = packet[2 + index as usize] as u16;
                f.basic_sensor.time_stamp |= (packet[3 + index as usize] as u16).shl(8);
                f.basic_sensor.bumper = packet[4 + index as usize];
                f.basic_sensor.wheel_drop = packet[5 + index as usize];
                f.basic_sensor.cliff = packet[6 + index as usize];
                f.basic_sensor.left_encoder = packet[7 + index as usize] as u16;
                f.basic_sensor.left_encoder |= (packet[8 + index as usize] as u16).shl(8);
                f.basic_sensor.right_encoder = packet[9 + index as usize] as u16;
                f.basic_sensor.right_encoder |= (packet[10 + index as usize] as u16).shl(8);
                f.basic_sensor.left_pwm = packet[11 + index as usize];
                f.basic_sensor.right_pwm = packet[12 + index as usize];
                f.basic_sensor.button = packet[13 + index as usize];
                f.basic_sensor.charger = packet[14 + index as usize];
                f.basic_sensor.battery = packet[15 + index as usize];
                f.basic_sensor.overcurrent_flags = packet[16 + index as usize];
                index += FDB_SIZE_BASIC_SENSOR_DATA + 2;
            }
            Some(FeedbackId::DockingIR) => {
                f.docking_ir.valid = true;
                f.docking_ir.right_signal = packet[2 + index as usize];
                f.docking_ir.central_signal = packet[3 + index as usize];
                f.docking_ir.left_signal = packet[4 + index as usize];
                index += FDB_SIZE_DOCKING_IR + 2;
            }
            Some(FeedbackId::InertialSensor) => {
                f.inertial_sensor.valid = true;
                f.inertial_sensor.angle = packet[2 + index as usize] as u16;
                f.inertial_sensor.angle |= (packet[3 + index as usize] as u16).shl(8);
                f.inertial_sensor.angle_rate = packet[4 + index as usize] as u16;
                f.inertial_sensor.angle_rate |= (packet[5 + index as usize] as u16).shl(8);
                index += FDB_SIZE_INERTIAL_SENSOR + 2;
            }
            Some(FeedbackId::Cliff) => {
                f.cliff.valid = true;
                f.cliff.right_cliff_sensor = packet[2 + index as usize] as u16;
                f.cliff.right_cliff_sensor |= (packet[3 + index as usize] as u16).shl(8);
                f.cliff.central_cliff_sensor = packet[4 + index as usize] as u16;
                f.cliff.central_cliff_sensor |= (packet[5 + index as usize] as u16).shl(8);
                f.cliff.left_cliff_sensor = packet[6 + index as usize] as u16;
                f.cliff.left_cliff_sensor |= (packet[7 + index as usize] as u16).shl(8);
                index += FDB_SIZE_CLIFF + 2;
            }
            Some(FeedbackId::Current) => {
                f.current.valid = true;
                f.current.left_motor = packet[2 + index as usize];
                f.current.right_motor = packet[3 + index as usize];
                index += FDB_SIZE_CURRENT + 2;
            }
            Some(FeedbackId::HardwareVersion) => {
                f.hardware_version.valid = true;
                f.hardware_version.patch = packet[2 + index as usize];
                f.hardware_version.minor = packet[3 + index as usize];
                f.hardware_version.major = packet[4 + index as usize];
                index += FDB_SIZE_HARDWARE_VERSION + 2;
            }
            Some(FeedbackId::FirmwareVersion) => {
                f.firmware_version.valid = true;
                f.firmware_version.patch = packet[2 + index as usize];
                f.firmware_version.minor = packet[3 + index as usize];
                f.firmware_version.major = packet[4 + index as usize];
                index += FDB_SIZE_FIRMWARE_VERSION + 2;
            }
            Some(FeedbackId::RawDataOf3AxisGyro) => {
                f.gyro.valid = true;
                f.gyro.frame_id = packet[2 + index as usize];
                f.gyro.followed_data_length = packet[3 + index as usize];
                //
                f.gyro.raw_gyro_data[0].x = packet[4 + index as usize] as u16;
                f.gyro.raw_gyro_data[0].x |= (packet[5 + index as usize] as u16).shl(8);
                f.gyro.raw_gyro_data[0].y = packet[6 + index as usize] as u16;
                f.gyro.raw_gyro_data[0].y |= (packet[7 + index as usize] as u16).shl(8);
                f.gyro.raw_gyro_data[0].z = packet[8 + index as usize] as u16;
                f.gyro.raw_gyro_data[0].z |= (packet[9 + index as usize] as u16).shl(8);
                //
                f.gyro.raw_gyro_data[1].x = packet[10 + index as usize] as u16;
                f.gyro.raw_gyro_data[1].x |= (packet[11 + index as usize] as u16).shl(8);
                f.gyro.raw_gyro_data[1].y = packet[12 + index as usize] as u16;
                f.gyro.raw_gyro_data[1].y |= (packet[13 + index as usize] as u16).shl(8);
                f.gyro.raw_gyro_data[1].z = packet[14 + index as usize] as u16;
                f.gyro.raw_gyro_data[1].z |= (packet[15 + index as usize] as u16).shl(8);
                //
                if packet[1 + index as usize] == FDB_SIZE_RAW_DATA_3_AXIS_GYRO_A {
                    index += FDB_SIZE_RAW_DATA_3_AXIS_GYRO_A + 2;
                } else if packet[1 + index as usize] == FDB_SIZE_RAW_DATA_3_AXIS_GYRO_B {
                    f.gyro.raw_gyro_data[2].x = packet[16 + index as usize] as u16;
                    f.gyro.raw_gyro_data[2].x |= (packet[17 + index as usize] as u16).shl(8);
                    f.gyro.raw_gyro_data[2].y = packet[18 + index as usize] as u16;
                    f.gyro.raw_gyro_data[2].y |= (packet[19 + index as usize] as u16).shl(8);
                    f.gyro.raw_gyro_data[2].z = packet[20 + index as usize] as u16;
                    f.gyro.raw_gyro_data[2].z |= (packet[21 + index as usize] as u16).shl(8);
                    index += FDB_SIZE_RAW_DATA_3_AXIS_GYRO_B + 2;
                }
            }
            Some(FeedbackId::GeneralPurposeInput) => {
                f.general_purpose_input.valid = true;
                f.general_purpose_input.d_ch0 = packet[2 + index as usize] as u16;
                f.general_purpose_input.d_ch0 |= (packet[3 + index as usize] as u16).shl(8);
                //
                f.general_purpose_input.a_ch0 = packet[4 + index as usize] as u16;
                f.general_purpose_input.a_ch0 |= (packet[5 + index as usize] as u16).shl(8);
                //
                f.general_purpose_input.a_ch1 = packet[6 + index as usize] as u16;
                f.general_purpose_input.a_ch1 |= (packet[7 + index as usize] as u16).shl(8);
                //
                f.general_purpose_input.a_ch2 = packet[8 + index as usize] as u16;
                f.general_purpose_input.a_ch2 |= (packet[9 + index as usize] as u16).shl(8);
                //
                f.general_purpose_input.a_ch3 = packet[10 + index as usize] as u16;
                f.general_purpose_input.a_ch3 |= (packet[11 + index as usize] as u16).shl(8);
                index += FDB_SIZE_GENERAL_PURPOSE_OUTPUT + 2;
            }
            Some(FeedbackId::UniqueDeviceId) => {
                f.unique_device_id.valid = true;
                f.unique_device_id.udid0 = packet[2 + index as usize] as u32;
                f.unique_device_id.udid0 |= (packet[3 + index as usize] as u32).shl(8);
                f.unique_device_id.udid0 |= (packet[4 + index as usize] as u32).shl(16);
                f.unique_device_id.udid0 |= (packet[5 + index as usize] as u32).shl(24);
                //
                f.unique_device_id.udid1 = packet[6 + index as usize] as u32;
                f.unique_device_id.udid1 |= (packet[7 + index as usize] as u32).shl(8);
                f.unique_device_id.udid1 |= (packet[8 + index as usize] as u32).shl(16);
                f.unique_device_id.udid1 |= (packet[9 + index as usize] as u32).shl(24);
                //
                f.unique_device_id.udid2 = packet[10 + index as usize] as u32;
                f.unique_device_id.udid2 |= (packet[11 + index as usize] as u32).shl(8);
                f.unique_device_id.udid2 |= (packet[12 + index as usize] as u32).shl(16);
                f.unique_device_id.udid2 |= (packet[13 + index as usize] as u32).shl(24);
                index += FDB_SIZE_UNIQUE_DEVICE_IDENTIFIER + 2;
            }
            Some(FeedbackId::ControllerInfo) => {
                f.controller_info.valid = true;
                f.controller_info.p_gain = packet[2 + index as usize] as u32;
                f.controller_info.p_gain |= (packet[3 + index as usize] as u32).shl(8);
                f.controller_info.p_gain |= (packet[4 + index as usize] as u32).shl(16);
                f.controller_info.p_gain |= (packet[5 + index as usize] as u32).shl(24);
                //
                f.controller_info.i_gain = packet[6 + index as usize] as u32;
                f.controller_info.i_gain |= (packet[7 + index as usize] as u32).shl(8);
                f.controller_info.i_gain |= (packet[8 + index as usize] as u32).shl(16);
                f.controller_info.i_gain |= (packet[9 + index as usize] as u32).shl(24);
                //
                f.controller_info.d_gain = packet[10 + index as usize] as u32;
                f.controller_info.d_gain |= (packet[11 + index as usize] as u32).shl(8);
                f.controller_info.d_gain |= (packet[12 + index as usize] as u32).shl(16);
                f.controller_info.d_gain |= (packet[13 + index as usize] as u32).shl(24);
                index += FDB_SIZE_CONTROLLER_INFO + 2;
            }
            _ => {
                // Nothing to do
            }
        }
    }
    Ok(f)
}

// https://stackoverflow.com/a/65051530
fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
