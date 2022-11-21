import { WorkerMessage, WebSocketMessage } from "./pkg/wasm_sync_ws_client";

let ws: WebSocket;
let responseArray: Int32Array;

self.addEventListener("message", (event: MessageEvent) => {
    if (event.data.constructor == SharedArrayBuffer) {
        let message: SharedArrayBuffer = event.data;
        responseArray = new Int32Array(message);
    } else {
        let message: WorkerMessage = event.data;

        if (message.connect) {
            ws = new WebSocket(message.connect);
            ws.addEventListener("open", (event) => {
                console.log("Ws connected!");
                // notify wasm thread that connection is open
                self.postMessage("connected");
            }),
                ws.addEventListener("close", (event) => {
                    console.log("Ws closed!");
                    // notify wasm thread that connection is closed
                    self.postMessage("closed");
                }),
                ws.addEventListener(
                    "message",
                    (event: MessageEvent<string | Uint8Array>) => {
                        if (event.data.constructor == Uint8Array) {
                            // received binary message
                            let bytes: Uint8Array = event.data;
                            storeBytesAndNotify(bytes);
                        } else if (event.data.constructor == String) {
                            // received text message
                            let message: string = event.data;
                            // encode message in utf8
                            let bytes = new TextEncoder().encode(message);
                            storeBytesAndNotify(bytes);
                        }
                    }
                ),
                console.log("Websocket created!");
        } else if (message.request) {
            let request: WebSocketMessage = message.request;
            if (request.binary) {
                if (ws.readyState == ws.OPEN) {
                    ws.send(request.binary);
                }
            } else if (request.text) {
                if (ws.readyState == ws.OPEN) {
                    ws.send(request.text);
                }
            }
        } else if (message.close) {
            console.log("Closing websocket...");
            ws.close();
        }
    }
});

function storeBytesAndNotify(bytes: Uint8Array) {
    for (let i = 0; i < responseArray.length; i++) {
        if (i < bytes.length) {
            // store response bytes in response array
            Atomics.store(responseArray, i, bytes[i]);
        } else {
            // store neutral value 256 in remaining response array slots
            Atomics.store(responseArray, i, 256);
        }
    }
    // notify wasm thread that response is ready
    Atomics.notify(responseArray, 0, 1);
}
