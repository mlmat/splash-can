use gui;
use std::env;
use engine::VulkanEngine;
use cgci::Draw;

const APP_NAME: &str = "PaintApp";

fn main() {
    let validation_layers_env_var: String = env::var("VALIDATION_LAYERS").unwrap_or("0".to_string());
    let validation_layers = if validation_layers_env_var == "1".to_string() {
        true
    } else if validation_layers_env_var == "0".to_string() {
        false
    } else {
        panic!("Wrong value for VALIDATION_LAYERS environmental value")
    };
    let main_window = gui::MainWindow::new(APP_NAME, 800, 600);
    let engine: Box<dyn Draw> = Box::new(
        VulkanEngine::new(APP_NAME, validation_layers, &main_window)
    );
    println!("{}", main_window.get_details());
    gui::start_main_loop(main_window.event_loop, engine);
}