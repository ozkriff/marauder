// See LICENSE file for copyright and license details.

use collections::hashmap::HashMap;
use serialize::{
    Decodable,
    json,
};
use glfw;
use cgmath::vector::{
    Vec3,
    Vec2,
};
use gl_helpers::{
    Shader,
    Texture,
    get_attr,
    get_uniform,
    uniform_mat4f,
    set_clear_color,
    clear,
    init_opengl,
    load_gl_funcs_with,
    viewport,
    tr,
};
use camera::Camera;
use map::MapPosIter;
use geom::Geom;
use tile_picker::TilePicker;
use obj;
use mesh::Mesh;
use core::{
    Core,
    Unit,
    CommandEndTurn,
    CommandMove,
    CommandCreateUnit,
    CommandAttackUnit,
    Event,
    EventMove,
    EventEndTurn,
    EventCreateUnit,
    EventAttackUnit,
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
};
use event_visualizer::{
    EventVisualizer,
    EventMoveVisualizer,
    EventEndTurnVisualizer,
    EventCreateUnitVisualizer,
    EventAttackUnitVisualizer,
};
use game_state::GameState;
use pathfinder::Pathfinder;
use misc::read_file;

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
        win_size.w as u32,
        win_size.h as u32,
        "Marauder",
        glfw::Windowed,
    ).unwrap();
    win.make_context_current();
    win.set_all_polling(true);
    win
}

fn get_scenes(players_count: Int) -> HashMap<PlayerId, Scene> {
    let mut m = HashMap::new();
    for i in range(0, players_count) {
        m.insert(PlayerId(i), HashMap::new());
    }
    m
}

fn get_game_states(players_count: Int) -> HashMap<PlayerId, GameState> {
    let mut m = HashMap::new();
    for i in range(0, players_count) {
        m.insert(PlayerId(i), GameState::new());
    }
    m
}

fn get_pathfinders(
    players_count: Int,
    map_size: Size2<Int>,
) -> HashMap<PlayerId, Pathfinder> {
    let mut m = HashMap::new();
    for i in range(0, players_count) {
        m.insert(PlayerId(i), Pathfinder::new(map_size));
    }
    m
}

pub struct Visualizer<'a> {
    shader: Shader,
    map_mesh: Mesh,
    unit_mesh: Mesh,
    mvp_mat: MatId,
    win: glfw::Window,
    mouse_pos: Point2<Float>,
    camera: Camera,
    picker: TilePicker,
    selected_tile_pos: Option<MapPos>,
    selected_unit_id: Option<UnitId>,
    geom: Geom,
    scenes: HashMap<PlayerId, Scene>,
    core: ~Core<'a>,
    event: Option<Event>,
    event_visualizer: Option<~EventVisualizer>,
    game_state: HashMap<PlayerId, GameState>,
    pathfinders: HashMap<PlayerId, Pathfinder>,
}

impl<'a> Visualizer<'a> {
    pub fn new() -> ~Visualizer {
        let players_count = 2;
        let win_size = read_win_size("config.json");
        let win = init_win(win_size);
        let geom = Geom::new();
        let core = Core::new();
        let map_size = core.map_size();
        let mut vis = ~Visualizer {
            shader: Shader(0),
            map_mesh: Mesh::new(),
            unit_mesh: Mesh::new(),
            mvp_mat: MatId(0),
            win: win,
            mouse_pos: Vec2::zero(),
            camera: Camera::new(),
            picker: TilePicker::new(win_size),
            selected_tile_pos: None,
            selected_unit_id: None,
            geom: geom,
            core: core,
            event_visualizer: None,
            event: None,
            scenes: get_scenes(players_count),
            game_state: get_game_states(players_count),
            pathfinders: get_pathfinders(players_count, map_size),
        };
        load_gl_funcs_with(glfw::get_proc_address);
        vis.init_opengl();
        vis.picker.init(&geom, vis.core.map_size());
        vis.init_models();
        vis
    }

