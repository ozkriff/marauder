// See LICENSE file for copyright and license details.

use cgmath::vector::{Vector, Vector3, Vector2};
use core::map::MapPosIter;
use core::types::{MInt, Size2, MapPos, UnitId};
use visualizer::gl_helpers::{
    set_clear_color,
    clear_screen,
    read_pixel_bytes,
};
use visualizer::camera::Camera;
use visualizer::geom::Geom;
use visualizer::mesh::Mesh;
use visualizer::types::{Color3, MFloat, MatId, WorldPos};
use visualizer::scene::Scene;
use visualizer::shader::Shader;

fn i_to_f(n: MInt) -> f32 {
    n as MFloat / 255.0
}

fn get_mesh(geom: &Geom, map_size: Size2<MInt>, shader: &Shader) -> Mesh {
    use std::slice::Vector;
    let mut c_data = Vec::new();
    let mut v_data = Vec::new();
    for tile_pos in MapPosIter::new(map_size) {
        let pos3d = geom.map_pos_to_world_pos(tile_pos);
        for num in range(0 as MInt, 6) {
            let vertex = geom.index_to_hex_vertex(num);
            let next_vertex = geom.index_to_hex_vertex(num + 1);
            let col_x = i_to_f(tile_pos.v.x);
            let col_y = i_to_f(tile_pos.v.y);
            let color = Color3{r: col_x, g: col_y, b: i_to_f(1)};
            v_data.push(pos3d + vertex);
            c_data.push(color);
            v_data.push(pos3d + next_vertex);
            c_data.push(color);
            v_data.push(pos3d + Vector3::zero());
            c_data.push(color);
        }
    }
    let mut mesh = Mesh::new(v_data.as_slice());
    mesh.set_color(c_data.as_slice());
    mesh.prepare(shader);
    mesh
}

pub enum PickResult {
    PickedMapPos(MapPos),
    PickedUnitId(UnitId),
    PickedNothing
}

pub struct TilePicker {
    shader: Shader,
    map_mesh: Mesh,
    units_mesh: Option<Mesh>,
    mvp_mat_id: MatId,
    win_size: Size2<MInt>,
}

impl TilePicker {
    pub fn new(
        win_size: Size2<MInt>,
        geom: &Geom,
        map_size: Size2<MInt>
    ) -> TilePicker {
        let shader = Shader::new(
            &Path::new("pick.vs.glsl"),
            &Path::new("pick.fs.glsl"),
        );
        let mvp_mat_id = MatId{id: shader.get_uniform("mvp_mat")};
        let map_mesh = get_mesh(geom, map_size, &shader);
        let tile_picker = TilePicker {
            map_mesh: map_mesh,
            units_mesh: None,
            shader: shader,
            mvp_mat_id: mvp_mat_id,
            win_size: win_size,
        };
        tile_picker
    }

    pub fn set_win_size(&mut self, win_size: Size2<MInt>) {
        self.win_size = win_size;
    }

    pub fn update_units( &mut self, geom: &Geom, scene: &Scene) {
        use std::slice::Vector;
        fn get_hex_vertex(geom: &Geom, n: MInt) -> WorldPos {
            let scale_factor = 0.5;
            geom.index_to_hex_vertex(n).mul_s(scale_factor)
        }
        let last_unit_node_id = 1000; // TODO
        let mut c_data = Vec::new();
        let mut v_data = Vec::new();
        for (node_id, node) in scene.nodes.iter() {
            let id = node_id.id;
            if id >= last_unit_node_id {
                continue;
            }
            let color = Color3 {r: i_to_f(id), g: 0.0, b: i_to_f(2)};
            for num in range(0 as MInt, 6) {
                v_data.push(node.pos + get_hex_vertex(geom, num));
                c_data.push(color);
                v_data.push(node.pos + get_hex_vertex(geom, num + 1));
                c_data.push(color);
                v_data.push(node.pos + Vector3::zero());
                c_data.push(color);
            }
        }
        // draw unit markers slightly above the floor
        let unit_marker_height = 0.01;
        for v in v_data.mut_iter() {
            v.z = unit_marker_height;
        }
        let mut mesh = Mesh::new(v_data.as_slice());
        mesh.set_color(c_data.as_slice());
        mesh.prepare(&self.shader);
        self.units_mesh = Some(mesh);
    }

    pub fn pick_tile(
        &mut self,
        camera: &Camera,
        mouse_pos: Vector2<MInt>
    ) -> PickResult {
        self.shader.activate();
        self.shader.uniform_mat4f(self.mvp_mat_id, &camera.mat());
        set_clear_color(Color3{r: 0.0, g: 0.0, b: 0.0});
        clear_screen();
        self.map_mesh.draw(&self.shader);
        match self.units_mesh {
            Some(ref units) => units.draw(&self.shader),
            None => {},
        };
        let (r, g, b, _) = read_pixel_bytes(self.win_size, mouse_pos);
        match b {
            0 => PickedNothing,
            1 => PickedMapPos(MapPos{v: Vector2{x: r, y: g}}),
            2 => PickedUnitId(UnitId{id: r}),
            n => fail!("Picker: bad color tag: {}", n),
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
