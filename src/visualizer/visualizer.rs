// See LICENSE file for copyright and license details.

use std::cell::RefCell;
use time::precise_time_ns;
use glfw;
use cgmath::vector::{Vector2};
use core::types::{Size2, MInt};
use core::conf::Config;
use core::fs::FileSystem;
use visualizer::mgl;
use visualizer::types::{MatId, ColorId, Time, ScreenPos};
use visualizer::shader::Shader;
use visualizer::font_stash::FontStash;
use visualizer::context::Context;
use visualizer::state_visualizer::{
    StateVisualizer,
    StartGame,
    EndGame,
    QuitMenu,
};
use visualizer::game_state_visualizer::GameStateVisualizer;
use visualizer::menu_state_visualizer::MenuStateVisualizer;

type EventsReceiver = Receiver<(f64, glfw::WindowEvent)>;

pub struct Visualizer {
    visualizers: Vec<Box<StateVisualizer>>,
    dtime: Time,
    last_time: Time,
    glfw: glfw::Glfw,
    events: EventsReceiver,
    context: Context,
    fs: FileSystem,
    should_close: bool,
}

fn create_win(glfw: &glfw::Glfw, win_size: Size2<MInt>)
    -> (glfw::Window, EventsReceiver)
{
    let w = win_size.w as u32;
    let h = win_size.h as u32;
    let title = "Marauder";
    let flags = glfw::Windowed;
    glfw.create_window(w, h, title, flags).unwrap()
}

impl Visualizer {
    pub fn new() -> Visualizer {
        let fs = FileSystem::new();
        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let config = Config::new(&fs.get(&Path::new("data/conf_visualizer.json")));
        let win_size = config.get::<Size2<MInt>>("screen_size");
        let (win, events) = create_win(&glfw, win_size);
        glfw.make_context_current(Some(&win));
        mgl::load_gl_funcs_with(|procname| glfw.get_proc_address(procname));
        mgl::init_opengl();
        win.set_all_polling(true);
        let font_size = config.get("font_size");
        let font_stash = FontStash::new(
            &fs.get(&Path::new("data/DroidSerif-Regular.ttf")), font_size);
        let shader = Shader::new(
            &fs.get(&Path::new("data/normal.vs.glsl")),
            &fs.get(&Path::new("data/normal.fs.glsl")),
        );
        let mvp_mat_id = MatId{id: shader.get_uniform("mvp_mat")};
        let basic_color_id = ColorId{id: shader.get_uniform("basic_color")};
        let context = Context {
            win: win,
            win_size: win_size,
            config: config,
            mouse_pos: ScreenPos{v: Vector2::zero()},
            font_stash: RefCell::new(font_stash),
            shader: shader,
            mvp_mat_id: mvp_mat_id,
            basic_color_id: basic_color_id,
        };
        let visualizer = box MenuStateVisualizer::new(&context);
        Visualizer {
            visualizers: vec![visualizer as Box<StateVisualizer>],
            dtime: Time{n: 0},
            last_time: Time{n: precise_time_ns()},
            glfw: glfw,
            events: events,
            context: context,
            fs: fs,
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
            None => fail!("No state visualizer"),
        };
        match cmd {
            Some(StartGame) => {
                let visualizer = box GameStateVisualizer::new(
                    &self.fs, &self.context);
                self.visualizers.push(visualizer as Box<StateVisualizer>);
            }
            Some(EndGame) => {
                let _ = self.visualizers.pop();
            },
            Some(QuitMenu) => {
                self.should_close = true;
            },
            None => {},
        }
    }

    // TODO: simplify
    pub fn tick(&mut self) {
        {
            let events = self.get_events();
            let visualizer = match self.visualizers.mut_last() {
                Some(visualizer) => visualizer,
                None => fail!("No state visualizer"),
            };
            for event in events.iter() {
                visualizer.handle_event(&self.context, *event);
                self.context.handle_event(*event);
            }
            visualizer.logic();
            visualizer.draw(&self.context, self.dtime);
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
