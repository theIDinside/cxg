use std::collections::HashMap;
use std::path::Path;

use crate::datastructure::generic::Vec2i;
use crate::debugger_catch;

/// Contains the texture coordinates & related glyph info about size & dimension
pub struct GlyphInfo {
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
    pub advance: i32,
    pub offsets: Vec2i,
    pub size: Vec2i,
    pub bearing: Vec2i,
}

impl GlyphInfo {
    pub fn width(&self) -> f32 {
        (self.x1 - self.x0) as f32
    }

    pub fn height(&self) -> f32 {
        (self.y1 - self.y0) as f32
    }
}

#[allow(unused)]
pub struct Font {
    pixel_size: i32,
    row_height: i32,
    max_glyph_dimensions: Vec2i,
    max_bearing_size_diff: i32,
    glyph_cache: HashMap<char, GlyphInfo>,
    pixel_data: Vec<u8>,
    texture_id: gl::types::GLuint,
    texture_dimensions: Vec2i,
}

fn debug_write_font_texture_to_file(font_path: &Path, pixels: &Vec<u8>, pixel_size: i32, tex_width: u32, tex_height: u32) {
    use std::fs::File;
    use std::io::BufWriter;
    println!("we are in debug mode");
    let mut png_data: Vec<u8> = Vec::with_capacity(pixels.len() * 4);

    for _p in pixels.iter() {
        let p = *_p;
        png_data.extend_from_slice(&[p, p, p, 0xff]);
    }

    let font_file_name = format!("{}_{}", font_path.file_stem().unwrap().to_str().unwrap(), pixel_size);
    let mut output_file = std::path::PathBuf::new();

    output_file.push("./");
    output_file.push("debug");
    if !output_file.exists() {
        std::fs::create_dir("./debug").unwrap();
    }
    output_file.push(font_file_name);
    output_file.set_extension("png");
    println!("Path: {}", &output_file.display());
    let path = output_file.as_path();
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, tex_width, tex_height);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&png_data).unwrap(); // Save
    println!("Wrote to file {}", path.display());
}

// fn debug_write_font_texture_to_file(_font_path: &Path, _pixels: &Vec<u8>, _pixel_size: i32, _tex_width: u32, _tex_height: u32) {}

impl Font {
    pub fn new(font_path: &Path, pixel_size: i32, characters: Vec<char>) -> Result<Font, ft::Error> {
        let lib = ft::Library::init()?;
        let face = lib.new_face(font_path, 0)?;
        face.set_pixel_sizes(pixel_size as u32, pixel_size as u32)?;
        let glyph_count = characters.len() as f64;
        let max_dim = ((1 + face.size_metrics().unwrap().height >> 6) as f64 * glyph_count.sqrt().ceil()) as i32;

        let mut texture_dimension = Vec2i { x: 1, y: 1 };
        while texture_dimension.x < max_dim {
            texture_dimension.x = texture_dimension.x << 1;
        }
        texture_dimension.y = texture_dimension.x;
        let mut pixels = Vec::new();
        pixels.resize((texture_dimension.x * texture_dimension.y) as usize, 0);

        let mut pen_x = 0;
        let mut pen_y = 0;
        let mut max_glyph_dimensions = Vec2i { x: 0, y: 0 };
        let mut max_bearing_size_diff = 0;
        let mut glyph_cache: HashMap<char, GlyphInfo> = HashMap::new();

        for c in characters {
            face.load_char(c as usize, ft::face::LoadFlag::RENDER | ft::face::LoadFlag::FORCE_AUTOHINT | ft::face::LoadFlag::TARGET_LIGHT | ft::face::LoadFlag::COLOR)?;
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            max_glyph_dimensions.y = std::cmp::max(bitmap.rows(), max_glyph_dimensions.x);
            max_glyph_dimensions.x = std::cmp::max(bitmap.width(), max_glyph_dimensions.x);

            if pen_x + bitmap.width() >= texture_dimension.x {
                pen_x = 0;
                pen_y += (face.size_metrics().unwrap().height >> 6) as i32 + 1;
            }

            for row in 0..bitmap.rows() {
                for col in 0..bitmap.width() {
                    let x = pen_x + col;
                    let y = pen_y + row;
                    let mut pixel_index = (y * texture_dimension.x + x) as usize;
                    let bitmap_index = (row * bitmap.pitch() + col) as usize;
                    if pixel_index >= pixels.len() {
                        debugger_catch!(!(pixel_index >= 262144), crate::DebuggerCatch::Handle("Pixel index must remaing below 262144".into()));
                        pixel_index = pixels.len() - 1;
                    }
                    pixels[pixel_index] 
                    = bitmap.buffer()[bitmap_index];
                }
            }

            let glyph_info = GlyphInfo {
                x0: pen_x,
                x1: pen_x + bitmap.width(),
                y0: pen_y,
                y1: pen_y + bitmap.rows(),
                advance: glyph.advance().x as i32 >> 6,
                offsets: Vec2i { x: glyph.bitmap_left(), y: glyph.bitmap_top() },
                size: Vec2i { x: bitmap.width(), y: bitmap.rows() },
                bearing: Vec2i { x: glyph.bitmap_left(), y: glyph.bitmap_top() },
            };
            max_bearing_size_diff = std::cmp::max((glyph_info.size.y - glyph_info.bearing.y).abs(), max_bearing_size_diff);
            glyph_cache.insert(c, glyph_info);
            pen_x += bitmap.width() + 1;
        }
        let max_adv_y = max_glyph_dimensions.y + 5;
        let row_advance = max_adv_y;

        let texture_id = unsafe { Font::upload_texture(&pixels, texture_dimension.x, texture_dimension.y) };

        debug_write_font_texture_to_file(font_path, &pixels, pixel_size, texture_dimension.x as u32, texture_dimension.y as u32);

        Ok(Font {
            pixel_size,
            row_height: row_advance,
            max_glyph_dimensions,
            max_bearing_size_diff,
            pixel_data: pixels,
            texture_id,
            glyph_cache,
            texture_dimensions: texture_dimension,
        })
    }

    unsafe fn upload_texture(data: &Vec<u8>, width: i32, height: i32) -> gl::types::GLuint {
        let mut id = 0;
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D, id);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as i32, width, height, 0, gl::RED, gl::UNSIGNED_BYTE, data.as_ptr() as *const _);
        gl::GenerateMipmap(gl::TEXTURE_2D);
        id
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
        }
    }

    pub fn get_glyph(&self, character: char) -> Option<&GlyphInfo> {
        self.glyph_cache.get(&character)
    }

    pub fn texture_width(&self) -> i32 {
        self.texture_dimensions.x
    }

    pub fn texture_height(&self) -> i32 {
        self.texture_dimensions.y
    }

    pub fn row_height(&self) -> i32 {
        self.row_height
    }

    pub fn get_max_glyph_width(&self) -> i32 {
        let mut w = 0;
        for (_, g) in self.glyph_cache.iter() {
            w = std::cmp::max(g.size.x, w);
        }
        w
    }
}
