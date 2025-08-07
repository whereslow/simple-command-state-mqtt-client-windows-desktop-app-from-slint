use crate::mqtt::connect::mqtt_connect;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, QoS};
use std::sync::{Arc, Mutex};
use std::thread;

// 定义回调函数类型
type Callback = Arc<dyn Fn(&str) + Send + Sync + 'static>;

pub struct MQTTConfig {
    pub host: String,
    pub username: String,
    pub password: String,
}

// 接收器结构体
pub struct Receiver {
    callbacks: Arc<Mutex<Vec<Callback>>>,
    client: AsyncClient,
    event_loop: EventLoop,
    topic: String,
}

impl Receiver {
    // 添加回调函数
    pub fn add_callback<F>(&self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.push(Arc::new(callback));
    }

    // 处理接收到的消息
    fn handle_message(&self, payload: &str) {
        // 获取所有回调的克隆
        let callbacks = self
            .callbacks
            .lock()
            .unwrap()
            .iter()
            .map(|cb| Arc::clone(cb))
            .collect::<Vec<_>>();

        let mut handles = vec![];

        // 为每个回调创建新线程
        for callback in callbacks {
            let payload = payload.to_string();
            handles.push(thread::spawn(move || {
                callback(&payload);
            }));
        }

        // 等待所有线程完成
        for handle in handles {
            let _ = handle.join();
        }
    }

    // 启动事件循环 - 消费self但可以通过Arc共享
    pub async fn start(&mut self) {
        // 订阅主题
        if let Err(e) = self.client.subscribe(&self.topic, QoS::ExactlyOnce).await {
            eprintln!("订阅主题失败: {:?}", e);
        }

        loop {
            match self.event_loop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(publish))) => {
                    if let Ok(payload) = String::from_utf8(publish.payload.to_vec()) {
                        self.handle_message(&payload);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("事件循环错误: {:?}", e);
                    break;
                }
            }
        }
    }
}

// pub async fn mqtt_receiver(
//     mqtt_cfg: &MQTTConfig,
//     topic: &str,
// ) -> Receiver {
//     // 使用连接池的create_receiver_connection方法创建专用连接
//     // 这样避免了EventLoop冲突问题，每个receiver都有独立的连接和EventLoop
//     let (client, event_loop) = {
//         use crate::mqtt::connect::mqtt_connect_with_client_id;
// 
//         // 使用进程ID和随机数创建唯一的client_id
//         let client_id = format!(
//             "dt_receiver_{}_{}",
//             std::process::id(),
//             rand::random::<u32>()
//         );
//         mqtt_connect_with_client_id(
//             mqtt_cfg.host.as_str(),
//             mqtt_cfg.username.as_str(),
//             mqtt_cfg.password.as_str(),
//             &client_id,
//         )
//         .await
//     };
// 
//     let receiver = Receiver {
//         callbacks: Arc::new(Mutex::new(Vec::new())),
//         client: client.clone(),
//         event_loop,
//         topic: topic.to_string(),
//     };
// 
//     // 订阅指定主题
//     if let Err(e) = client.subscribe(topic, QoS::AtLeastOnce).await {
//         eprintln!("订阅主题失败: {:?}", e);
//     }
// 
//     receiver
// }

// 兼容旧版本的函数（使用配置创建连接）
pub async fn mqtt_receiver_with_config(mqtt_cfg: &MQTTConfig, topic: &str) -> Receiver {
    let (client, event_loop) = mqtt_connect(
        mqtt_cfg.host.as_str(),
        mqtt_cfg.username.as_str(),
        mqtt_cfg.password.as_str(),
    )
    .await;
    let receiver = Receiver {
        callbacks: Arc::new(Mutex::new(Vec::new())),
        client: client.clone(),
        event_loop,
        topic: topic.to_string(),
    };

    // 订阅指定主题
    if let Err(e) = client.subscribe(topic, QoS::ExactlyOnce).await {
        eprintln!("订阅主题失败: {:?}", e);
    }

    receiver
}
