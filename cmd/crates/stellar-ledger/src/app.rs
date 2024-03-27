use byteorder::{BigEndian, WriteBytesExt};
use std::{io::Write, str::FromStr};

use ledger_transport::APDUCommand;
use ledger_transport_hid::{
    hidapi::{HidApi, HidError},
    LedgerHIDError, TransportNativeHID,
};

// these came from https://github.com/LedgerHQ/app-stellar/blob/develop/docs/COMMANDS.md
const CLA: u8 = 0xE0; // Instruction class
const GET_PUBLIC_KEY: u8 = 0x02; // Instruction code to get public key
const P1_GET_PUBLIC_KEY: u8 = 0x00;
const P2_GET_PUBLIC_KEY_NO_DISPLAY: u8 = 0x00;
const P2_GET_PUBLIC_KEY_DISPLAY: u8 = 0x01;

const RETURN_CODE_OK: u16 = 36864; // APDUAnswer.retcode which means success from Ledger

#[derive(Debug)]
pub enum LedgerError {
    /// Error occuring on init of hidapid and getting current devices list
    HidApiError(HidError),
    /// Error occuring on creating a new hid transport, connecting to first ledger device found  
    LedgerHidError(LedgerHIDError),
    /// Error occurred while exchanging with Ledger device
    APDUExchangeError(String),
    /// Error with transport
    LedgerHIDError(LedgerHIDError),
}

pub fn get_public_key(index: u32) -> Result<stellar_strkey::ed25519::PublicKey, LedgerError> {
    let hd_path = bip_path_from_index(index);
    get_public_key_with_display_flag(hd_path, false)
}

fn bip_path_from_index(index: u32) -> slip10::BIP32Path {
    let path = format!("m/44'/148'/{index}'");
    path.parse().unwrap() // this is basically the same thing as slip10::BIP32Path::from_str

    // the device handles this part: https://github.com/AhaLabs/rs-sep5/blob/9d6e3886b4b424dd7b730ec24c865f6fad5d770c/src/seed_phrase.rs#L86
}

fn hd_path_to_bytes(hd_path: &slip10::BIP32Path) -> Vec<u8> {
    println!("hd_path.depth: {:?}", hd_path.depth());
    (0..hd_path.depth())
        .map(|index| {
            let value = *hd_path.index(index).unwrap();
            value.to_be_bytes()
        })
        .flatten()
        .collect::<Vec<u8>>()
}

/// The display_and_confirm bool determines if the Ledger will display the public key on its screen and requires user approval to share
pub fn get_public_key_with_display_flag(
    hd_path: slip10::BIP32Path,
    display_and_confirm: bool,
) -> Result<stellar_strkey::ed25519::PublicKey, LedgerError> {
    // instantiate the connect to the Ledger, return an error if not connected
    let transport = get_transport()?;

    // convert the hd_path into bytes to be sent as `data` to the Ledger
    // the first element of the data should be the number of elements in the path
    let mut hd_path_to_bytes = hd_path_to_bytes(&hd_path);
    let hd_path_elements_count = hd_path.depth();
    hd_path_to_bytes.insert(0, hd_path_elements_count);

    let p2 = if display_and_confirm {
        P2_GET_PUBLIC_KEY_DISPLAY
    } else {
        P2_GET_PUBLIC_KEY_NO_DISPLAY
    };

    // more information about how to build this command can be found at https://github.com/LedgerHQ/app-stellar/blob/develop/docs/COMMANDS.md
    let command = APDUCommand {
        cla: CLA,
        ins: GET_PUBLIC_KEY,
        p1: P1_GET_PUBLIC_KEY,
        p2: p2,
        data: hd_path_to_bytes,
    };

    tracing::info!("APDU in: {}", hex::encode(&command.serialize()));

    match transport.exchange(&command) {
        Ok(response) => {
            tracing::info!(
                "APDU out: {}\nAPDU ret code: {:x}",
                hex::encode(response.apdu_data()),
                response.retcode(),
            );
            // Ok means we successfully connected with the Ledger but it doesn't mean our request succeeded. We still need to check the response.retcode
            if response.retcode() == RETURN_CODE_OK {
                return Ok(
                    stellar_strkey::ed25519::PublicKey::from_payload(&response.data()).unwrap(),
                );
            } else {
                let retcode = response.retcode();

                let error_string = format!("Ledger APDU retcode: 0x{:X}", retcode);
                return Err(LedgerError::APDUExchangeError(error_string));
            }
        }
        Err(err) => return Err(LedgerError::LedgerHIDError(err)),
    };
}

fn get_transport() -> Result<TransportNativeHID, LedgerError> {
    // instantiate the connection to Ledger, this will return an error if Ledger is not connected
    let hidapi = HidApi::new().map_err(LedgerError::HidApiError)?;
    TransportNativeHID::new(&hidapi).map_err(LedgerError::LedgerHidError)
}
