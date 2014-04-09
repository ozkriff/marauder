// See LICENSE file for copyright and license details.

use collections::hashmap::HashMap;
use time::precise_time_ns;
use glfw;
use glfw::Context;
use cgmath::vector::{Vec3, Vec2};
use core::map::MapPosIter;
use core::types::{Size2, MInt, MBool, UnitId, PlayerId, MapPos, Point2};
use core::game_state::GameState;
use core::pathfinder::Pathfinder;
use core::conf::Config;
use core::core;
use visualizer::gl_helpers::{
    set_clear_color,
    clear_screen,
    init_opengl,
    load_gl_funcs_with,
    set_viewport,
    tr,
    rot_z,
};
use visualizer::camera::Camera;
use visualizer::geom::Geom;
use visualizer::picker;
use visualizer::obj;
use visualizer::mesh::Mesh;
use visualizer::types::{
    Scene,
    VertexCoord,
    TextureCoord,
    MFloat,
    MatId,
    Time,
};
use visualizer::event_visualizer::{
    EventVisualizer,
    EventMoveVisualizer,
    EventEndTurnVisualizer,
    EventCreateUnitVisualizer,
    EventAttackUnitVisualizer,
};
use visualizer::shader::Shader;
use visualizer::texture::Texture;
use visualizer::font_stash::FontStash;

fn build_hex_mesh(&geom: &Geom, map_size: Size2<MInt>) -> Vec<VertexCoord> {
    let mut vertex_data = Vec::new();
    for tile_pos in MapPosIter::new(map_size) {
        let pos = geom.map_pos_to_world_pos(tile_pos);
        for num in range(0 as MInt, 6) {
            let vertex = geom.index_to_hex_vertex(num);
            let next_vertex = geom.index_to_hex_vertex(num + 1);
            vertex_data.push(pos + vertex);
            vertex_data.push(pos + next_vertex);
            vertex_data.push(pos + Vec3::zero());
        }
    }
    vertex_data
}

fn build_hex_tex_coord(map_size: Size2<MInt>) -> Vec<TextureCoord> {
    let mut vertex_data = Vec::new();
    for _ in MapPosIter::new(map_size) {
        for _ in range(0, 6) {
            vertex_data.push(Vec2{x: 0.0, y: 0.0});
            vertex_data.push(Vec2{x: 1.0, y: 0.0});
            vertex_data.push(Vec2{x: 0.5, y: 0.5});
        }
    }
    vertex_data
}

fn get_marker_pre_mesh() -> (Vec<VertexCoord>, Vec<TextureCoord>) {
    let n = 0.2;
    let mut vertex_data = Vec::new();
    vertex_data.push(Vec3{x: -n, y: 0.0, z: 0.1});
    vertex_data.push(Vec3{x: 0.0, y: n * 1.4, z: 0.1});
    vertex_data.push(Vec3{x: n, y: 0.0, z: 0.1});
    let mut tex_data = Vec::new();
    tex_data.push(Vec2{x: 0.0, y: 0.0});
    tex_data.push(Vec2{x: 1.0, y: 0.0});
    tex_data.push(Vec2{x: 0.5, y: 0.5});
    (vertex_data, tex_data)
}

fn get_marker(shader: &Shader, tex_path: ~str) -> Mesh {
    let (vertex_data, tex_data) = get_marker_pre_mesh();
    let mut mesh = Mesh::new(vertex_data.as_slice());
    let tex = Texture::new(tex_path);
    mesh.set_texture(tex, tex_data.as_slice());
    mesh.prepare(shader);
    mesh
}

fn get_scenes(players_count: MInt) -> HashMap<PlayerId, Scene> {
    let mut m = HashMap::new();
    for i in range(0, players_count) {
        m.insert(PlayerId(i), HashMap::new());
    }
    m
}

fn get_game_states(players_count: MInt) -> HashMap<PlayerId, GameState> {
    let mut m = HashMap::new();
    for i in range(0, players_count) {
        m.insert(PlayerId(i), GameState::new());
    }
    m
}

fn get_pathfinders(
    players_count: MInt,
    map_size: Size2<MInt>,
) -> HashMap<PlayerId, Pathfinder> {
    let mut m = HashMap::new();
    for i in range(0, players_count) {
        m.insert(PlayerId(i), Pathfinder::new(map_size));
    }
    m
}

fn get_map_mesh(geom: &Geom, map_size: Size2<MInt>, shader: &Shader) -> Mesh {
    let tex = Texture::new(~"data/floor.png");
    let mut mesh = Mesh::new(build_hex_mesh(geom, map_size).as_slice());
    mesh.set_texture(tex, build_hex_tex_coord(map_size).as_slice());
    mesh.prepare(shader);
    mesh
}

