// See LICENSE file for copyright and license details.

use extra::json;
use std::hashmap::HashMap;
use serialize::Decodable;
use glfw;
use gl;
use cgmath::vector::{
    Vec3,
    Vec2,
};
use glh = gl_helpers;
use camera::Camera;
use map::MapPosIter;
use geom::Geom;
use tile_picker::TilePicker;
use obj;
use mesh::Mesh;
use misc::read_file;
use core::{
    Core,
    CommandEndTurn,
    CommandMove,
    CommandCreateUnit,
    Event,
    EventMove,
    EventEndTurn,
    EventCreateUnit,
};
use core_types::{
    Size2,
    Int,
    Bool,
    UnitId,
    PlayerId,
    MapPos,
};
use gl_types::{
    Scene,
    VertexCoord,
    TextureCoord,
    Float,
    Point2,
    MatId,
    ShaderId,
};
use event_visualizer::{
    EventVisualizer,
    EventMoveVisualizer,
    EventEndTurnVisualizer,
    EventCreateUnitVisualizer,
};

fn build_hex_mesh(&geom: &Geom, map_size: Size2<Int>) -> ~[VertexCoord] {
    let mut vertex_data = ~[];
    for tile_pos in MapPosIter::new(map_size) {
        let pos = geom.map_pos_to_world_pos(tile_pos);
        for num in range(0 as Int, 6) {
            let vertex = geom.index_to_hex_vertex(num);
            let next_vertex = geom.index_to_hex_vertex(num + 1);
            vertex_data.push(pos + vertex);
            vertex_data.push(pos + next_vertex);
            vertex_data.push(pos + Vec3::zero());
        }
    }
    vertex_data
}

fn build_hex_tex_coord(map_size: Size2<Int>) -> ~[TextureCoord] {
    let mut vertex_data = ~[];
    for _ in MapPosIter::new(map_size) {
        for _ in range(0, 6) {
            vertex_data.push(Vec2{x: 0.0, y: 0.0});
            vertex_data.push(Vec2{x: 1.0, y: 0.0});
            vertex_data.push(Vec2{x: 0.5, y: 0.5});
        }
    }
    vertex_data
}

fn read_win_size(config_path: &str) -> Size2<Int> {
    let path = Path::new(config_path);
    let json = json::from_str(read_file(&path)).unwrap();
    let mut decoder = json::Decoder::new(json);
    let size: Size2<Int> = Decodable::decode(&mut decoder);
    size
}

fn init_win(win_size: Size2<Int>) -> glfw::Window {
    glfw::set_error_callback(~glfw::LogErrorHandler);
    let init_status = glfw::init();
    if !init_status.is_ok() {
        fail!("Failed to initialize GLFW");
    }
    let win = glfw::Window::create(
        win_size.x as u32,
        win_size.y as u32,
        "Marauder",
        glfw::Windowed,
    ).unwrap();
    win.make_context_current();
    win.set_all_polling(true);
    win
}

pub struct Visualizer<'a> {
    program: ShaderId,
    map_mesh: Mesh,
    unit_mesh: Mesh,
    mat_id: MatId,
    win: glfw::Window,
    mouse_pos: Point2<Float>,
    camera: Camera,
    picker: TilePicker,
    selected_tile_pos: Option<MapPos>,
    selected_unit_id: Option<UnitId>,
    geom: Geom,
    scenes: HashMap<PlayerId, Scene>,
    core: Core<'a>,
    event_visualizer: Option<~EventVisualizer>,
}

