use rumqttc::{AsyncClient, QoS};
use tokio::runtime::Runtime;

/// 发送消息到指定主题
///
/// # 参数
/// * `client` - MQTT客户端
/// * `topic` - 目标主题
/// * `message` - 要发送的消息
pub async fn mqtt_send(client: &AsyncClient, topic: &str, message: &str) {
    if let Err(e) = client
        .publish(topic, QoS::ExactlyOnce, false, message)
        .await
    {
        eprintln!("发送消息失败: {:?}", e);
    }
}

/// 同步发送消息到指定主题
///
/// # 参数
/// * `client` - MQTT客户端
/// * `topic` - 目标主题
/// * `message` - 要发送的消息
pub fn mqtt_send_sync(client: &AsyncClient, topic: &str, message: &str) {
    // 创建一个运行时实例
    let rt = Runtime::new().unwrap();
    // 在运行时中执行异步发送
    rt.block_on(async { mqtt_send(client, topic, message).await });
}
