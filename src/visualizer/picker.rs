// See LICENSE file for copyright and license details.

use cgmath::vector::{Vector2};
use core::map::MapPosIter;
use core::types::{MInt, Size2, MapPos, UnitId};
use core::fs::FileSystem;
use core::game_state::GameState;
use visualizer::mgl;
use visualizer::camera::Camera;
use visualizer::geom;
use visualizer::mesh::Mesh;
use visualizer::types::{Color3, MFloat, MatId, VertexCoord};
use visualizer::shader::Shader;

fn i_to_f(n: MInt) -> f32 {
    n as MFloat / 255.0
}

pub enum PickResult {
    PickedMapPos(MapPos),
    PickedUnitId(UnitId),
    PickedNothing
}

pub struct TilePicker {
    shader: Shader,
    mesh: Mesh,
    mvp_mat_id: MatId,
    map_size: Size2<MInt>,
}

fn tile_color(state: &GameState, pos: MapPos) -> Color3 {
    let mut unit = None;
    for (_, unit2) in state.units.iter() {
        if unit2.pos == pos {
            unit = Some(unit2);
        }
    }
    match unit {
        Some(unit) => {
            Color3{r: i_to_f(unit.id.id), g: 0.0, b: i_to_f(2)}
        },
        None => {
            let col_x = i_to_f(pos.v.x);
            let col_y = i_to_f(pos.v.y);
            Color3{r: col_x, g: col_y, b: i_to_f(1)}
        },
    }
}

fn get_mesh(state: &GameState, map_size: Size2<MInt>, shader: &Shader) -> Mesh {
    let mut c_data = Vec::new();
    let mut v_data = Vec::new();
    for tile_pos in MapPosIter::new(map_size) {
        let pos3d = geom::map_pos_to_world_pos(tile_pos);
        for num in range(0 as MInt, 6) {
            let vertex = geom::index_to_hex_vertex(num);
            let next_vertex = geom::index_to_hex_vertex(num + 1);
            let color = tile_color(state, tile_pos);
            v_data.push(VertexCoord{v: pos3d.v + vertex.v});
            c_data.push(color);
            v_data.push(VertexCoord{v: pos3d.v + next_vertex.v});
            c_data.push(color);
            v_data.push(VertexCoord{v: pos3d.v});
            c_data.push(color);
        }
    }
    let mut mesh = Mesh::new(v_data.as_slice());
    mesh.set_color(c_data.as_slice());
    mesh.prepare(shader);
    mesh
}

impl TilePicker {
    pub fn new(fs: &FileSystem, state: &GameState, map_size: Size2<MInt>) -> TilePicker {
        let shader = Shader::new(
            &fs.get(&Path::new("data/pick.vs.glsl")),
            &fs.get(&Path::new("data/pick.fs.glsl")),
        );
        let mvp_mat_id = MatId{id: shader.get_uniform("mvp_mat")};
        let mesh = get_mesh(state, map_size, &shader);
        let tile_picker = TilePicker {
            mesh: mesh,
            shader: shader,
            mvp_mat_id: mvp_mat_id,
            map_size: map_size,
        };
        tile_picker
    }

    pub fn update_units(&mut self, state: &GameState) {
        self.mesh = get_mesh(state, self.map_size, &self.shader);
    }

    pub fn pick_tile(
        &mut self,
        camera: &Camera,
        win_size: Size2<MInt>,
        mouse_pos: Vector2<MInt>
    ) -> PickResult {
        self.shader.activate();
        self.shader.uniform_mat4f(self.mvp_mat_id, &camera.mat());
        mgl::set_clear_color(Color3{r: 0.0, g: 0.0, b: 0.0});
        mgl::clear_screen();
        self.mesh.draw(&self.shader);
        let (r, g, b, _) = mgl::read_pixel_bytes(win_size, mouse_pos);
        match b {
            0 => PickedNothing,
            1 => PickedMapPos(MapPos{v: Vector2{x: r, y: g}}),
            2 => PickedUnitId(UnitId{id: r}),
            n => fail!("Picker: bad color tag: {}", n),
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