impl<'a> Visualizer<'a> {
    pub fn new() -> ~Visualizer {
        let win_size = read_win_size("config.json");
        let win = init_win(win_size);
        let geom = Geom::new();
        let mut vis = ~Visualizer {
            program: 0,
            map_mesh: Mesh::new(),
            unit_mesh: Mesh::new(),
            mat_id: 0,
            win: win,
            mouse_pos: Vec2::zero(),
            camera: Camera::new(),
            picker: TilePicker::new(win_size),
            selected_tile_pos: None,
            selected_unit_id: None,
            geom: geom,
            scenes: {
                let mut m = HashMap::new();
                m.insert(0 as PlayerId, HashMap::new());
                m.insert(1 as PlayerId, HashMap::new());
                m
            },
            core: Core::new(),
            event_visualizer: None,
        };
        vis.init_opengl();
        vis.picker.init(&geom, vis.core.map_size);
        vis.init_models();
        vis
    }

    fn win<'a>(&'a self) -> &'a glfw::Window {
        &self.win
    }

    fn init_models(&mut self) {
        self.program = glh::compile_program(
            read_file(&Path::new("normal.vs.glsl")),
            read_file(&Path::new("normal.fs.glsl")),
        );
        gl::UseProgram(self.program);
        self.mat_id = glh::get_uniform(self.program, "mvp_mat");
        let vertex_coordinates_attr = glh::get_attr(
            self.program, "in_vertex_coordinates");
        gl::EnableVertexAttribArray(vertex_coordinates_attr);
        glh::vertex_attrib_pointer(vertex_coordinates_attr, 3);
        let texture_coords_attr = glh::get_attr(
            self.program, "in_texture_coordinates");
        gl::EnableVertexAttribArray(texture_coords_attr);
        glh::vertex_attrib_pointer(texture_coords_attr, 3);
        let map_vertex_data = build_hex_mesh(&self.geom, self.core.map_size);
        self.map_mesh.set_vertex_coords(map_vertex_data);
        self.map_mesh.set_texture_coords(
            build_hex_tex_coord(self.core.map_size));
        self.map_mesh.set_texture(glh::load_texture(~"data/floor.png"));
        let unit_obj = obj::Model::new("data/tank.obj");
        self.unit_mesh.set_vertex_coords(unit_obj.build());
        self.unit_mesh.set_texture_coords(unit_obj.build_tex_coord());
        self.unit_mesh.set_texture(glh::load_texture(~"data/tank.png"));
    }

    fn scene<'a>(&'a self) -> &'a Scene {
        self.scenes.get(&self.core.current_player_id)
    }

    fn init_opengl(&mut self) {
        gl::load_with(glfw::get_proc_address);
        gl::Enable(gl::DEPTH_TEST);
    }

    fn cleanup_opengl(&self) {
        gl::DeleteProgram(self.program);
    }

    fn draw_units(&self) {
        gl::UseProgram(self.program);
        for (_, unit) in self.scene().iter() {
            let m = glh::tr(self.camera.mat(), unit.pos);
            glh::uniform_mat4f(self.mat_id, &m);
            self.unit_mesh.draw(self.program);
        }
    }

    fn draw_map(&self) {
        glh::uniform_mat4f(self.mat_id, &self.camera.mat());
        self.map_mesh.draw(self.program);
    }

    fn draw(&mut self) {
        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::UseProgram(self.program);
        self.draw_units();
        self.draw_map();
        if !self.event_visualizer.is_none() {
            let scene = self.scenes.get_mut(&self.core.current_player_id);
            self.event_visualizer.get_mut_ref().draw(&self.geom, scene);
        }
        self.win().swap_buffers();
    }

    pub fn is_running(&self) -> Bool {
        return !self.win().should_close()
    }

    fn handle_key_event(&mut self, key: glfw::Key) {
        match key {
            glfw::KeyEscape | glfw::KeyQ => {
                self.win().set_should_close(true);
            },
            glfw::KeyT => {
                self.core.do_command(CommandEndTurn);
            },
            glfw::KeyU => {
                let pos_opt = self.selected_tile_pos;
                if pos_opt.is_some() {
                    let pos = pos_opt.unwrap();
                    if self.core.unit_at_opt(pos).is_none() {
                        let cmd = CommandCreateUnit(pos);
                        self.core.do_command(cmd);
                    }
                }
            },
            glfw::KeySpace => println!("space"),
            glfw::KeyUp => self.camera.move(270.0),
            glfw::KeyDown => self.camera.move(90.0),
            glfw::KeyRight => self.camera.move(0.0),
            glfw::KeyLeft => self.camera.move(180.0),
            glfw::KeyMinus => self.camera.zoom += 1.0,
            glfw::KeyEqual => self.camera.zoom -= 1.0,
            _ => {},
        }
    }

    fn handle_cursor_pos_event(&mut self, pos: Point2<Float>) {
        let button = self.win().get_mouse_button(glfw::MouseButtonRight);
        if button == glfw::Press {
            self.camera.z_angle += (self.mouse_pos.x - pos.x) / 2.0;
            self.camera.x_angle += (self.mouse_pos.y - pos.y) / 2.0;
        }
        self.mouse_pos = pos;
    }

    fn handle_mouse_button_event(&mut self) {
        if self.selected_tile_pos.is_some() {
            let pos = self.selected_tile_pos.unwrap();
            if self.core.unit_at_opt(pos).is_some() {
                let unit = self.core.unit_at_opt(pos).unwrap();
                self.selected_unit_id = Some(unit.id);
            } else if self.selected_unit_id.is_some() {
                let unit_id = self.selected_unit_id.unwrap();
                self.core.do_command(CommandMove(unit_id, pos));
            }
        }
    }

    fn get_events(&mut self) -> ~[glfw::WindowEvent] {
        glfw::poll_events();
        let mut events = ~[];
        for (_, event) in self.win().flush_events() {
            events.push(event);
        }
        events
    }

    fn handle_event(&mut self, event: glfw::WindowEvent) {
        match event {
            glfw::KeyEvent(key, _, glfw::Press, _) => {
                self.handle_key_event(key);
            },
            glfw::CursorPosEvent(x, y) => {
                let p = Vec2{x: x as Float, y: y as Float};
                self.handle_cursor_pos_event(p);
            },
            glfw::MouseButtonEvent(glfw::MouseButtonLeft, glfw::Press, _) => {
                self.handle_mouse_button_event();
            },
            glfw::SizeEvent(w, h) => {
                gl::Viewport(0, 0, w, h);
                self.picker.set_win_size(Size2{x: w, y: h});
            },
            _ => {},
        }
    }

    fn handle_events(&mut self) {
        for event in self.get_events().iter() {
            self.handle_event(*event);
        }
    }

    fn pick_tile(&mut self) {
        let mouse_pos = Vec2 {
            x: self.mouse_pos.x as Int,
            y: self.mouse_pos.y as Int,
        };
        self.selected_tile_pos = self.picker.pick_tile(&self.camera, mouse_pos);
    }

    pub fn make_event_visualizer(&mut self, event: Event) -> ~EventVisualizer {
        match event {
            EventMove(unit_id, path) => {
                EventMoveVisualizer::new(unit_id, path)
            },
            EventEndTurn(_, _) => {
                EventEndTurnVisualizer::new()
            },
            EventCreateUnit(id, pos) => {
                EventCreateUnitVisualizer::new(id, pos)
            },
        }
    }

    pub fn logic(&mut self) {
        if self.event_visualizer.is_none() {
            let event_opt = self.core.get_event();
            if event_opt.is_some() {
                let event = event_opt.unwrap();
                let vis = self.make_event_visualizer(event);
                self.event_visualizer = Some(vis);
            }
        } else if self.event_visualizer.get_ref().is_finished() {
            let scene = self.scenes.get_mut(&self.core.current_player_id);
            self.event_visualizer.get_mut_ref().end(&self.geom, scene);
            self.event_visualizer = None;
        }
    }

    pub fn tick(&mut self) {
        self.handle_events();
        self.logic();
        self.pick_tile();
        self.draw();
    }
}

impl<'a> Drop for Visualizer<'a> {
    fn drop(&mut self) {
        self.cleanup_opengl();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
