// See LICENSE file for copyright and license details.

use crate::core::core::Command::{
    CommandAttackUnit, CommandCreateUnit, CommandEndTurn, CommandMove,
};
use crate::core::core::Event::{EventAttackUnit, EventCreateUnit, EventEndTurn, EventMove};
use crate::core::core::{Core, Event, UnitTypeId};
use crate::core::dir::Dir;
use crate::core::fs::FileSystem;
use crate::core::game_state::GameState;
use crate::core::map::{distance, MapPosIter};
use crate::core::pathfinder::Pathfinder;
use crate::core::types::{MInt, MapPos, PlayerId, Size2, UnitId};
use crate::visualizer::camera::Camera;
use crate::visualizer::context::Context;
use crate::visualizer::event_visualizer::{
    EventAttackUnitVisualizer, EventCreateUnitVisualizer, EventEndTurnVisualizer,
    EventMoveVisualizer, EventVisualizer,
};
use crate::visualizer::gui::{Button, ButtonId, ButtonManager};
use crate::visualizer::mesh::{Mesh, MeshId};
use crate::visualizer::picker::PickResult::{PickedMapPos, PickedUnitId};
use crate::visualizer::scene::{Scene, SceneNode};
use crate::visualizer::selection::{get_selection_mesh, SelectionManager};
use crate::visualizer::shader::Shader;
use crate::visualizer::state_visualizer::StateChangeCommand::EndGame;
use crate::visualizer::state_visualizer::{StateChangeCommand, StateVisualizer};
use crate::visualizer::texture::Texture;
use crate::visualizer::types::{MFloat, ScreenPos, TextureCoord, Time, VertexCoord, WorldPos};
use crate::visualizer::unit_type_visual_info::{UnitTypeVisualInfo, UnitTypeVisualInfoManager};
use crate::visualizer::{geom, mgl, obj, picker};
use cgmath::{Matrix4, Vector2, Vector3};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use time::precise_time_ns;

fn get_marker(shader: &Shader, tex_path: &Path) -> Mesh {
    let n = 0.2;
    let vertex_data = vec![
        VertexCoord {
            v: Vector3 {
                x: -n,
                y: 0.0,
                z: 0.1,
            },
        },
        VertexCoord {
            v: Vector3 {
                x: 0.0,
                y: n * 1.4,
                z: 0.1,
            },
        },
        VertexCoord {
            v: Vector3 {
                x: n,
                y: 0.0,
                z: 0.1,
            },
        },
    ];
    let tex_data = vec![
        TextureCoord {
            v: Vector2 { x: 0.0, y: 0.0 },
        },
        TextureCoord {
            v: Vector2 { x: 1.0, y: 0.0 },
        },
        TextureCoord {
            v: Vector2 { x: 0.5, y: 0.5 },
        },
    ];
    let mut mesh = Mesh::new(vertex_data.as_slice());
    let tex = Texture::new(tex_path);
    mesh.set_texture(tex, tex_data.as_slice());
    mesh.prepare(shader);
    mesh
}

fn get_scenes(players_count: MInt) -> HashMap<PlayerId, Scene> {
    let mut m = HashMap::new();
    for i in 0..players_count {
        let _ = m.insert(PlayerId { id: i }, Scene::new());
    }
    m
}

fn get_game_states(players_count: MInt) -> HashMap<PlayerId, GameState> {
    let mut m = HashMap::new();
    for i in 0..players_count {
        let _ = m.insert(PlayerId { id: i }, GameState::new());
    }
    m
}

fn get_pathfinders(players_count: MInt, map_size: Size2<MInt>) -> HashMap<PlayerId, Pathfinder> {
    let mut m = HashMap::new();
    for i in 0..players_count {
        let _ = m.insert(PlayerId { id: i }, Pathfinder::new(map_size));
    }
    m
}