fn load_unit_mesh(shader: &Shader) -> Mesh {
    let tex = Texture::new(~"data/tank.png");
    let obj = obj::Model::new("data/tank.obj");
    let mut mesh = Mesh::new(obj.build().as_slice());
    mesh.set_texture(tex, obj.build_tex_coord().as_slice());
    mesh.prepare(shader);
    mesh
}

fn add_mesh(meshes: &mut Vec<Mesh>, mesh: Mesh) -> MInt {
    meshes.push(mesh);
    (meshes.len() as MInt) - 1
}

pub struct Visualizer<'a> {
    shader: Shader,
    map_mesh_id: MInt,
    unit_mesh_id: MInt,
    shell_mesh_id: MInt,
    marker_1_mesh_id: MInt,
    marker_2_mesh_id: MInt,
    meshes: Vec<Mesh>,
    mvp_mat_id: MatId,
    win: glfw::Window,
    mouse_pos: Point2<MFloat>,
    camera: Camera,
    picker: ~picker::TilePicker,
    map_pos_under_cursor: Option<MapPos>,
    selected_unit_id: Option<UnitId>,
    unit_under_cursor_id: Option<UnitId>,
    geom: Geom,
    scenes: HashMap<PlayerId, Scene>,
    core: ~core::Core,
    event: Option<core::Event>,
    event_visualizer: Option<~EventVisualizer>,
    game_state: HashMap<PlayerId, GameState>,
    pathfinders: HashMap<PlayerId, Pathfinder>,
    last_time: Time,
    dtime: MInt,
    win_size: Size2<MInt>,
    glfw: glfw::Glfw,
    events: Receiver<(f64, glfw::WindowEvent)>,
    font_stash: FontStash,
}

impl<'a> Visualizer<'a> {
    pub fn new() -> ~Visualizer {
        let players_count = 2;
        let config = Config::new("conf_visualizer.json");
        let win_size = config.get::<Size2<MInt>>("screen_size");
        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let (win, events) = glfw.create_window(
            win_size.w as u32,
            win_size.h as u32,
            "Marauder",
            glfw::Windowed
        ).unwrap();
        glfw.make_context_current(Some(&win));
        win.set_all_polling(true);
        load_gl_funcs_with(|procname| glfw.get_proc_address(procname));
        init_opengl();
        let geom = Geom::new();
        let core = core::Core::new();
        let map_size = core.map_size();
        let picker = picker::TilePicker::new(
            win_size, &geom, core.map_size());
        let shader = Shader::new("normal.vs.glsl", "normal.fs.glsl");
        let mvp_mat_id = MatId(shader.get_uniform("mvp_mat"));
        let mut meshes = Vec::new();
        let map_mesh_id = add_mesh(
            &mut meshes, get_map_mesh(&geom, map_size, &shader));
        let unit_mesh_id = add_mesh(&mut meshes, load_unit_mesh(&shader));
        let shell_mesh_id = add_mesh(
            &mut meshes, get_marker(&shader, ~"data/shell.png"));
        let marker_1_mesh_id = add_mesh(
            &mut meshes, get_marker(&shader, ~"data/flag1.png"));
        let marker_2_mesh_id = add_mesh(
            &mut meshes, get_marker(&shader, ~"data/flag2.png"));
        let /*mut*/ font_stash = FontStash::new("data/DroidSerif-Regular.ttf", 100.0);
        let vis = ~Visualizer {
            map_mesh_id: map_mesh_id,
            unit_mesh_id: unit_mesh_id,
            shell_mesh_id: shell_mesh_id,
            marker_1_mesh_id: marker_1_mesh_id,
            marker_2_mesh_id: marker_2_mesh_id,
            meshes: meshes,
            mvp_mat_id: mvp_mat_id,
            shader: shader,
            win: win,
            mouse_pos: Vec2::zero(),
            camera: Camera::new(win_size),
            picker: picker,
            map_pos_under_cursor: None,
            selected_unit_id: None,
            unit_under_cursor_id: None,
            geom: geom,
            core: core,
            event_visualizer: None,
            event: None,
            scenes: get_scenes(players_count),
            game_state: get_game_states(players_count),
            pathfinders: get_pathfinders(players_count, map_size),
            last_time: precise_time_ns(),
            dtime: 0,
            win_size: win_size,
            glfw: glfw,
            events: events,
            font_stash: font_stash,
        };
        vis
    }

