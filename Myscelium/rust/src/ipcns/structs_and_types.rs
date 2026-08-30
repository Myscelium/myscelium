// SPDX-License-Identifier: MPL-2.0
// Copyright © 2021-2026 Cristian Camargo Filho

use serde::{Deserialize, Serialize};
use OxidizedMyscelium::{CommandInstructions, DownCommand, SchedulingError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcnsError {
    Error(String),
    CannotObtainValidIpcnsAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OrderVariant {
    MatchParityId(String),
    ScheduleCommandInstructions(CommandInstructions, u8),
}

#[derive(Debug)]
pub enum StreamError {
    WriteError(std::io::Error),
    WriteSizeError(std::io::Error),
    ConnectionClosed,
    ReadSizeError(std::io::Error),
    ReadDataError(std::io::Error),
}

/// OrderResponse::Confirmed basically returns the parity id assigned to the command
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OrderResponse {
    MatchingDownCommand(DownCommand),
    Confirmed(String),
    Error(IpcnsError),
    IncplaceResponseNotArrivedYet,
}