fn build_walkable_mesh(pathfinder: &Pathfinder, shader: &Shader) -> Mesh {
    let map = pathfinder.get_map();
    let map_size = map.get_size();
    let mut vertex_data = Vec::new();
    for tile_pos in MapPosIter::new(map_size) {
        match map.tile(tile_pos.clone()).parent {
            Some(parent_dir) => {
                let tile_pos_to = Dir::get_neighbour_pos(tile_pos, parent_dir);
                let world_pos_from = geom::map_pos_to_world_pos(tile_pos.clone());
                let world_pos_to = geom::map_pos_to_world_pos(tile_pos_to);
                vertex_data.push(VertexCoord {
                    v: geom::lift(world_pos_from.v),
                });
                vertex_data.push(VertexCoord {
                    v: geom::lift(world_pos_to.v),
                });
            }
            None => {}
        }
    }
    let mut mesh = Mesh::new(vertex_data.as_slice());
    mesh.set_mode(mgl::MeshRenderMode::Lines);
    mesh.prepare(shader);
    mesh
}

fn get_map_mesh(fs: &FileSystem, map_size: Size2<MInt>, shader: &Shader) -> Mesh {
    let mut vertex_data = Vec::new();
    let mut tex_data = Vec::new();
    for tile_pos in MapPosIter::new(map_size) {
        let pos = geom::map_pos_to_world_pos(tile_pos);
        for num in 0..6 {
            let vertex = geom::index_to_hex_vertex(num);
            let next_vertex = geom::index_to_hex_vertex(num + 1);
            vertex_data.push(VertexCoord {
                v: pos.v + vertex.v,
            });
            vertex_data.push(VertexCoord {
                v: pos.v + next_vertex.v,
            });
            vertex_data.push(VertexCoord { v: pos.v });
            tex_data.push(TextureCoord {
                v: Vector2 { x: 0.0, y: 0.0 },
            });
            tex_data.push(TextureCoord {
                v: Vector2 { x: 1.0, y: 0.0 },
            });
            tex_data.push(TextureCoord {
                v: Vector2 { x: 0.5, y: 0.5 },
            });
        }
    }
    let tex = Texture::new(&fs.get(&Path::new("data/floor.png")));
    let mut mesh = Mesh::new(vertex_data.as_slice());
    mesh.set_texture(tex, tex_data.as_slice());
    mesh.prepare(shader);
    mesh
}

fn load_unit_mesh(fs: &FileSystem, shader: &Shader, name: &str) -> Mesh {
    let png = format!("data/{}.png", name);
    let obj = format!("data/{}.obj", name);
    let tex_path = Path::new(&png);
    let obj_path = Path::new(&obj);
    let tex = Texture::new(&fs.get(&tex_path));
    let obj = obj::Model::new(&fs.get(&obj_path));
    let mut mesh = Mesh::new(obj.build().as_slice());
    mesh.set_texture(tex, obj.build_tex_coord().as_slice());
    mesh.prepare(shader);
    mesh
}

fn add_mesh(meshes: &mut Vec<Mesh>, mesh: Mesh) -> MeshId {
    meshes.push(mesh);
    MeshId {
        id: (meshes.len() as MInt) - 1,
    }
}

fn get_initial_camera_pos(map_size: &Size2<MInt>) -> WorldPos {
    let pos = get_max_camera_pos(map_size);
    WorldPos {
        v: Vector3 {
            x: pos.v.x / 2.0,
            y: pos.v.y / 2.0,
            z: 0.0,
        },
    }
}

fn get_max_camera_pos(map_size: &Size2<MInt>) -> WorldPos {
    let pos = geom::map_pos_to_world_pos(MapPos {
        v: Vector2 {
            x: map_size.w,
            y: map_size.h,
        },
    });
    WorldPos {
        v: Vector3 {
            x: -pos.v.x,
            y: -pos.v.y,
            z: 0.0,
        },
    }
}

fn get_marker_mesh_id(mesh_ids: &MeshIdManager, player_id: PlayerId) -> MeshId {
    match player_id.id {
        0 => mesh_ids.marker_1_mesh_id,
        1 => mesh_ids.marker_2_mesh_id,
        n => panic!("Wrong player id: {}", n),
    }
}

fn get_unit_mesh_id(
    unit_type_visual_info: &UnitTypeVisualInfoManager,
    unit_type_id: UnitTypeId,
) -> MeshId {
    unit_type_visual_info.get(unit_type_id).mesh_id
}

