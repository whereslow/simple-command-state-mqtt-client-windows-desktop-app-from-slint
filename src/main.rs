#![windows_subsystem = "windows"]
use slint::{spawn_local, VecModel};

slint::include_modules!();
mod mqtt;
mod mqtt_handle;
mod node_state_entity;
mod slint_handle;


#[tokio::main]
async fn main() {
    let main_view = MainView::new().unwrap();
    main_view.on_open_command_set_window(move || {
        slint::invoke_from_event_loop(move || {
            let command_set_window = CommandSetWindow::new().unwrap();
            let window = command_set_window.clone_strong();
            command_set_window.on_add_item(move || {
                let data = window.get_data().clone();
                println!("{:?}",data);

            });
            command_set_window.show().unwrap();
        })
        .unwrap();
    });
    main_view.run().unwrap();
}
