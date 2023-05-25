use cepa_common::{NodeData, NodeList, NodeListPointer};

use rocket::get;
use rocket::launch;
use rocket::post;
use rocket::routes;
use rocket::serde::json::Json;
use rocket::State;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[launch]
fn rocket() -> _ {
    let data = Arc::new(Mutex::new(NodeList {
        timestamp: 0,
        list: Vec::new(),
    }));

    rocket::build()
        .mount("/", routes![get_index, get_reset, add_node])
        .manage(data)
}

#[get("/")]
fn get_index(state: &State<NodeListPointer>) -> Json<NodeList> {
    match state.lock() {
        Ok(d) => Json(NodeList {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("SystemTime before UNIX EPOCH!")
                .as_secs(),

            list: d.list.clone(),
        }),
        Err(_) => Json(NodeList {
            timestamp: (0),
            list: (Vec::new()),
        }),
    }
}

#[get("/reset")]
fn get_reset(state: &State<NodeListPointer>) -> &str {
    state.lock().unwrap().list.clear();

    "Index has been reset"
}

#[post("/", format = "json", data = "<data>")]
fn add_node(state: &State<NodeListPointer>, data: Json<NodeData>) -> &str {
    println!(
        "DATA RECVD: [host: {}, pub_key: {}]",
        data.host, data.pub_key
    );

    match state.lock() {
        Ok(mut d) => {
            d.list.push(NodeData {
                host: (data.host.clone()),
                pub_key: (data.pub_key.clone()),
            });
            "OK"
        }
        Err(_) => "NOK",
    }
}