struct MeshIdManager {
    map_mesh_id: MeshId,
    shell_mesh_id: MeshId,
    marker_1_mesh_id: MeshId,
    marker_2_mesh_id: MeshId,
}

pub struct GameStateVisualizer {
    mesh_ids: MeshIdManager,
    unit_type_visual_info: UnitTypeVisualInfoManager,
    meshes: Vec<Mesh>,
    walkable_mesh: Option<Mesh>,
    // TODO: move to 'meshes'
    map_text_mesh: Mesh,
    camera: Camera,
    commands_rx: Receiver<StateChangeCommand>,
    commands_tx: Sender<StateChangeCommand>,
    picker: picker::TilePicker,
    map_pos_under_cursor: Option<MapPos>,
    selected_unit_id: Option<UnitId>,
    unit_under_cursor_id: Option<UnitId>,
    scenes: HashMap<PlayerId, Scene>,
    core: Core,
    event: Option<Event>,
    event_visualizer: Option<Box<dyn EventVisualizer + 'static>>,
    game_states: HashMap<PlayerId, GameState>,
    pathfinders: HashMap<PlayerId, Pathfinder>,
    button_manager: ButtonManager,
    button_end_turn_id: ButtonId,
    button_quit_id: ButtonId,
    selection_manager: SelectionManager,
}

fn get_unit_type_visual_info(
    fs: &FileSystem,
    context: &Context,
    meshes: &mut Vec<Mesh>,
) -> UnitTypeVisualInfoManager {
    let tank_mesh_id = add_mesh(meshes, load_unit_mesh(fs, &context.shader, "tank"));
    let soldier_mesh_id = add_mesh(meshes, load_unit_mesh(fs, &context.shader, "soldier"));
    let mut unit_type_visual_info = UnitTypeVisualInfoManager::new();
    // TODO: Add by name not by order
    unit_type_visual_info.add_info(UnitTypeVisualInfo {
        mesh_id: tank_mesh_id,
        move_speed: 3.8,
    });
    unit_type_visual_info.add_info(UnitTypeVisualInfo {
        mesh_id: soldier_mesh_id,
        move_speed: 2.0,
    });
    unit_type_visual_info
}

impl GameStateVisualizer {
    pub fn new(fs: &FileSystem, context: &Context) -> GameStateVisualizer {
        // set_error_context!("constructing GameStateVisualizer", "-");
        let players_count = 2;
        let core = Core::new(fs);
        let map_size = core.map_size();
        let game_states = get_game_states(players_count);
        let picker = picker::TilePicker::new(fs, &game_states[&core.player_id()], core.map_size());
        let mut meshes = Vec::new();
        let map_mesh_id = add_mesh(&mut meshes, get_map_mesh(fs, map_size, &context.shader));
        let selection_marker_mesh_id =
            add_mesh(&mut meshes, get_selection_mesh(fs, &context.shader));
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
        camera.set_max_pos(get_max_camera_pos(&map_size));
        camera.set_pos(get_initial_camera_pos(&map_size));
        let mut button_manager = ButtonManager::new();
        let button_end_turn_id = button_manager.add_button(Button::new(
            "end turn",
            context.font_stash.borrow_mut().deref_mut(),
            &context.shader,
            ScreenPos {
                v: Vector2 { x: 10, y: 40 },
            },
        ));
        let button_quit_id = button_manager.add_button(Button::new(
            "quit",
            context.font_stash.borrow_mut().deref_mut(),
            &context.shader,
            ScreenPos {
                v: Vector2 { x: 10, y: 10 },
            },
        ));
        let map_text_mesh = context
            .font_stash
            .borrow_mut()
            .deref_mut()
            .get_mesh("test text", &context.shader);
        // TODO: store this info in separate json
        let mesh_ids = MeshIdManager {
            map_mesh_id,
            shell_mesh_id,
            marker_1_mesh_id,
            marker_2_mesh_id,
        };
        let (commands_tx, commands_rx) = channel();
        let vis = GameStateVisualizer {
            walkable_mesh: None,
            unit_type_visual_info: get_unit_type_visual_info(fs, context, &mut meshes),
            mesh_ids,
            meshes,
            map_text_mesh,
            camera,
            picker,
            map_pos_under_cursor: None,
            selected_unit_id: None,
            unit_under_cursor_id: None,
            core,
            event_visualizer: None,
            event: None,
            scenes: get_scenes(players_count),
            game_states,
            pathfinders: get_pathfinders(players_count, map_size),
            button_manager,
            button_end_turn_id,
            button_quit_id,
            selection_manager: SelectionManager::new(selection_marker_mesh_id),
            commands_rx,
            commands_tx,
        };
        vis
    }

