use js_sys::{Atomics, Int32Array};
use wasm_bindgen::{JsError, JsValue};

pub(crate) fn decode_byte_response(response_array: &Int32Array) -> Result<String, JsValue> {
    let mut bytes_response: Vec<u8> = vec![];

    for i in 0..response_array.length() {
        let byte = Atomics::load(response_array, i)?;
        Atomics::store(response_array, i, 256)?;
        if byte == 256 {
            break;
        }
        bytes_response.push(byte as u8);
    }

    Ok(String::from_utf8(bytes_response).map_err(JsError::from)?)
}
