// See LICENSE file for copyright and license details.

use crate::core::conf::Config;
use crate::core::fs::FileSystem;
use crate::core::types::{MInt, Size2};
use crate::visualizer::context::Context;
use crate::visualizer::font_stash::FontStash;
use crate::visualizer::game_state_visualizer::GameStateVisualizer;
use crate::visualizer::menu_state_visualizer::MenuStateVisualizer;
use crate::visualizer::mgl;
use crate::visualizer::shader::Shader;
use crate::visualizer::state_visualizer::{StateVisualizer, StateChangeCommand};
use crate::visualizer::types::{ColorId, MFloat, MatId, ScreenPos, Time};
use cgmath::{Vector, Vector2};
use std::cell::RefCell;
use std::path::Path;
use std::sync::mpsc::Receiver;
use time::precise_time_ns;

type EventsReceiver = Receiver<(f64, glfw::WindowEvent)>;

pub struct Visualizer {
    visualizers: Vec<Box<dyn StateVisualizer + 'static>>,
    dtime: Time,
    last_time: Time,
    glfw: glfw::Glfw,
    events: EventsReceiver,
    context: Context,
    fs: FileSystem,
    should_close: bool,
}

fn create_win(glfw: &glfw::Glfw, win_size: Size2<MInt>) -> (glfw::Window, EventsReceiver) {
    let w = win_size.w as u32;
    let h = win_size.h as u32;
    let title = "Marauder";
    let flags = glfw::WindowMode::Windowed;
    glfw.create_window(w, h, title, flags).unwrap()
}

impl Visualizer {
    pub fn new() -> Visualizer {
        let fs = FileSystem::new();
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let config = Config::new(&fs.get(&Path::new("data/conf_visualizer.json")));
        let win_size: Size2<MInt> = serde_json::from_value(config.get("screen_size").clone()).unwrap();
        let (mut win, events) = create_win(&glfw, win_size);
        glfw.make_context_current(Some(&win));
        gl::load_with(|procname| win.get_proc_address(procname));
        mgl::init_opengl();
        mgl::set_viewport(win_size);
        win.set_all_polling(true);
        let font_size = config.get("font_size").as_f64().unwrap() as MFloat;
        let font_stash = FontStash::new(
            &fs.get(&Path::new("data/DroidSerif-Regular.ttf")),
            font_size,
        );
        let shader = Shader::new(
            &fs.get(&Path::new("data/normal.vs.glsl")),
            &fs.get(&Path::new("data/normal.fs.glsl")),
        );
        let mvp_mat_id = MatId {
            id: shader.get_uniform("mvp_mat"),
        };
        let basic_color_id = ColorId {
            id: shader.get_uniform("basic_color"),
        };
        let context = Context {
            win,
            win_size,
            config,
            mouse_pos: ScreenPos { v: Vector2::zero() },
            font_stash: RefCell::new(font_stash),
            shader,
            mvp_mat_id,
            basic_color_id,
        };
        let visualizer = Box::new(MenuStateVisualizer::new(&context));
        Visualizer {
            visualizers: vec![visualizer as Box<dyn StateVisualizer>],
            dtime: Time { n: 0 },
            last_time: Time {
                n: precise_time_ns(),
            },
            glfw,
            events,
            context,
            fs,
            should_close: false,
        }
    }

    fn get_events(&mut self) -> Vec<glfw::WindowEvent> {
        self.glfw.poll_events();
        let mut events = Vec::new();
        for (_, event) in glfw::flush_messages(&self.events) {
            events.push(event);
        }
        events
    }

    pub fn is_running(&self) -> bool {
        !self.should_close
    }

    fn handle_cmd(&mut self) {
        let cmd = match self.visualizers.last() {
            Some(visualizer) => visualizer.get_command(),
            None => panic!("No state visualizer"),
        };
        match cmd {
            Some(StateChangeCommand::StartGame) => {
                let visualizer = Box::new(GameStateVisualizer::new(&self.fs, &self.context));
                self.visualizers
                    .push(visualizer as Box<dyn StateVisualizer>);
            }
            Some(StateChangeCommand::EndGame) => {
                let _ = self.visualizers.pop();
            }
            Some(StateChangeCommand::QuitMenu) => {
                self.should_close = true;
            }
            None => {}
        }
    }

    // TODO: simplify
    pub fn tick(&mut self) {
        {
            let events = self.get_events();
            let visualizer = match self.visualizers.last_mut() {
                Some(visualizer) => visualizer,
                None => panic!("No state visualizer"),
            };
            for event in events.iter() {
                visualizer.handle_event(&self.context, event.clone());
                self.context.handle_event(event.clone());
            }
            visualizer.logic(&self.context);
            visualizer.draw(&mut self.context, self.dtime);
        }
        self.handle_cmd();
        self.update_time();
    }

    pub fn update_time(&mut self) {
        let time = precise_time_ns();
        self.dtime.n = time - self.last_time.n;
        self.last_time.n = time;
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