    fn scene(&self) -> &Scene {
        &self.scenes[&self.core.player_id()]
    }

    fn draw_scene_node(&self, node: &SceneNode, m: Matrix4<MFloat>, context: &Context) {
        let m = mgl::tr(m, node.pos.v);
        let m = mgl::rot_z(m, node.rot);
        match node.mesh_id {
            Some(mesh_id) => {
                context.shader.uniform_mat4f(context.mvp_mat_id.clone(), &m);
                let id = mesh_id.id as usize;
                self.meshes[id].draw(&context.shader);
            }
            None => {}
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
        context
            .shader
            .uniform_mat4f(context.mvp_mat_id.clone(), &self.camera.mat());
        self.meshes[self.mesh_ids.map_mesh_id.id as usize].draw(&context.shader);
    }

    fn draw_3d_text(&mut self, context: &Context) {
        let font_stash = context.font_stash.borrow_mut();
        let m = self.camera.mat();
        let m = mgl::scale(m, 1.0 / font_stash.get_size());
        let m = mgl::rot_x(m, 90.0);
        context.shader.uniform_mat4f(context.mvp_mat_id.clone(), &m);
        self.map_text_mesh.draw(&context.shader);
    }

    fn draw_scene(&mut self, context: &Context, dtime: Time) {
        context
            .shader
            .uniform_color(context.basic_color_id.clone(), mgl::WHITE);
        self.draw_scene_nodes(context);
        self.draw_map(context);
        match self.walkable_mesh {
            Some(ref walkable_mesh) => {
                context
                    .shader
                    .uniform_color(context.basic_color_id.clone(), mgl::BLUE);
                walkable_mesh.draw(&context.shader);
            }
            None => {}
        }
        match self.event_visualizer {
            Some(ref mut event_visualizer) => {
                let scene = self.scenes.get_mut(&self.core.player_id()).unwrap();
                event_visualizer.draw(scene, dtime);
            }
            None => {}
        }
    }

    fn end_turn(&mut self) {
        self.core.do_command(CommandEndTurn);
        self.selected_unit_id = None;
        let scene = self.scenes.get_mut(&self.core.player_id());
        self.selection_manager.deselect(scene.unwrap());
        self.walkable_mesh = None;
    }

    fn is_tile_occupied(&self, pos: MapPos) -> bool {
        let state = &self.game_states[&self.core.player_id()];
        state.units_at(pos).len() > 0
    }

    fn create_unit(&mut self) {
        match self.map_pos_under_cursor {
            Some(pos) => {
                if self.is_tile_occupied(pos) {
                    return;
                }
                let cmd = CommandCreateUnit(pos);
                self.core.do_command(cmd);
            }
            None => {}
        }
    }

    fn attack_unit(&mut self) {
        match (self.unit_under_cursor_id, self.selected_unit_id) {
            (Some(defender_id), Some(attacker_id)) => {
                let state = &self.game_states[&self.core.player_id()];
                let attacker = &state.units[&attacker_id];
                if attacker.attacked {
                    return;
                }
                let defender = &state.units[&defender_id];
                let max_distance = {
                    let attacker_type = self.core.object_types().get_unit_type(attacker.type_id);
                    let weapon_type = self.core.get_weapon_type(attacker_type.weapon_type_id);
                    weapon_type.max_distance
                };
                if distance(attacker.pos, defender.pos) > max_distance {
                    return;
                }
                let cmd = CommandAttackUnit(attacker_id, defender_id);
                self.core.do_command(cmd);
            }
            _ => {}
        }
    }

    fn select_unit(&mut self, context: &Context) {
        match self.unit_under_cursor_id {
            Some(unit_id) => {
                self.selected_unit_id = Some(unit_id);
                let state = &self.game_states[&self.core.player_id()];
                let pf = self.pathfinders.get_mut(&self.core.player_id()).unwrap();
                pf.fill_map(state, &state.units[&unit_id]);
                self.walkable_mesh = Some(build_walkable_mesh(pf, &context.shader));
                let scene = self.scenes.get_mut(&self.core.player_id()).unwrap();
                self.selection_manager
                    .create_selection_marker(state, scene, unit_id);
                // TODO: highlight potential targets
            }
            None => {}
        }
    }

    fn handle_key_event(&mut self, _: &Context, key: glfw::Key) {
        match key {
            glfw::Key::Escape | glfw::Key::Q => self.commands_tx.send(EndGame).unwrap(),
            glfw::Key::Up | glfw::Key::W => self.camera.move_camera(270.0, 0.1),
            glfw::Key::Down | glfw::Key::S => self.camera.move_camera(90.0, 0.1),
            glfw::Key::Right | glfw::Key::D => self.camera.move_camera(0.0, 0.1),
            glfw::Key::Left | glfw::Key::A => self.camera.move_camera(180.0, 0.1),
            glfw::Key::Minus => self.camera.change_zoom(1.3),
            glfw::Key::Equal => self.camera.change_zoom(0.7),
            _ => {}
        }
        if self.event_visualizer.is_some() {
            return;
        }
        match key {
            glfw::Key::T => self.end_turn(),
            glfw::Key::U => self.create_unit(),
            _ => {}
        }
    }

    fn handle_cursor_pos_event(&mut self, context: &Context, new_pos: ScreenPos) {
        let rmb = context.win.get_mouse_button(glfw::MouseButtonRight);
        if rmb == glfw::Action::Press {
            let diff = context.mouse_pos.v - new_pos.v;
            let win_w = context.win_size.w as MFloat;
            let win_h = context.win_size.h as MFloat;
            self.camera.add_z_angle(diff.x as MFloat * (360.0 / win_w));
            self.camera.add_x_angle(diff.y as MFloat * (360.0 / win_h));
        }
    }

    fn move_unit(&mut self) {
        let pos = self.map_pos_under_cursor.unwrap();
        let unit_id = match self.selected_unit_id {
            Some(unit_id) => unit_id,
            None => return,
        };
        if self.is_tile_occupied(pos) {
            return;
        }
        let state = &self.game_states[&self.core.player_id()];
        let unit = &state.units[&unit_id];
        if unit.move_points == 0 {
            return;
        }
        let pf = self.pathfinders.get_mut(&self.core.player_id()).unwrap();
        let path = pf.get_path(pos);
        if path.len() < 2 {
            return;
        }
        self.core.do_command(CommandMove(unit_id, path));
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
                    self.commands_tx.send(EndGame).unwrap();
                } else {
                    println!("Clicked on {} at {}", button_id.id, precise_time_ns());
                }
                return;
            }
            None => {}
        }
        if self.map_pos_under_cursor.is_some() {
            self.move_unit();
        }
        match self.unit_under_cursor_id {
            Some(unit_under_cursor_id) => {
                let player_id = {
                    let state = &self.game_states[&self.core.player_id()];
                    let unit = &state.units[&unit_under_cursor_id];
                    unit.player_id.clone()
                };
                if player_id == self.core.player_id() {
                    self.select_unit(context);
                } else {
                    self.attack_unit();
                }
            }
            None => {}
        }
    }

    fn pick_tile(&mut self, context: &Context) {
        let pick_result =
            self.picker
                .pick_tile(&self.camera, context.win_size, context.mouse_pos.clone());
        match pick_result {
            PickedMapPos(pos) => {
                self.map_pos_under_cursor = Some(pos);
                self.unit_under_cursor_id = None;
            }
            PickedUnitId(id) => {
                self.map_pos_under_cursor = None;
                self.unit_under_cursor_id = Some(id);
            }
            _ => {}
        }
    }

    fn make_event_visualizer(&mut self, event: &Event) -> Box<dyn EventVisualizer + 'static> {
        let player_id = self.core.player_id();
        let scene = self.scenes.get_mut(&player_id).unwrap();
        let state = &self.game_states[&player_id];
        match *event {
            EventMove(unit_id, ref path) => {
                let type_id = state.units[&unit_id].type_id;
                let unit_type_visual_info = self.unit_type_visual_info.get(type_id);
                EventMoveVisualizer::new(scene, state, unit_id, unit_type_visual_info, path.clone())
            }
            EventEndTurn(_, _) => EventEndTurnVisualizer::new(),
            EventCreateUnit(id, ref pos, ref type_id, ref player_id) => {
                EventCreateUnitVisualizer::new(
                    &self.core,
                    scene,
                    state,
                    id,
                    *type_id,
                    *pos,
                    get_unit_mesh_id(&self.unit_type_visual_info, type_id.clone()),
                    get_marker_mesh_id(&self.mesh_ids, player_id.clone()),
                )
            }
            EventAttackUnit(attacker_id, defender_id, killed) => EventAttackUnitVisualizer::new(
                scene,
                state,
                attacker_id,
                defender_id,
                killed,
                self.mesh_ids.shell_mesh_id,
            ),
        }
    }

    fn start_event_visualization(&mut self, event: Event) {
        let vis = self.make_event_visualizer(&event);
        self.event = Some(event);
        self.event_visualizer = Some(vis);
    }

    fn end_event_visualization(&mut self, context: &Context) {
        let scene = self.scenes.get_mut(&self.core.player_id()).unwrap();
        let state = self.game_states.get_mut(&self.core.player_id()).unwrap();
        self.event_visualizer.as_mut().unwrap().end(scene, state);
        state.apply_event(self.core.object_types(), self.event.as_ref().unwrap());
        self.event_visualizer = None;
        self.event = None;
        match self.selected_unit_id {
            Some(selected_unit_id) => {
                let pf = self.pathfinders.get_mut(&self.core.player_id()).unwrap();
                pf.fill_map(state, &state.units[&selected_unit_id]);
                self.walkable_mesh = Some(build_walkable_mesh(pf, &context.shader));
                self.selection_manager.move_selection_marker(state, scene);
            }
            None => {}
        }
        self.picker.update_units(state);
    }
}

