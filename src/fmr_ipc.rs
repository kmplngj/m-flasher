//! Bounded, typed privilege-helper frames for exactly one FMR1 operation.
//! This pure codec never opens files/devices or interprets arbitrary commands.
//! Image bytes still require the pinned SHA-256 check at both trust boundaries.

pub const FMR_CHECK: u8 = 5;
pub const IMAGE_LEN: usize = 1456;
pub const REQUEST_LEN: usize = 1479;
pub const MAX_BINDING_LEN: usize = 8192;
pub const MAX_RESPONSE_LEN: usize = 2 + MAX_BINDING_LEN + 512;

#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    pub location: u32,
    pub device: String,
    pub serial: String,
    pub image: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    pub first: [u8; 256],
    pub second: [u8; 256],
    pub binding: String,
}

fn selection(location: u32, device: &str, serial: &str) -> Result<(u8, u8), String> {
    let (bus, address) = device
        .strip_prefix("usb:")
        .and_then(|s| s.split_once('-'))
        .ok_or("FMR1 IPC device must be canonical usb:BUS-ADDRESS")?;
    let bus: u8 = bus.parse().map_err(|_| "FMR1 IPC invalid USB bus")?;
    let address: u8 = address
        .parse()
        .map_err(|_| "FMR1 IPC invalid USB address")?;
    if address == 0 || address > 127 || device != format!("usb:{bus}-{address}") {
        return Err("FMR1 IPC noncanonical or invalid USB device".into());
    }
    if location == 0 || (location >> 24) as u8 != bus {
        return Err("FMR1 IPC location does not match selected bus".into());
    }
    if serial.len() != 16 || !serial.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("FMR1 IPC stock serial must contain sixteen ASCII hex digits".into());
    }
    Ok((bus, address))
}

pub fn encode_request(request: &Request) -> Result<Vec<u8>, String> {
    let (bus, address) = selection(request.location, &request.device, &request.serial)?;
    if request.image.len() != IMAGE_LEN {
        return Err("FMR1 IPC image length mismatch".into());
    }
    let mut bytes = Vec::with_capacity(REQUEST_LEN);
    bytes.push(FMR_CHECK);
    bytes.extend_from_slice(&request.location.to_be_bytes());
    bytes.extend_from_slice(&[bus, address]);
    bytes.extend_from_slice(request.serial.to_ascii_uppercase().as_bytes());
    bytes.extend_from_slice(&request.image);
    Ok(bytes)
}

pub fn decode_request(bytes: &[u8]) -> Result<Request, String> {
    if bytes.len() != REQUEST_LEN || bytes[0] != FMR_CHECK {
        return Err("FMR1 IPC request must be exactly one fixed reader-check frame".into());
    }
    let location = u32::from_be_bytes(bytes[1..5].try_into().unwrap());
    let device = format!("usb:{}-{}", bytes[5], bytes[6]);
    let serial = std::str::from_utf8(&bytes[7..23])
        .map_err(|_| "FMR1 IPC serial is not ASCII hexadecimal")?
        .to_ascii_uppercase();
    selection(location, &device, &serial)?;
    Ok(Request {
        location,
        device,
        serial,
        image: bytes[23..].to_vec(),
    })
}

pub fn encode_response(response: &Response) -> Result<Vec<u8>, String> {
    if response.binding.len() > MAX_BINDING_LEN {
        return Err("FMR1 IPC binding exceeds the response bound".into());
    }
    if response.first != response.second {
        return Err("FMR1 IPC read samples differ".into());
    }
    let mut bytes = Vec::with_capacity(2 + response.binding.len() + 512);
    bytes.extend_from_slice(&(response.binding.len() as u16).to_le_bytes());
    bytes.extend_from_slice(response.binding.as_bytes());
    bytes.extend_from_slice(&response.first);
    bytes.extend_from_slice(&response.second);
    Ok(bytes)
}

pub fn decode_response(bytes: &[u8]) -> Result<Response, String> {
    let prefix = bytes.get(..2).ok_or("FMR1 IPC truncated response length")?;
    let binding_len = u16::from_le_bytes(prefix.try_into().unwrap()) as usize;
    if binding_len > MAX_BINDING_LEN || bytes.len() != 2 + binding_len + 512 {
        return Err("FMR1 IPC response length mismatch".into());
    }
    let binding = std::str::from_utf8(&bytes[2..2 + binding_len])
        .map_err(|_| "FMR1 IPC binding is not UTF-8")?
        .to_string();
    let first: [u8; 256] = bytes[2 + binding_len..258 + binding_len]
        .try_into()
        .unwrap();
    let second: [u8; 256] = bytes[258 + binding_len..].try_into().unwrap();
    if first != second {
        return Err("FMR1 IPC read samples differ".into());
    }
    Ok(Response {
        first,
        second,
        binding,
    })
}
