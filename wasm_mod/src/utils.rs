// use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

#[allow(dead_code)]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// pub async fn sleep(duration: Duration) {
//     JsFuture::from(Promise::new(&mut |yes, _| {
//         window()
//             .unwrap()
//             .set_timeout_with_callback_and_timeout_and_arguments_0(
//                 &yes,
//                 duration.as_millis() as i32,
//             )
//             .unwrap();
//     }))
//     .await
//     .unwrap();
// }

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub(crate) async fn fetch_playlist(auth_header_value: &str, token_header_value: &str) {
    log!("fetch_playlist entered");
    // set request params
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    // the URL to call
    let url = "https://api-partner.spotify.com/pathfinder/v1/query?operationName=fetchPlaylist&variables=%7B%22uri%22%3A%22spotify%3Aplaylist%3A37i9dQZF1DWZeKCadgRdKQ%22%2C%22offset%22%3A0%2C%22limit%22%3A25%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%225534e86cc2181b9e70be86ae26d514abd8d828be2ee56e5f8b7882dd70204c62%22%7D%7D";

    // create the request
    let request = match Request::new_with_str_and_init(&url, &opts) {
        Ok(v) => v,
        Err(e) => {
            log!("Spotify request creation failed");
            log!("{:?}", e);
            return;
        }
    };

    log!("Request created");

    // add headers
    let _ = request.headers().set("Accept", "application/json");
    // these keys were manually extracted from chrome webdev tools
    let _ = request.headers().set("authorization", auth_header_value);
    let _ = request.headers().set("client-token", token_header_value);

    log!("Headers set");

    let worker_global_scope = js_sys::global().unchecked_into::<WorkerGlobalScope>();
    let resp_value = match JsFuture::from(worker_global_scope.fetch_with_request(&request)).await {
        Ok(v) => v,
        Err(e) => {
            log!("Spotify request failed");
            log!("{:?}", e);
            return;
        }
    };

    log!("HTTP request completed");

    // log the response, if possible
    if resp_value.is_instance_of::<Response>() {
        let resp: Response = resp_value.dyn_into().unwrap();

        // Convert this other `Promise` into a rust `Future`.
        let json = match resp.json() {
            Ok(v) => JsFuture::from(v).await,
            Err(e) => {
                log!("Cannot convert response to JSON");
                log!("{:?}", e);
                return;
            }
        };

        match json {
            Ok(v) => {
                log!("Response in JSON:");
                log!("{:?}", v);
            }
            Err(e) => {
                log!("The response returned an error");
                log!("{:?}", e);
            }
        };
    } else {
        log!("Spotify response in not Response");
        log!("{:?}", resp_value);
    };
}
