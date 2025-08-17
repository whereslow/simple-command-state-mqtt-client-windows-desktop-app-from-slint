#![windows_subsystem = "windows"]

use crate::mqtt_handle::{POOL, STATE, init_mqtt};
use crate::node_state_entity::NodeCommand;
use crate::slint_handle::{COMMAND_NAME2COMMAND_DICT, COMMAND_NAME2TOPIC, open_command_set_window};
use event_listener::Event;
use once_cell::sync::Lazy;
use serde::Serialize;
use slint::{ModelRc, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use futures::executor::block_on;

struct ForceSend<T>(T);
unsafe impl<T> Send for ForceSend<T> {}

slint::include_modules!();

pub static UPDATE_STATE_EVENT: Lazy<Arc<Event>> = Lazy::new(|| Arc::new(Event::new()));
mod mqtt;
mod mqtt_handle;
mod node_state_entity;
mod slint_handle;

async fn update_state_handle(main_view: Weak<MainView>) {
    println!("update_state_handle");
    let event = UPDATE_STATE_EVENT.clone();
    loop {
        let listener = event.listen();
        listener.await;
        let current_state = STATE.lock().await;
        let mut insert_vec = Vec::new();
        current_state.iter().for_each(|(k, v)| {
            insert_vec.push((SharedString::from(k), SharedString::from(v)));
        });
        let main_view = main_view.clone();
        slint::invoke_from_event_loop(move || {
            main_view
                .unwrap()
                .set_node_states(ModelRc::from(Rc::new(VecModel::from(insert_vec))));
        })
        .expect("to slint thread失败");
    }
}
#[tokio::main]
async fn main() {
    let main_view = MainView::new().unwrap();
    let main_view_week = main_view.as_weak();
    main_view.on_open_command_set_window(move || open_command_set_window(main_view_week.clone()));
    main_view.on_run_command(|arg0: slint::SharedString| run_cmd(arg0.as_str()));
    tokio::spawn(update_state_handle(main_view.as_weak()));
    tokio::spawn(init_mqtt("127.0.0.1:1883", "server/0"));
    main_view.run().unwrap();
}
fn run_cmd(command_name: &str) {
    let c2t = COMMAND_NAME2TOPIC.lock().unwrap();
    let c2d = COMMAND_NAME2COMMAND_DICT.lock().unwrap();
    let topic = c2t.get(command_name).unwrap();
    let dict = c2d.get(command_name).unwrap();
    let resp_dict: HashMap<String, f64> = dict.iter().map(|(k,v)|{
        (k.to_string(),*v as f64)
    }).collect();
    let resp = NodeCommand::builder()
        .op(command_name.to_string())
        .op_value(resp_dict)
        .build();
    let text = serde_json::to_string(&resp).unwrap();
    block_on(POOL.get()
        .unwrap()
        .send(topic.as_str(), text.as_str())).unwrap();

}