    fn win<'a>(&'a self) -> &'a glfw::Window {
        &self.win
    }

    fn scene<'a>(&'a self) -> &'a Scene {
        self.scenes.get(&self.core.player_id())
    }

    fn draw_units(&self) {
        for (_, node) in self.scene().iter() {
            let mut m = tr(self.camera.mat(), node.pos);
            m = rot_z(m, node.rot);
            self.shader.uniform_mat4f(self.mvp_mat_id, &m);
            self.meshes.get(node.mesh_id as uint).draw(&self.shader);
        }
    }

    fn draw_map(&mut self) {
        self.shader.uniform_mat4f(self.mvp_mat_id, &self.camera.mat());
        self.meshes.get(self.map_mesh_id as uint).draw(&self.shader);
    }

    fn draw(&mut self) {
        set_clear_color(0.3, 0.3, 0.3);
        clear_screen();
        self.shader.activate();
        self.draw_units();
        self.draw_map();
        if !self.event_visualizer.is_none() {
            let scene = self.scenes.get_mut(&self.core.player_id());
            self.event_visualizer.get_mut_ref().draw(
                &self.geom, scene, self.dtime);
        }
        {
            let m = self.font_stash.get_mesh("kill_them_all", &self.shader);
            m.draw(&self.shader);
        }
        self.win().swap_buffers();
    }

    pub fn is_running(&self) -> MBool {
        return !self.win().should_close()
    }

    fn end_turn(&mut self) {
        self.core.do_command(core::CommandEndTurn);
        self.selected_unit_id = None;
    }

    fn is_full_tile(&self, pos: MapPos) -> MBool {
        let state = self.game_state.get(&self.core.player_id());
        let max_units_per_tile = 6;
        state.units_at(pos).len() >= max_units_per_tile
    }

    fn create_unit(&mut self) {
        let pos_opt = self.map_pos_under_cursor;
        if pos_opt.is_some() {
            let pos = pos_opt.unwrap();
            if self.is_full_tile(pos) {
                return;
            }
            let cmd = core::CommandCreateUnit(pos);
            self.core.do_command(cmd);
        }
    }

    fn attack_unit(&mut self) {
        let defender_id_opt = self.unit_under_cursor_id;
        let attacker_id_opt = self.selected_unit_id;
        if defender_id_opt.is_some() && attacker_id_opt.is_some() {
            let defender_id = defender_id_opt.unwrap();
            let attacker_id = attacker_id_opt.unwrap();
            let cmd = core::CommandAttackUnit(attacker_id, defender_id);
            self.core.do_command(cmd);
        }
    }

    fn select_unit(&mut self) {
        if self.unit_under_cursor_id.is_some() {
            let unit_id = self.unit_under_cursor_id.unwrap();
            self.selected_unit_id = Some(unit_id);
            let state = self.game_state.get(&self.core.player_id());
            let pf = self.pathfinders.get_mut(&self.core.player_id());
            pf.fill_map(state, state.units.get(&unit_id));
        }
    }

    fn handle_key_event(&mut self, key: glfw::Key) {
        match key {
            glfw::KeyEscape | glfw::KeyQ => {
                self.win().set_should_close(true);
            },
            glfw::KeyUp => self.camera.move(270.0, 0.1),
            glfw::KeyDown => self.camera.move(90.0, 0.1),
            glfw::KeyRight => self.camera.move(0.0, 0.1),
            glfw::KeyLeft => self.camera.move(180.0, 0.1),
            glfw::KeyMinus => self.camera.zoom *= 1.3,
            glfw::KeyEqual => self.camera.zoom /= 1.3,
            _ => {},
        }
        if self.event_visualizer.is_some() {
            return;
        }
        match key {
            glfw::KeyT => self.end_turn(),
            glfw::KeyU => self.create_unit(),
            _ => {},
        }
    }

    fn handle_cursor_pos_event(&mut self, pos: Point2<MFloat>) {
        let rmb = self.win().get_mouse_button(glfw::MouseButtonRight);
        if rmb == glfw::Press {
            let diff = self.mouse_pos - pos;
            let win_w = self.win_size.w as MFloat;
            let win_h = self.win_size.h as MFloat;
            self.camera.z_angle += diff.x * (360.0 / win_w);
            self.camera.x_angle += diff.y * (360.0 / win_h);
        }
        let mmb = self.win().get_mouse_button(glfw::MouseButtonMiddle);
        if mmb == glfw::Press {
            // TODO: Remove 0.05
            let diff = self.mouse_pos - pos;
            self.camera.move(90.0, diff.y * 0.01);
            self.camera.move(0.0, diff.x * 0.01);
        }
        self.mouse_pos = pos;
    }

    fn move_unit(&mut self) {
        let pos = self.map_pos_under_cursor.unwrap();
        if self.selected_unit_id.is_none() {
            return;
        }
        if self.is_full_tile(pos) {
            return;
        }
        let unit_id = self.selected_unit_id.unwrap();
        let pf = self.pathfinders.get_mut(&self.core.player_id());
        let path = pf.get_path(pos);
        if path.len() < 2 {
            return;
        }
        self.core.do_command(core::CommandMove(unit_id, path));
    }

    fn handle_mouse_button_event(&mut self) {
        if self.event_visualizer.is_some() {
            return;
        }
        if self.map_pos_under_cursor.is_some() {
            self.move_unit();
        }
        if self.unit_under_cursor_id.is_some() {
            let id = self.unit_under_cursor_id.unwrap();
            let player_id = {
                let state = self.game_state.get(&self.core.player_id());
                let unit = state.units.get(&id);
                unit.player_id
            };
            if player_id == self.core.player_id() {
                self.select_unit();
            } else {
                self.attack_unit();
            }
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

    fn handle_event(&mut self, event: glfw::WindowEvent) {
        match event {
            glfw::KeyEvent(key, _, glfw::Press, _) => {
                self.handle_key_event(key);
            },
            glfw::CursorPosEvent(x, y) => {
                let p = Vec2{x: x as MFloat, y: y as MFloat};
                self.handle_cursor_pos_event(p);
            },
            glfw::MouseButtonEvent(glfw::MouseButtonLeft, glfw::Press, _) => {
                self.handle_mouse_button_event();
            },
            glfw::SizeEvent(w, h) => {
                let size = Size2{w: w, h: h};
                set_viewport(size);
                self.picker.set_win_size(size);
                self.camera.set_win_size(size);
                self.win_size = size;
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
            x: self.mouse_pos.x as MInt,
            y: self.mouse_pos.y as MInt,
        };
        match self.picker.pick_tile(&self.camera, mouse_pos) {
            picker::PickedMapPos(pos) => {
                self.map_pos_under_cursor = Some(pos);
                self.unit_under_cursor_id = None;
            },
            picker::PickedUnitId(id) => {
                self.map_pos_under_cursor = None;
                self.unit_under_cursor_id = Some(id);
            },
            picker::PickedNothing => {},
        }
    }

    fn make_event_visualizer(
        &mut self,
        event: &core::Event
    ) -> ~EventVisualizer {
        let player_id = self.core.player_id();
        let scene = self.scenes.get_mut(&player_id);
        let state = self.game_state.get(&player_id);
        let geom = &self.geom;
        match *event {
            core::EventMove(ref unit_id, ref path) => {
                EventMoveVisualizer::new(
                    geom, scene, state, *unit_id, path.clone())
            },
            core::EventEndTurn(_, _) => {
                EventEndTurnVisualizer::new()
            },
            core::EventCreateUnit(id, ref pos, player_id) => {
                let marker_mesh = match player_id {
                    PlayerId(0) => self.marker_1_mesh_id,
                    PlayerId(1) => self.marker_2_mesh_id,
                    PlayerId(n) => fail!("Wrong player id: {}", n),
                };
                EventCreateUnitVisualizer::new(
                    geom,
                    scene,
                    state,
                    id,
                    *pos,
                    self.unit_mesh_id,
                    marker_mesh,
                )
            },
            core::EventAttackUnit(attacker_id, defender_id) => {
                EventAttackUnitVisualizer::new(
                    geom,
                    scene,
                    state,
                    attacker_id,
                    defender_id,
                    self.shell_mesh_id,
                )
            },
        }
    }

    fn start_event_visualization(&mut self, event: core::Event) {
        let vis = self.make_event_visualizer(&event);
        self.event = Some(event);
        self.event_visualizer = Some(vis);
    }

    fn end_event_visualization(&mut self) {
        let scene = self.scenes.get_mut(&self.core.player_id());
        let state = self.game_state.get_mut(&self.core.player_id());
        self.event_visualizer.get_mut_ref().end(&self.geom, scene, state);
        state.apply_event(self.event.get_ref());
        self.event_visualizer = None;
        self.event = None;
        if self.selected_unit_id.is_some() {
            let unit_id = self.selected_unit_id.unwrap();
            let pf = self.pathfinders.get_mut(&self.core.player_id());
            pf.fill_map(state, state.units.get(&unit_id));
        }
        self.picker.update_units(&self.geom, scene);
    }

    fn logic(&mut self) {
        if self.event_visualizer.is_none() {
            match self.core.get_event() {
                Some(e) => self.start_event_visualization(e),
                None => {},
            }
        } else if self.event_visualizer.get_ref().is_finished() {
            self.end_event_visualization();
        }
    }

    pub fn update_time(&mut self) {
        let time = precise_time_ns();
        self.dtime = (time - self.last_time) as MInt;
        self.last_time = time;
    }

    pub fn tick(&mut self) {
        self.handle_events();
        self.logic();
        self.pick_tile();
        self.draw();
        self.update_time();
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
