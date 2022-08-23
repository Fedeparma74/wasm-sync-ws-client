mod types;
mod utils;

use js_sys::{Atomics, Int32Array, SharedArrayBuffer};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{MessageEvent, Worker, WorkerOptions, WorkerType};

pub use types::{WorkerMessage, WsMessage, WsStatus};
use utils::decode_byte_response;

#[wasm_bindgen]
pub struct WsClient {
    ws_worker: Worker,
    response_array: Int32Array,
    ws_status: Rc<RefCell<WsStatus>>,
    timeout_millis: Option<u32>,
}

#[wasm_bindgen]
impl WsClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String, timeout_millis: Option<u32>) -> Result<WsClient, JsValue> {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let mut worker_options = WorkerOptions::new();
        worker_options.type_(WorkerType::Module);

        let ws_worker = Worker::new_with_options(
            // "../node_modules/wasm-sync-ws-client/ws-worker.ts",
            "./ws-worker.ts",
            &worker_options,
        )?;

        let max_response_bytes = 1024;
        let response_buffer = SharedArrayBuffer::new(4 * max_response_bytes);
        let response_array = Int32Array::new(&response_buffer);
        // fill response_array of '256' (max u8 value is 255, this allows us to discriminate the end of a message)
        for i in 0..response_array.length() {
            Atomics::store(&response_array, i, 256)?;
        }

        // init sharedArrayBuffer on the worker side
        ws_worker.post_message(&response_buffer)?;

        // connect to the websocket
        let msg = WorkerMessage::new_connect(url);
        ws_worker.post_message(&serde_wasm_bindgen::to_value(&msg)?)?;

        let ws_status = Rc::new(RefCell::new(WsStatus::Connecting));

        let ws_status_clone = ws_status.clone();
        let on_message_callback: Closure<dyn FnMut(MessageEvent)> =
            Closure::new(move |event: MessageEvent| {
                let message = match event.data().as_string() {
                    Some(message) => message,
                    None => {
                        return;
                    }
                };
                match message.as_str() {
                    "connected" => {
                        let mut ws_status = ws_status_clone.borrow_mut();
                        *ws_status = WsStatus::Open;
                        // TODO: maybe add request queue
                    }
                    "closed" => {
                        let mut ws_status = ws_status_clone.borrow_mut();
                        *ws_status = WsStatus::Closed;
                    }
                    _ => {}
                }
            });
        ws_worker.set_onmessage(Some(on_message_callback.as_ref().unchecked_ref()));
        on_message_callback.forget();

        let client = WsClient {
            ws_worker,
            response_array,
            ws_status,
            timeout_millis,
        };

        Ok(client)
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> WsStatus {
        self.ws_status.borrow().to_owned()
    }

    pub fn close(&self) -> Result<(), JsValue> {
        self.ws_worker
            .post_message(&serde_wasm_bindgen::to_value(&WorkerMessage::new_close())?)?;
        Ok(())
    }

    pub fn call_binary(&self, message: Vec<u8>) -> Result<String, JsValue> {
        self.flush_response()?;
        self.send_binary(message)?;
        self.recv()
    }

    pub fn call_text(&self, message: String) -> Result<String, JsValue> {
        self.flush_response()?;
        self.send_text(message)?;
        self.recv()
    }

    pub fn send_binary(&self, message: Vec<u8>) -> Result<(), JsValue> {
        self.ws_worker.post_message(&serde_wasm_bindgen::to_value(
            &WorkerMessage::new_request(WsMessage::Binary(message)),
        )?)?;

        Ok(())
    }

    pub fn send_text(&self, message: String) -> Result<(), JsValue> {
        self.ws_worker.post_message(&serde_wasm_bindgen::to_value(
            &WorkerMessage::new_request(WsMessage::Text(message)),
        )?)?;

        Ok(())
    }

    pub fn recv(&self) -> Result<String, JsValue> {
        if self.timeout_millis.is_none() {
            Atomics::wait(&self.response_array, 0, 256)?;
        } else {
            let result = Atomics::wait_with_timeout(
                &self.response_array,
                0,
                256,
                self.timeout_millis.unwrap().into(),
            )?;
            if result.as_string().unwrap() == "timed-out" {
                return Err(JsValue::from_str("Request timed out"));
            }
        }

        decode_byte_response(&self.response_array)
    }

    pub fn flush_response(&self) -> Result<(), JsValue> {
        for i in 0..self.response_array.length() {
            Atomics::store(&self.response_array, i, 256)?;
        }
        Ok(())
    }
}
