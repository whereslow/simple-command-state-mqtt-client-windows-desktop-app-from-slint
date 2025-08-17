use crate::{CommandSetWindow, MainView};
use once_cell::sync::Lazy;

use native_dialog::{DialogBuilder, MessageLevel};
use slint::{ComponentHandle, Model, ModelRc, PhysicalPosition, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;

pub static COMMAND_NAME2COMMAND_DICT: Lazy<Mutex<HashMap<String, Vec<(String, f32)>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub static COMMAND_NAME2TOPIC: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub static COMMANDS: Lazy<Mutex<Vec<SharedString>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub fn show_windows_dialog(text: String, title: String) {
    thread::spawn(move || {
        DialogBuilder::message()
            .set_level(MessageLevel::Warning)
            .set_text(text.as_str())
            .set_title(title.as_str())
            .alert()
            .show()
            .unwrap();
    });
}
fn add_item_to_model(window: CommandSetWindow) {
    let data_model = window.get_data().clone();
    if let Some(vec_model) = data_model
        .as_any()
        .downcast_ref::<VecModel<(SharedString, f32)>>()
    {
        vec_model.push((SharedString::from(""), 0.0));
    } else {
        eprintln!("Failed to downcast to VecModel");
    }
}

fn add_command(
    data_model: ModelRc<(SharedString, f32)>,
    name: SharedString,
    topic: SharedString,
    main_view: Weak<MainView>,
) {
    let mut n2d = COMMAND_NAME2COMMAND_DICT.lock().unwrap();
    let mut n2t = COMMAND_NAME2TOPIC.lock().unwrap();
    let mut command_names = COMMANDS.lock().unwrap();
    let mut value_vec: Vec<(String, f32)> = vec![];
    if let Some(vec_model) = data_model
        .as_any()
        .downcast_ref::<VecModel<(SharedString, f32)>>()
    {
        vec_model.iter().for_each(|item| {
            value_vec.push((item.0.to_string(), item.1));
        });
        n2d.insert(name.to_string(), value_vec);
        n2t.insert(name.to_string(), topic.to_string());
        command_names.push(name);
        println!("{:?}", n2d);
        println!("{:?}",n2t);
        main_view.unwrap().set_commands(ModelRc::from(Rc::new(VecModel::from(command_names.clone()))));
    }
}
pub fn open_command_set_window(main_view: Weak<MainView>) {
    slint::invoke_from_event_loop(move || {
        let command_set_window = CommandSetWindow::new().unwrap();
        let window = command_set_window.clone_strong();
        command_set_window.on_add_item(move || {
            add_item_to_model(window.clone_strong());
        });
        let parent_pos = (&main_view).unwrap().window().position();
        command_set_window.window().set_position(PhysicalPosition::new(parent_pos.x+520,parent_pos.y+150));
        command_set_window.on_submit(move |d, n, t| add_command(d, n, t, main_view.clone()));
        command_set_window.show().unwrap();
    })
    .unwrap();
}
