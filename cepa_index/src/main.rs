#[macro_use]
extern crate rocket;

use rocket::State;
use std::sync::Arc;
use std::sync::Mutex;

struct ControllerData {
    host_list: Vec<String>,
}

type ControllerDataPointer = Arc<Mutex<ControllerData>>;

#[launch]
fn rocket() -> _ {
    let data = Arc::new(Mutex::new(ControllerData {
        host_list: Vec::new(),
    }));

    rocket::build()
        .mount("/", routes![add, retrieve])
        .manage(data)
}

#[get("/")]
fn retrieve(state: &State<ControllerDataPointer>) -> String {
    let d = state.lock().unwrap();
    return d.host_list.join("\n");
}

#[get("/add/<host>/<pub_key>")]
fn add(state: &State<ControllerDataPointer>, host: &str, pub_key: &str) -> String {
    let mut d = state.lock().unwrap();
    let s = format!("{} {}", host, pub_key);
    d.host_list.push(s);
    return "OK".to_string();
}
