#[macro_use]
extern crate rocket;
extern crate cepa_common;

use cepa_common::{NodeData, NodeList, NodeListPointer};

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
        .mount("/", routes![get_index, add_node])
        .manage(data)
}

#[get("/")]
fn get_index(state: &State<NodeListPointer>) -> Json<NodeList> {
    match state.lock() {
        Ok(d) => {
            return Json(NodeList {
                timestamp: match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(n) => n.as_secs(),
                    Err(_) => panic!("SystemTime before UNIX EPOCH!"),
                },
                list: d.list.clone(),
            });
        }
        Err(_) => {
            return Json(NodeList {
                timestamp: (0),
                list: (Vec::new()),
            })
        }
    };
}

#[post("/", format = "json", data = "<data>")]
fn add_node(state: &State<NodeListPointer>, data: Json<NodeData>) -> String {
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
            return "OK".to_string();
        }
        Err(_) => return "NOK".to_string(),
    };
}
