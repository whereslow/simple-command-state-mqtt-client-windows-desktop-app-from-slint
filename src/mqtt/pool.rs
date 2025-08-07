use crate::mqtt::connect::mqtt_connect_with_client_id;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, QoS};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

// 定义消息回调函数类型
type MessageCallback = Arc<dyn Fn(&str, &str) + Send + Sync + 'static>; // (topic, payload)

pub struct MqttConnection {
    client: AsyncClient,
    _eventloop_handle: tokio::task::JoinHandle<()>, // 保持EventLoop运行
    message_callbacks: Arc<Mutex<Vec<MessageCallback>>>, // 消息回调列表
}

impl MqttConnection {
    pub async fn new(host: &str, username: &str, password: &str, client_id: &str) -> Self {
        let (client, mut eventloop) =
            mqtt_connect_with_client_id(host, username, password, client_id).await;

        // 克隆client_id以便在异步闭包中使用
        let client_id_owned = client_id.to_string();

        // 创建消息回调列表
        let message_callbacks = Arc::new(Mutex::new(Vec::<MessageCallback>::new()));
        let callbacks_clone = message_callbacks.clone();

        // 启动EventLoop
        let eventloop_handle = tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Incoming::Publish(publish))) => {
                        // 处理接收到的消息
                        if let Ok(payload) = String::from_utf8(publish.payload.to_vec()) {
                            let topic = publish.topic.clone();

                            // 调用所有注册的回调函数
                            let callbacks = callbacks_clone.lock().await;
                            for callback in callbacks.iter() {
                                callback(&topic, &payload);
                            }
                        }
                    }
                    Ok(_) => {
                        // 处理其他事件
                    }
                    Err(e) => {
                        eprintln!("MQTT EventLoop错误 [{}]: {:?}", client_id_owned, e);
                        break;
                    }
                }
            }
        });

        // 等待连接建立
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Self {
            client,
            _eventloop_handle: eventloop_handle,
            message_callbacks,
        }
    }

    pub async fn publish(&self, topic: &str, message: &str) -> Result<(), rumqttc::ClientError> {
        // 先尝试AtLeastOnce，失败后降级到AtMostOnce
        match self
            .client
            .publish(topic, QoS::AtLeastOnce, false, message)
            .await
        {
            Ok(_) => {
                println!("消息发送成功: topic={}, message={}", topic, message);
                Ok(())
            }
            Err(e) => {
                eprintln!("AtLeastOnce发送失败，尝试AtMostOnce: {:?}", e);
                match self
                    .client
                    .publish(topic, QoS::AtMostOnce, false, message)
                    .await
                {
                    Ok(_) => {
                        println!("AtMostOnce发送成功: topic={}, message={}", topic, message);
                        Ok(())
                    }
                    Err(e2) => {
                        eprintln!("AtMostOnce也失败: {:?}", e2);
                        Err(e2)
                    }
                }
            }
        }
    }

    /// 订阅主题
    pub async fn subscribe(&self, topic: &str) -> Result<(), rumqttc::ClientError> {
        self.client.subscribe(topic, QoS::AtLeastOnce).await
    }

    /// 添加消息回调
    pub async fn add_message_callback<F>(&self, callback: F)
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        let mut callbacks = self.message_callbacks.lock().await;
        callbacks.push(Arc::new(callback));
    }
}

pub struct MqttConnectionPool {
    connections: Vec<Arc<Mutex<MqttConnection>>>,
    semaphore: Arc<Semaphore>,
    current_index: Arc<Mutex<usize>>,
}

impl MqttConnectionPool {
    pub async fn new(host: &str, username: &str, password: &str, pool_size: usize) -> Self {
        let mut connections = Vec::with_capacity(pool_size);

        for i in 0..pool_size {
            let client_id = format!("dt_client_{}", i);
            let connection = MqttConnection::new(host, username, password, &client_id).await;
            connections.push(Arc::new(Mutex::new(connection)));
        }

        Self {
            connections,
            semaphore: Arc::new(Semaphore::new(pool_size)),
            current_index: Arc::new(Mutex::new(0)),
        }
    }

    /// 获取一个连接进行发送（轮询策略）
    pub async fn send(&self, topic: &str, message: &str) -> Result<(), rumqttc::ClientError> {
        // 获取信号量许可
        let _permit = self.semaphore.acquire().await.unwrap();

        // 轮询选择连接
        let connection_arc = {
            let mut index = self.current_index.lock().await;
            let current = *index;
            *index = (current + 1) % self.connections.len();
            self.connections[current].clone()
        };

        // 使用连接发送消息
        let connection = connection_arc.lock().await;
        connection.publish(topic, message).await
    }

    /// 获取池大小
    pub fn size(&self) -> usize {
        self.connections.len()
    }

    /// 为receiver获取一个专用连接（不使用信号量限制）  
    pub async fn get_receiver_connection(&self) -> Arc<Mutex<MqttConnection>> {
        // 为receiver分配第一个连接
        self.connections[0].clone()
    }

    /// 创建一个新的专用receiver连接
    pub async fn create_receiver_connection(
        &self,
        host: &str,
        username: &str,
        password: &str,
    ) -> MqttConnection {
        let client_id = format!("dt_receiver_{}", std::process::id());
        MqttConnection::new(host, username, password, &client_id).await
    }
}
