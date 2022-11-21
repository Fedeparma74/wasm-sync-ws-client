use js_sys::Uint8Array;
use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsStatus {
    Connecting,
    Open,
    Closed,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize)]
pub struct WorkerMessage {
    connect: Option<String>,
    request: Option<WebSocketMessage>,
    close: bool,
}

#[wasm_bindgen]
impl WorkerMessage {
    pub(crate) fn new_connect(message: String) -> WorkerMessage {
        WorkerMessage {
            connect: Some(message),
            request: None,
            close: false,
        }
    }
    pub(crate) fn new_request(message: WsMessage) -> WorkerMessage {
        WorkerMessage {
            connect: None,
            request: Some(message.into()),
            close: false,
        }
    }
    pub(crate) fn new_close() -> WorkerMessage {
        WorkerMessage {
            connect: None,
            request: None,
            close: true,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn connect(&self) -> Option<String> {
        self.connect.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn request(&self) -> JsValue {
        match self.request.as_ref() {
            Some(request) => request.to_owned().into(),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn close(&self) -> bool {
        self.close
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum WsMessage {
    Binary(Vec<u8>),
    Text(String),
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize)]
pub struct WebSocketMessage {
    binary: Option<Vec<u8>>,
    text: Option<String>,
}

#[wasm_bindgen]
impl WebSocketMessage {
    #[wasm_bindgen(getter)]
    pub fn binary(&self) -> Option<Uint8Array> {
        match &self.binary {
            Some(bytes) => {
                let bytes_array = Uint8Array::new_with_length(bytes.len() as u32);
                bytes_array.copy_from(bytes);
                // let blob = Blob::new_with_u8_array_sequence(&bytes_array).unwrap();
                Some(bytes_array)
            }
            None => None,
        }
    }
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> Option<String> {
        self.text.as_ref().cloned()
    }
}

impl From<WsMessage> for WebSocketMessage {
    fn from(message: WsMessage) -> Self {
        match message {
            WsMessage::Binary(bytes) => WebSocketMessage {
                binary: Some(bytes),
                text: None,
            },
            WsMessage::Text(text) => WebSocketMessage {
                binary: None,
                text: Some(text),
            },
        }
    }
}
