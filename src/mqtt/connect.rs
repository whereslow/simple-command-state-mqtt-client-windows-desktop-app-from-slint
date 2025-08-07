use rumqttc::{AsyncClient, EventLoop, MqttOptions};
use std::time::Duration;

/// MQTT连接函数，返回客户端和事件循环
///
/// # 参数
/// * `url` - MQTT服务器地址
/// * `user` - 用户名
/// * `password` - 密码
///
/// # 返回值
/// 返回一个元组，包含 AsyncClient 和 EventLoop
pub async fn mqtt_connect(url: &str, user: &str, password: &str) -> (AsyncClient, EventLoop) {
    mqtt_connect_with_client_id(url, user, password, "dt_command_client").await
}

/// MQTT连接函数（带自定义客户端ID），返回客户端和事件循环
///
/// # 参数
/// * `url` - MQTT服务器地址
/// * `user` - 用户名  
/// * `password` - 密码
/// * `client_id` - 客户端ID
///
/// # 返回值
/// 返回一个元组，包含 AsyncClient 和 EventLoop
pub async fn mqtt_connect_with_client_id(
    url: &str,
    user: &str,
    password: &str,
    client_id: &str,
) -> (AsyncClient, EventLoop) {
    // 解析主机名和端口
    let (host, port) = parse_url(url);

    // 创建MQTT选项
    let mut mqtt_options = MqttOptions::new(client_id, host, port);

    // 配置连接参数
    mqtt_options
        .set_keep_alive(Duration::from_secs(2))
        .set_clean_session(true)
        .set_credentials(user.to_string(), password.to_string());

    // 创建客户端和事件循环
    AsyncClient::new(mqtt_options, 10)
}

/// 解析URL获取主机名和端口
fn parse_url(url: &str) -> (String, u16) {
    let parts: Vec<&str> = url.split(':').collect();
    match parts.len() {
        2 => {
            let host = parts[0].to_string();
            let port = parts[1].parse().unwrap_or(1883);
            (host, port)
        }
        _ => {
            // 如果URL格式不正确，使用默认端口
            (url.to_string(), 1883)
        }
    }
}
