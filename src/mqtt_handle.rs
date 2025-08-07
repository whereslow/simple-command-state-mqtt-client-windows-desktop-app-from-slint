use crate::mqtt::pool::MqttConnectionPool;
use crate::mqtt::receiver::{MQTTConfig, Receiver, mqtt_receiver_with_config};
use crate::node_state_entity::{ NodeStateChangeString, NodeStateRegister};
use anyhow::{Error, format_err};
use log::error;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub static TOPIC2RECEIVER: Lazy<Mutex<HashMap<String, Arc<Mutex<Receiver>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub static POOL: OnceLock<Arc<MqttConnectionPool>> = OnceLock::new();

pub static STATE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub static STATE_CHANGE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
// 暂时不内联 rmqtt
// pub struct BrokerConfigure {
//     pub class: String,
//     pub name: String,
//     pub addr: [u8; 4],
//     pub port: u16,
// }
// impl BrokerConfigure {
//     pub fn new(class: &str, name: &str, addr: [u8; 4], port: u16) -> Self {
//         BrokerConfigure {
//             class: String::from(class),
//             name: String::from(name),
//             addr,
//             port,
//         }
//     }
// }
// pub async fn run_mqtt_broker(configures: Vec<BrokerConfigure>) -> Result<(), Err> {
//     let scx = ServerContext::new().build().await;
//     let mut mqtt_server = MqttServer::new(scx);
//     for config in configures {
//         match config.class.as_str() {
//             "tcp" => {
//                 mqtt_server = mqtt_server.listener(
//                     Builder::new()
//                         .name(config.name.as_str())
//                         .laddr((config.addr, config.port).into())
//                         .bind()?
//                         .tcp()?,
//                 );
//             }
//             "ws" => {
//                 mqtt_server = mqtt_server.listener(
//                     Builder::new()
//                         .name(config.name.as_str())
//                         .laddr((config.addr, config.port).into())
//                         .bind()?
//                         .ws()?,
//                 );
//             }
//             _ => {}
//         }
//     }
//     mqtt_server.build().run().await?;
//     Ok(())
// }

pub async fn init_client_pool(name: String, size: usize) -> Result<(), Error> {
    POOL.set(Arc::new(
        MqttConnectionPool::new(name.as_str(), "", "", size).await,
    ))
    .map_err(|_| format_err!("Can't init MqttConnection pool"))
    .expect("cant format error");
    Ok(())
}

pub async fn send_mqtt_msg(topic: String, msg: String) -> Result<(), Error> {
    let pool = POOL.get().expect("Pool not initialized");
    pool.send(topic.as_str(), msg.as_str()).await?;
    Ok(())
}
pub async fn add_mqtt_receiver(server_url: &str, topic: &str) -> Result<(), Error> {
    let mqtt_client_cfg = MQTTConfig {
        host: server_url.to_string(),
        username: "".to_string(),
        password: "".to_string(),
    };
    let mut topic2receiver = TOPIC2RECEIVER.lock().unwrap();
    topic2receiver.insert(
        topic.to_string(),
        Arc::new(Mutex::new(
            mqtt_receiver_with_config(&mqtt_client_cfg, topic).await,
        )),
    );
    Ok(())
}

pub async fn add_state_receiver(server_url: &str, topic: &str) -> Result<(), Error> {
    let mqtt_client_cfg = MQTTConfig {
        host: server_url.to_string(),
        username: "".to_string(),
        password: "".to_string(),
    };
    let receiver = mqtt_receiver_with_config(&mqtt_client_cfg, topic).await;
    receiver.add_callback(|msg| {
        if let Ok(change_data) = serde_json::from_str::<NodeStateChangeString>(msg) {
            let mut state = STATE.lock().unwrap();
            change_data
                .state_change
                .iter()
                .for_each(|(k, v)| match state.get_mut(k) {
                    None => {
                        error!("State change {} not found", k)
                    }
                    Some(ori) => {
                        *ori = v.clone();
                    }
                });
            *(STATE_CHANGE.lock().unwrap()) = true;
        } else if let Ok(register) = serde_json::from_str::<NodeStateRegister>(msg) {
            let mut state = STATE.lock().unwrap();
            state.insert("id".to_string(), register.id);
            state.insert(
                "position_type".to_string(),
                register.position_type.to_string(),
            );
            state.insert(
                "position".to_string(),
                register.position.iter().fold(String::new(), |mut acc, s| {
                    acc.push_str(s.to_string().as_str());
                    acc
                }),
            );
            for (k, v) in register.state.iter() {
                state.insert(k.to_string(), v.to_string());
            }
            *(STATE_CHANGE.lock().unwrap()) = true;
        } else {
            error!("State change {} not found", msg)
        }
    });
    let mut topic2receiver = TOPIC2RECEIVER.lock().unwrap();
    topic2receiver.insert(topic.to_string(), Arc::new(Mutex::new(receiver)));
    Ok(())
}

#[tokio::test]
async fn test_mqtt_broker() {
    init_client_pool("127.0.0.1:1883".to_string(), 10)
        .await
        .unwrap();
    add_state_receiver("127.0.0.1:1883", "test/1")
        .await
        .unwrap();
    let top2r = TOPIC2RECEIVER.lock().unwrap();
    let receiver = top2r.get("test/1").unwrap();
    let mut receiver = receiver.lock().unwrap();
    receiver.add_callback(|msg| {
        println!("{:?}", msg);
    });
    receiver.start().await;
}
