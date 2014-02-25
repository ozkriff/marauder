// See LICENSE file for copyright and license details.

use cgmath::vector::{
    Vec3,
    Vec2,
};
use gl_helpers::{
    Shader,
    uniform_mat4f,
    set_clear_color,
    clear,
    get_vec2_from_pixel,
};
use map::MapPosIter;
use camera::Camera;
use geom::Geom;
use mesh::Mesh;
use core_types::{
    Int,
    Size2,
    MapPos,
};
use gl_types::{
    VertexCoord,
    Color3,
    Float,
    MatId,
};

fn build_hex_map_mesh(
    geom: &Geom,
    map_size: Size2<Int>
) -> (~[VertexCoord], ~[Color3]) {
    let mut c_data = ~[];
    let mut v_data = ~[];
    for tile_pos in MapPosIter::new(map_size) {
        let pos3d = geom.map_pos_to_world_pos(tile_pos);
        for num in range(0 as Int, 6) {
            let vertex = geom.index_to_hex_vertex(num);
            let next_vertex = geom.index_to_hex_vertex(num + 1);
            let col_x = tile_pos.x as Float / 255.0;
            let col_y = tile_pos.y as Float / 255.0;
            let color = Color3{r: col_x, g: col_y, b: 1.0};
            v_data.push(pos3d + vertex);
            c_data.push(color);
            v_data.push(pos3d + next_vertex);
            c_data.push(color);
            v_data.push(pos3d + Vec3::zero());
            c_data.push(color);
        }
    }
    (v_data, c_data)
}

pub struct TilePicker {
    shader: Shader,
    map_mesh: Mesh,
    mat_id: MatId,
    win_size: Size2<Int>,
}

impl TilePicker {
    pub fn new(
        win_size: Size2<Int>,
        geom: &Geom,
        map_size: Size2<Int>
    ) -> ~TilePicker {
        let mut picker = ~TilePicker {
            shader: Shader(0),
            map_mesh: Mesh::new(),
            mat_id: MatId(0),
            win_size: win_size,
        };
        picker.init(geom, map_size);
        picker
    }

    pub fn set_win_size(&mut self, win_size: Size2<Int>) {
        self.win_size = win_size;
    }

    fn init(&mut self, geom: &Geom, map_size: Size2<Int>) {
        self.shader = Shader::new("pick.vs.glsl", "pick.fs.glsl");
        self.shader.activate();
        let position_attr = self.shader.get_attr("in_vertex_coordinates");
        let color_attr = self.shader.get_attr("color");
        position_attr.enable();
        color_attr.enable();
        position_attr.vertex_pointer(3);
        color_attr.vertex_pointer(3);
        let (vertex_data, color_data) = build_hex_map_mesh(geom, map_size);
        self.map_mesh.set_vertex_coords(vertex_data);
        self.map_mesh.set_color(color_data);
        self.mat_id = MatId(self.shader.get_uniform("mvp_mat"));
    }

    pub fn pick_tile(
        &mut self,
        camera: &Camera,
        mouse_pos: Vec2<Int>
    ) -> Option<MapPos> {
        self.shader.activate();
        uniform_mat4f(self.mat_id, &camera.mat());
        set_clear_color(0.0, 0.0, 0.0);
        clear();
        self.map_mesh.draw(&self.shader);
        get_vec2_from_pixel(self.win_size, mouse_pos)
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