    fn win<'a>(&'a self) -> &'a glfw::Window {
        &self.win
    }

    fn init_models(&mut self) {
        self.shader = Shader::new("normal.vs.glsl", "normal.fs.glsl");
        self.shader.activate();
        self.mvp_mat = MatId(get_uniform(&self.shader, "mvp_mat"));
        let vertex_coordinates_attr = get_attr(
            &self.shader, "in_vertex_coordinates");
        vertex_coordinates_attr.enable();
        vertex_coordinates_attr.vertex_pointer(3);
        let texture_coords_attr = get_attr(
            &self.shader, "in_texture_coordinates");
        texture_coords_attr.enable();
        texture_coords_attr.vertex_pointer(3);
        let map_size = self.core.map_size();
        let map_vertex_data = build_hex_mesh(&self.geom, map_size);
        self.map_mesh.set_vertex_coords(map_vertex_data);
        self.map_mesh.set_texture_coords(build_hex_tex_coord(map_size));
        self.map_mesh.set_texture(Texture::new(~"data/floor.png"));
        let unit_obj = obj::Model::new("data/tank.obj");
        self.unit_mesh.set_vertex_coords(unit_obj.build());
        self.unit_mesh.set_texture_coords(unit_obj.build_tex_coord());
        self.unit_mesh.set_texture(Texture::new(~"data/tank.png"));
    }

    fn scene<'a>(&'a self) -> &'a Scene {
        self.scenes.get(&self.core.player_id())
    }

    fn unit_at_opt(&'a self, pos: MapPos) -> Option<&'a Unit> {
        let mut res = None;
        let id = self.core.player_id();
        for (_, unit) in self.game_state.get(&id).units.iter() {
            if unit.pos == pos {
                res = Some(unit);
                break;
            }
        }
        res
    }

    fn init_opengl(&mut self) {
        init_opengl();
    }

    fn draw_units(&self) {
        self.shader.activate();
        for (_, unit) in self.scene().iter() {
            let m = tr(self.camera.mat(), unit.pos);
            uniform_mat4f(self.mvp_mat, &m);
            self.unit_mesh.draw(&self.shader);
        }
    }

    fn draw_map(&self) {
        uniform_mat4f(self.mvp_mat, &self.camera.mat());
        self.map_mesh.draw(&self.shader);
    }

    fn draw(&mut self) {
        set_clear_color(0.3, 0.3, 0.3);
        clear();
        self.shader.activate();
        self.draw_units();
        self.draw_map();
        if !self.event_visualizer.is_none() {
            let scene = self.scenes.get_mut(&self.core.player_id());
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
            glfw::KeyUp => self.camera.move(270.0),
            glfw::KeyDown => self.camera.move(90.0),
            glfw::KeyRight => self.camera.move(0.0),
            glfw::KeyLeft => self.camera.move(180.0),
            glfw::KeyMinus => self.camera.zoom += 1.0,
            glfw::KeyEqual => self.camera.zoom -= 1.0,
            _ => {},
        }
        if self.event_visualizer.is_some() {
            return;
        }
        match key {
            glfw::KeyT => {
                self.core.do_command(CommandEndTurn);
                self.selected_unit_id = None;
            },
            glfw::KeyU => {
                let pos_opt = self.selected_tile_pos;
                if pos_opt.is_some() {
                    let pos = pos_opt.unwrap();
                    if self.unit_at_opt(pos).is_none() {
                        let cmd = CommandCreateUnit(pos);
                        self.core.do_command(cmd);
                    }
                }
            },
            glfw::KeyA => {
                let pos_opt = self.selected_tile_pos;
                if pos_opt.is_some() {
                    let pos = pos_opt.unwrap();
                    if self.unit_at_opt(pos).is_some() {
                        let defender_id = self.unit_at_opt(pos).unwrap().id;
                        let cmd = CommandAttackUnit(UnitId(0), defender_id);
                        self.core.do_command(cmd);
                    }
                }
            },
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
        if self.event_visualizer.is_some() {
            return;
        }
        if self.selected_tile_pos.is_some() {
            let pos = self.selected_tile_pos.unwrap();
            if self.unit_at_opt(pos).is_some() {
                let unit_id = self.unit_at_opt(pos).unwrap().id;
                let state = self.game_state.get_mut(&self.core.player_id());
                self.selected_unit_id = Some(unit_id);
                let pf = self.pathfinders.get_mut(&self.core.player_id());
                pf.fill_map(state, state.units.get(&unit_id));
            } else if self.selected_unit_id.is_some() {
                let unit_id = self.selected_unit_id.unwrap();
                let pf = self.pathfinders.get_mut(&self.core.player_id());
                let path = pf.get_path(pos);
                self.core.do_command(CommandMove(unit_id, path));
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
                viewport(Size2{w: w, h: h});
                self.picker.set_win_size(Size2{w: w, h: h});
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

    pub fn make_event_visualizer(&mut self, event: &Event) -> ~EventVisualizer {
        match *event {
            EventMove(ref unit_id, ref path) => {
                EventMoveVisualizer::new(*unit_id, path.clone())
            },
            EventEndTurn(_, _) => {
                EventEndTurnVisualizer::new()
            },
            EventCreateUnit(id, ref pos) => {
                EventCreateUnitVisualizer::new(id, *pos)
            },
            EventAttackUnit(attacker_id, defender_id) => {
                EventAttackUnitVisualizer::new(attacker_id, defender_id)
            },
        }
    }

    pub fn logic(&mut self) {
        if self.event_visualizer.is_none() {
            let event_opt = self.core.get_event();
            if event_opt.is_some() {
                let event = event_opt.unwrap();
                let vis = self.make_event_visualizer(&event);
                self.event = Some(event);
                self.event_visualizer = Some(vis);
            }
        } else if self.event_visualizer.get_ref().is_finished() {
            let scene = self.scenes.get_mut(&self.core.player_id());
            self.event_visualizer.get_mut_ref().end(&self.geom, scene);
            let state = self.game_state.get_mut(&self.core.player_id());
            state.apply_event(self.event.get_ref());
            self.event_visualizer = None;
            self.event = None;
            if self.selected_unit_id.is_some() {
                let unit_id = self.selected_unit_id.unwrap();
                let pf = self.pathfinders.get_mut(&self.core.player_id());
                pf.fill_map(state, state.units.get(&unit_id));
            }
        }
    }

    pub fn tick(&mut self) {
        self.handle_events();
        self.logic();
        self.pick_tile();
        self.draw();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