impl StateVisualizer for GameStateVisualizer {
    fn logic(&mut self, context: &Context) {
        if self.event_visualizer.is_none() {
            match self.core.get_event() {
                Some(e) => self.start_event_visualization(e),
                None => {}
            }
        } else if self.event_visualizer.as_ref().unwrap().is_finished() {
            self.end_event_visualization(context);
        }
    }

    fn draw(&mut self, context: &mut Context, dtime: Time) {
        self.pick_tile(context);
        mgl::set_clear_color(mgl::GREY_3);
        mgl::clear_screen();
        context.shader.activate();
        self.draw_scene(context, dtime);
        context
            .shader
            .uniform_color(context.basic_color_id, mgl::BLACK);
        self.draw_3d_text(context);
        self.button_manager.draw(context);
        use glfw::Context;
        context.win.swap_buffers();
    }

    fn handle_event(&mut self, context: &Context, event: glfw::WindowEvent) {
        match event {
            glfw::WindowEvent::Key(key, _, glfw::Action::Press, _) => {
                self.handle_key_event(context, key);
            }
            glfw::WindowEvent::CursorPos(x, y) => {
                let p = ScreenPos {
                    v: Vector2 {
                        x: x as MInt,
                        y: y as MInt,
                    },
                };
                self.handle_cursor_pos_event(context, p);
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, glfw::Action::Press, _) => {
                self.handle_mouse_button_event(context);
            }
            glfw::WindowEvent::Size(w, h) => {
                self.camera.regenerate_projection_mat(Size2 { w, h });
            }
            _ => {}
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
