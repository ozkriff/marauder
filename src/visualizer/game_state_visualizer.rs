// See LICENSE file for copyright and license details.

use std::collections::hashmap::HashMap;
use time::precise_time_ns;
use glfw;
use cgmath::vector::{Vector3, Vector2};
use cgmath::matrix::{Matrix4};
use error_context;
use core::map::MapPosIter;
use core::types::{Size2, MInt, UnitId, PlayerId, MapPos, Point2};
use core::game_state::GameState;
use core::pathfinder::Pathfinder;
use core::core;
use core::fs::FileSystem;
use visualizer::mgl;
use visualizer::camera::Camera;
use visualizer::geom;
use visualizer::picker;
use visualizer::obj;
use visualizer::mesh::{Mesh, MeshId};
use visualizer::scene::{Scene, SceneNode};
use visualizer::types::{
    WorldPos,
    VertexCoord,
    TextureCoord,
    MFloat,
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
use visualizer::gui::{ButtonManager, Button, ButtonId};
use visualizer::selection::{SelectionManager, get_selection_mesh};
use visualizer::context::Context;
use visualizer::state_visualizer::{
    StateVisualizer,
    StateChangeCommand,
    EndGame,
};

fn get_marker(shader: &Shader, tex_path: &Path) -> Mesh {
    let n = 0.2;
    let vertex_data = vec!(
        VertexCoord{v: Vector3{x: -n, y: 0.0, z: 0.1}},
        VertexCoord{v: Vector3{x: 0.0, y: n * 1.4, z: 0.1}},
        VertexCoord{v: Vector3{x: n, y: 0.0, z: 0.1}},
    );
    let tex_data = vec!(
        TextureCoord{v: Vector2{x: 0.0, y: 0.0}},
        TextureCoord{v: Vector2{x: 1.0, y: 0.0}},
        TextureCoord{v: Vector2{x: 0.5, y: 0.5}},
    );
    let mut mesh = Mesh::new(vertex_data.as_slice());
    let tex = Texture::new(tex_path);
    mesh.set_texture(tex, tex_data.as_slice());
    mesh.prepare(shader);
    mesh
}

fn get_scenes(players_count: MInt) -> HashMap<PlayerId, Scene> {
    let mut m = HashMap::new();
    for i in range(0, players_count) {
        m.insert(PlayerId{id: i}, Scene::new());
    }
    m
}

fn get_game_states(players_count: MInt) -> HashMap<PlayerId, GameState> {
    let mut m = HashMap::new();
    for i in range(0, players_count) {
        m.insert(PlayerId{id: i}, GameState::new());
    }
    m
}

fn get_pathfinders(
    players_count: MInt,
    map_size: Size2<MInt>,
) -> HashMap<PlayerId, Pathfinder> {
    let mut m = HashMap::new();
    for i in range(0, players_count) {
        m.insert(PlayerId{id: i}, Pathfinder::new(map_size));
    }
    m
}

fn get_map_mesh(fs: &FileSystem, map_size: Size2<MInt>, shader: &Shader) -> Mesh {
    let mut vertex_data = Vec::new();
    let mut tex_data = Vec::new();
    for tile_pos in MapPosIter::new(map_size) {
        let pos = geom::map_pos_to_world_pos(tile_pos);
        for num in range(0i32, 6) {
            let vertex = geom::index_to_hex_vertex(num);
            let next_vertex = geom::index_to_hex_vertex(num + 1);
            vertex_data.push(VertexCoord{v: pos.v + vertex.v});
            vertex_data.push(VertexCoord{v: pos.v + next_vertex.v});
            vertex_data.push(VertexCoord{v: pos.v});
            tex_data.push(TextureCoord{v: Vector2{x: 0.0, y: 0.0}});
            tex_data.push(TextureCoord{v: Vector2{x: 1.0, y: 0.0}});
            tex_data.push(TextureCoord{v: Vector2{x: 0.5, y: 0.5}});
        }
    }
    let tex = Texture::new(&fs.get(&Path::new("data/floor.png")));
    let mut mesh = Mesh::new(vertex_data.as_slice());
    mesh.set_texture(tex, tex_data.as_slice());
    mesh.prepare(shader);
    mesh
}

fn load_unit_mesh(fs: &FileSystem, shader: &Shader, name: &str) -> Mesh {
    let tex_path = Path::new(format!("data/{}.png", name).as_slice());
    let obj_path = Path::new(format!("data/{}.obj", name).as_slice());
    let tex = Texture::new(&fs.get(&tex_path));
    let obj = obj::Model::new(&fs.get(&obj_path));
    let mut mesh = Mesh::new(obj.build().as_slice());
    mesh.set_texture(tex, obj.build_tex_coord().as_slice());
    mesh.prepare(shader);
    mesh
}

fn add_mesh(meshes: &mut Vec<Mesh>, mesh: Mesh) -> MeshId {
    meshes.push(mesh);
    MeshId{id: (meshes.len() as MInt) - 1}
}

fn get_initial_camera_pos(map_size: &Size2<MInt>) -> WorldPos {
    let pos = get_max_camera_pos(map_size);
    WorldPos{v: Vector3{x: pos.v.x / 2.0, y: pos.v.y / 2.0, z: 0.0}}
}

fn get_max_camera_pos(map_size: &Size2<MInt>) -> WorldPos {
    let pos = geom::map_pos_to_world_pos(
        MapPos{v: Vector2{x: map_size.w, y: map_size.h}});
    WorldPos{v: Vector3{x: -pos.v.x, y: -pos.v.y, z: 0.0}}
}

pub struct GameStateVisualizer {
    map_mesh_id: MeshId,
    tank_mesh_id: MeshId,
    soldier_mesh_id: MeshId,
    shell_mesh_id: MeshId,
    marker_1_mesh_id: MeshId,
    marker_2_mesh_id: MeshId,
    meshes: Vec<Mesh>,
    map_text_mesh: Mesh,
    camera: Camera,
    commands_rx: Receiver<StateChangeCommand>,
    commands_tx: Sender<StateChangeCommand>,
    picker: picker::TilePicker,
    map_pos_under_cursor: Option<MapPos>,
    selected_unit_id: Option<UnitId>,
    unit_under_cursor_id: Option<UnitId>,
    scenes: HashMap<PlayerId, Scene>,
    core: core::Core,
    event: Option<core::Event>,
    event_visualizer: Option<Box<EventVisualizer>>,
    game_states: HashMap<PlayerId, GameState>,
    pathfinders: HashMap<PlayerId, Pathfinder>,
    button_manager: ButtonManager,
    button_end_turn_id: ButtonId,
    button_quit_id: ButtonId,
    selection_manager: SelectionManager,
}

impl GameStateVisualizer {
    pub fn new(fs: &FileSystem, context: &Context) -> GameStateVisualizer {
        set_error_context!("constructing GameStateVisualizer", "-");
        let players_count = 2;
        let core = core::Core::new(fs);
        let map_size = core.map_size();
        let game_states = get_game_states(players_count);
        let picker = picker::TilePicker::new(
            fs,
            game_states.get(&core.player_id()),
            core.map_size(),
        );
        let mut meshes = Vec::new();
        let map_mesh_id = add_mesh(
            &mut meshes, get_map_mesh(fs, map_size, &context.shader));
        let tank_mesh_id = add_mesh(
            &mut meshes, load_unit_mesh(fs, &context.shader, "tank"));
        let soldier_mesh_id = add_mesh(
            &mut meshes, load_unit_mesh(fs, &context.shader, "soldier"));
        let selection_marker_mesh_id = add_mesh(
            &mut meshes, get_selection_mesh(fs, &context.shader));
        let shell_mesh_id = add_mesh(
            &mut meshes,
            get_marker(&context.shader, &fs.get(&Path::new("data/shell.png"))),
        );
        let marker_1_mesh_id = add_mesh(
            &mut meshes,
            get_marker(&context.shader, &fs.get(&Path::new("data/flag1.png"))),
        );
        let marker_2_mesh_id = add_mesh(
            &mut meshes,
            get_marker(&context.shader, &fs.get(&Path::new("data/flag2.png"))),
        );
        let mut camera = Camera::new(context.win_size);
        camera.set_pos(get_initial_camera_pos(&map_size));
        camera.set_max_pos(get_max_camera_pos(&map_size));
        let mut button_manager = ButtonManager::new();
        let button_end_turn_id = button_manager.add_button(Button::new(
            "end turn",
            context.font_stash.borrow_mut().deref_mut(),
            &context.shader,
            Point2{v: Vector2{x: 10, y: 40}})
        );
        let button_quit_id = button_manager.add_button(Button::new(
            "quit",
            context.font_stash.borrow_mut().deref_mut(),
            &context.shader,
            Point2{v: Vector2{x: 10, y: 10}})
        );
        let map_text_mesh = context.font_stash.borrow_mut().deref_mut()
            .get_mesh("test text", &context.shader);
        let (commands_tx, commands_rx) = channel();
        let vis = GameStateVisualizer {
            map_mesh_id: map_mesh_id,
            tank_mesh_id: tank_mesh_id,
            soldier_mesh_id: soldier_mesh_id,
            shell_mesh_id: shell_mesh_id,
            marker_1_mesh_id: marker_1_mesh_id,
            marker_2_mesh_id: marker_2_mesh_id,
            meshes: meshes,
            map_text_mesh: map_text_mesh,
            camera: camera,
            picker: picker,
            map_pos_under_cursor: None,
            selected_unit_id: None,
            unit_under_cursor_id: None,
            core: core,
            event_visualizer: None,
            event: None,
            scenes: get_scenes(players_count),
            game_states: game_states,
            pathfinders: get_pathfinders(players_count, map_size),
            button_manager: button_manager,
            button_end_turn_id: button_end_turn_id,
            button_quit_id: button_quit_id,
            selection_manager: SelectionManager::new(selection_marker_mesh_id),
            commands_rx: commands_rx,
            commands_tx: commands_tx,
        };
        vis
    }

    fn scene<'a>(&'a self) -> &'a Scene {
        self.scenes.get(&self.core.player_id())
    }

    fn draw_scene_node(
        &self,
        node: &SceneNode,
        m: Matrix4<MFloat>,
        context: &Context
    ) {
        let m = mgl::tr(m, node.pos.v);
        let m = mgl::rot_z(m, node.rot);
        match node.mesh_id {
            Some(mesh_id) => {
                context.shader.uniform_mat4f(context.mvp_mat_id, &m);
                let id = mesh_id.id as uint;
                self.meshes.get(id).draw(&context.shader);
            },
            None => {},
        }
        for node in node.children.iter() {
            self.draw_scene_node(node, m, context);
        }
    }

    fn draw_scene_nodes(&self, context: &Context) {
        for (_, node) in self.scene().nodes.iter() {
            self.draw_scene_node(node, self.camera.mat(), context);
        }
    }

    fn draw_map(&mut self, context: &Context) {
        context.shader.uniform_mat4f(context.mvp_mat_id, &self.camera.mat());
        self.meshes.get(self.map_mesh_id.id as uint).draw(&context.shader);
    }

    fn draw_3d_text(&mut self, context: &Context) {
        let mut font_stash = context.font_stash.borrow_mut();
        let m = self.camera.mat();
        let m = mgl::scale(m, 1.0 / font_stash.get_size());
        let m = mgl::rot_x(m, 90.0);
        context.shader.uniform_mat4f(context.mvp_mat_id, &m);
        self.map_text_mesh.draw(&context.shader);
    }

    fn draw_scene(&mut self, context: &Context, dtime: Time) {
        context.shader.uniform_color(context.basic_color_id, mgl::WHITE);
        self.draw_scene_nodes(context);
        self.draw_map(context);
        if !self.event_visualizer.is_none() {
            let scene = self.scenes.get_mut(&self.core.player_id());
            self.event_visualizer.get_mut_ref().draw(scene, dtime);
        }
    }

    fn end_turn(&mut self) {
        self.core.do_command(core::CommandEndTurn);
        self.selected_unit_id = None;
        let scene = self.scenes.get_mut(&self.core.player_id());
        self.selection_manager.deselect(scene);
    }

    fn is_tile_occupied(&self, pos: MapPos) -> bool {
        let state = self.game_states.get(&self.core.player_id());
        state.units_at(pos).len() > 0
    }

    fn create_unit(&mut self) {
        let pos_opt = self.map_pos_under_cursor;
        if pos_opt.is_some() {
            let pos = pos_opt.unwrap();
            if self.is_tile_occupied(pos) {
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
            let state = self.game_states.get(&self.core.player_id());
            let attacker = state.units.get(&attacker_id);
            if attacker.attacked {
                return;
            }
            let cmd = core::CommandAttackUnit(attacker_id, defender_id);
            self.core.do_command(cmd);
        }
    }

    fn select_unit(&mut self) {
        if self.unit_under_cursor_id.is_some() {
            let unit_id = self.unit_under_cursor_id.unwrap();
            self.selected_unit_id = Some(unit_id);
            let state = self.game_states.get(&self.core.player_id());
            let pf = self.pathfinders.get_mut(&self.core.player_id());
            pf.fill_map(state, state.units.get(&unit_id));
            let scene = self.scenes.get_mut(&self.core.player_id());
            self.selection_manager.create_selection_marker(
                state, scene, unit_id);
        }
    }

    fn handle_key_event(&mut self, _: &Context, key: glfw::Key) {
        match key {
            glfw::KeyEscape | glfw::KeyQ => self.commands_tx.send(EndGame),
            glfw::KeyUp | glfw::KeyW => self.camera.move(270.0, 0.1),
            glfw::KeyDown | glfw::KeyS => self.camera.move(90.0, 0.1),
            glfw::KeyRight | glfw::KeyD => self.camera.move(0.0, 0.1),
            glfw::KeyLeft | glfw::KeyA => self.camera.move(180.0, 0.1),
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

    fn handle_cursor_pos_event(&mut self, context: &Context, new_pos: Point2<MFloat>) {
        let rmb = context.win.get_mouse_button(glfw::MouseButtonRight);
        if rmb == glfw::Press {
            let diff = context.mouse_pos.v - new_pos.v;
            let win_w = context.win_size.w as MFloat;
            let win_h = context.win_size.h as MFloat;
            self.camera.add_z_angle(diff.x * (360.0 / win_w));
            self.camera.add_x_angle(diff.y * (360.0 / win_h));
        }
    }

    fn move_unit(&mut self) {
        let pos = self.map_pos_under_cursor.unwrap();
        if self.selected_unit_id.is_none() {
            return;
        }
        let unit_id = self.selected_unit_id.unwrap();
        if self.is_tile_occupied(pos) {
            return;
        }
        let state = self.game_states.get(&self.core.player_id());
        let unit = state.units.get(&unit_id);
        if unit.moved {
            return;
        }
        let pf = self.pathfinders.get_mut(&self.core.player_id());
        let path = pf.get_path(pos);
        if path.len() < 2 {
            return;
        }
        self.core.do_command(core::CommandMove(unit_id, path));
    }

    fn handle_mouse_button_event(&mut self, context: &Context) {
        if self.event_visualizer.is_some() {
            return;
        }
        match self.button_manager.get_clicked_button_id(context) {
            Some(button_id) => {
                if button_id == self.button_end_turn_id {
                    self.end_turn();
                } else if button_id == self.button_quit_id {
                    self.commands_tx.send(EndGame);
                } else {
                    print!("Clicked on {} at {}\n", button_id.id, precise_time_ns());
                }
                return;
            },
            None => {},
        }
        if self.map_pos_under_cursor.is_some() {
            self.move_unit();
        }
        if self.unit_under_cursor_id.is_some() {
            let id = self.unit_under_cursor_id.unwrap();
            let player_id = {
                let state = self.game_states.get(&self.core.player_id());
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

    fn pick_tile(&mut self, context: &Context) {
        let mouse_pos = Vector2 {
            x: context.mouse_pos.v.x as MInt,
            y: context.mouse_pos.v.y as MInt,
        };
        match self.picker.pick_tile(&self.camera, context.win_size, mouse_pos) {
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
    ) -> Box<EventVisualizer> {
        let player_id = self.core.player_id();
        let scene = self.scenes.get_mut(&player_id);
        let state = self.game_states.get(&player_id);
        match *event {
            core::EventMove(ref unit_id, ref path) => {
                EventMoveVisualizer::new(
                    scene, state, *unit_id, path.clone())
            },
            core::EventEndTurn(_, _) => {
                EventEndTurnVisualizer::new()
            },
            core::EventCreateUnit(id, ref pos, type_id, player_id) => {
                let marker_mesh = match player_id.id {
                    0 => self.marker_1_mesh_id,
                    1 => self.marker_2_mesh_id,
                    n => fail!("Wrong player id: {}", n),
                };
                EventCreateUnitVisualizer::new(
                    scene,
                    state,
                    id,
                    type_id,
                    *pos,
                    match type_id {
                        core::Tank => self.tank_mesh_id,
                        core::Soldier => self.soldier_mesh_id,
                    },
                    marker_mesh,
                )
            },
            core::EventAttackUnit(attacker_id, defender_id, killed) => {
                EventAttackUnitVisualizer::new(
                    scene,
                    state,
                    attacker_id,
                    defender_id,
                    killed,
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
        let state = self.game_states.get_mut(&self.core.player_id());
        self.event_visualizer.get_mut_ref().end(scene, state);
        state.apply_event(self.event.get_ref());
        self.event_visualizer = None;
        self.event = None;
        if self.selected_unit_id.is_some() {
            let unit_id = self.selected_unit_id.unwrap();
            let pf = self.pathfinders.get_mut(&self.core.player_id());
            pf.fill_map(state, state.units.get(&unit_id));
            self.selection_manager.move_selection_marker(state, scene);
        }
        self.picker.update_units(state);
    }
}

impl StateVisualizer for GameStateVisualizer {
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

    fn draw(&mut self, context: &Context, dtime: Time) {
        use glfw::Context;
        self.pick_tile(context);
        mgl::set_clear_color(mgl::GREY_3);
        mgl::clear_screen();
        context.shader.activate();
        self.draw_scene(context, dtime);
        context.shader.uniform_color(context.basic_color_id, mgl::BLACK);
        self.draw_3d_text(context);
        self.button_manager.draw(context);
        context.win.swap_buffers();
    }

    fn handle_event(&mut self, context: &Context, event: glfw::WindowEvent) {
        match event {
            glfw::KeyEvent(key, _, glfw::Press, _) => {
                self.handle_key_event(context, key);
            },
            glfw::CursorPosEvent(x, y) => {
                let p = Point2{v: Vector2{x: x as MFloat, y: y as MFloat}};
                self.handle_cursor_pos_event(context, p);
            },
            glfw::MouseButtonEvent(glfw::MouseButtonLeft, glfw::Press, _) => {
                self.handle_mouse_button_event(context);
            },
            glfw::SizeEvent(w, h) => {
                self.camera.regenerate_projection_mat(Size2{w: w, h: h});
            }
            _ => {},
        }
    }

    fn get_command(&self) -> Option<StateChangeCommand> {
        match self.commands_rx.try_recv() {
            Ok(cmd) => Some(cmd),
            Err(_) => None,
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
