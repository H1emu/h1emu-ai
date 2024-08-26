use binrw::binrw;
use binrw::{
    io::{Read, Seek},
    BinRead, BinResult,
};

#[binrw]
#[derive(Debug)]
struct Texture {
    color_nx_map_data_len: u32,
    #[br(count = color_nx_map_data_len)]
    color_nx_map: Vec<u8>,

    spec_ny_map_data_len: u32,
    #[br(count = spec_ny_map_data_len)]
    spec_ny_map: Vec<u8>,

    extradata1_data_len: u32,
    #[br(count = extradata1_data_len)]
    extradata1: Vec<u8>,

    extradata2_data_len: u32,
    #[br(count = extradata2_data_len)]
    extradata2: Vec<u8>,

    extradata3_data_len: u32,
    #[br(count = extradata3_data_len)]
    extradata3: Vec<u8>,

    extradata4_data_len: u32,
    #[br(count = extradata4_data_len)]
    extradata4: Vec<u8>,
}

#[binrw]
#[derive(Debug)]
struct HeightMap {
    datalen: u32,
    #[br(count = datalen * 4)]
    bytes: Vec<u8>,
}

#[binrw]
#[derive(Debug)]
pub struct Vertices {
    pub x: u16,
    pub y: u16,
    pub hf: i16,
    pub hn: i16,
    color: u32,
}

#[binrw]
#[derive(Debug)]
pub struct Batches {
    pub unk: u32,
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
    pub vertex_count: u32,
}

#[binrw]
#[derive(Debug)]
struct TileInfo {
    info: [u8; 64],
}

#[binrw]
#[derive(Debug)]
struct Draws {
    info: [u8; 320],
}
#[binrw]
#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [i32; 3], // Indices into a vertex list
    pub neighbors_count: u32,
    #[br(count = neighbors_count)]
    pub neighbors: Vec<i32>, // Indices of adjacent triangles
}
#[binrw]
#[derive(Debug, Default, Clone, Copy)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub start_poly_index: u32,
    pub end_poly_index: u32,
}

#[binrw]
#[derive(Debug, Default, Clone)]
pub struct NavData {
    pub cells_count: u32,
    #[br(count = cells_count)]
    pub cells: Vec<Cell>,
    pub polygons_count: u32,
    #[br(count = polygons_count)]
    pub polygons: Vec<Triangle>,
}

#[binrw]
#[derive(Debug)]
pub struct ChunkH1z1 {
    chunk_type: [u8; 4],
    version: u32,

    textures_count: u32,
    #[br(count = textures_count)]
    textures: Vec<Texture>,

    pub verts_per_side: u32,

    hmap: HeightMap,

    indices_count: u32,
    #[br(count = indices_count)]
    pub indices: Vec<u16>,

    vertices_count: u32,
    #[br(count = vertices_count)]
    pub vertices: Vec<Vertices>,

    batches_count: u32,
    #[br(count = batches_count)]
    pub batches: Vec<Batches>,

    draw_count: u32,
    #[br(count = draw_count)]
    optimized_draw: Vec<Draws>,

    shorts_count: u32,
    #[br(count = shorts_count)]
    shorts: Vec<u16>,

    vec_count: u32,
    #[br(count = vec_count * 3)]
    vectors: Vec<f32>,

    tileinfo_count: u32,
    #[br(count = tileinfo_count)]
    tileinfo: Vec<TileInfo>,
}
