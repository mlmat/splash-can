use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use cgci::Draw;

pub fn start_main_loop(event_loop: EventLoop<()>, mut engine: Box<dyn Draw>) {
    event_loop.run(move |event, _, control_flow|{
        match event {
            Event::WindowEvent {event, ..} => {
                match event {
                    WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit }
                    _ => (),
                }
            },
            | Event::RedrawRequested(_window_id) => {
                engine.draw_frame();
            },
            | _ => (),
        }
    });
}

pub struct MainWindow<'prc> {
    window_title: &'prc str,
    pub window_width: u32,
    pub window_height: u32,
    pub window: winit::window::Window,
    pub event_loop: EventLoop<()>,
}

impl<'prc> MainWindow<'prc> {
    pub fn new(window_title: &'prc str, window_width: u32, window_height: u32) -> Self {
        let event_loop = EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height))
            .build(&event_loop)
            .expect("Failed to create main window!");
        
        Self {
            window_title: window_title,
            window_width: window_width,
            window_height: window_height,
            window: window,
            event_loop: event_loop,
        }
    }

    pub fn get_details(&self) -> String {
        format!("The window with title: {} and dimensions of {}x{} initiated",
            self.window_title,
            self.window_width,
            self.window_height)
    }
}

