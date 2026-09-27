use ab_glyph::{Font as _, FontArc, point};
use eframe::egui::FontDefinitions;

pub(crate) const CELL_SIZE: u32 = 32;
pub(crate) const COLUMNS: u32 = 16;
pub(crate) const ROWS: u32 = 6;
pub(crate) const WIDTH: u32 = CELL_SIZE * COLUMNS;
pub(crate) const HEIGHT: u32 = CELL_SIZE * ROWS;

pub(crate) fn glyph_uv(character: char) -> [[f32; 2]; 2] {
    let character = if (' '..='~').contains(&character) {
        character
    } else {
        '?'
    };
    let index = character as u32 - ' ' as u32;
    let cell_x = index % COLUMNS;
    let cell_y = index / COLUMNS;
    let inset = 1.0;
    let dimensions = [WIDTH as f32, HEIGHT as f32];
    let min = [
        (cell_x * CELL_SIZE) as f32 + inset,
        (cell_y * CELL_SIZE) as f32 + inset,
    ];
    let max = [
        ((cell_x + 1) * CELL_SIZE) as f32 - inset,
        ((cell_y + 1) * CELL_SIZE) as f32 - inset,
    ];
    [
        [min[0] / dimensions[0], min[1] / dimensions[1]],
        [max[0] / dimensions[0], max[1] / dimensions[1]],
    ]
}

pub(crate) fn rasterize_atlas() -> Vec<u8> {
    let definitions = FontDefinitions::default();
    let font_data = definitions
        .font_data
        .get("Hack")
        .expect("egui default fonts include Hack");
    let font = FontArc::try_from_vec(font_data.font.as_ref().to_vec())
        .expect("egui Hack font data is valid");
    let mut pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];

    for character_code in ' '..='~' {
        let index = character_code as u32 - ' ' as u32;
        let cell_x = (index % COLUMNS) * CELL_SIZE;
        let cell_y = (index / COLUMNS) * CELL_SIZE;
        let initial = font.outline_glyph(
            font.glyph_id(character_code)
                .with_scale_and_position(24.0, point(0.0, 0.0)),
        );
        let Some(initial) = initial else {
            continue;
        };
        let bounds = initial.px_bounds();
        let left = cell_x as f32 + (CELL_SIZE as f32 - bounds.width()) * 0.5 - bounds.min.x;
        let top = cell_y as f32 + (CELL_SIZE as f32 - bounds.height()) * 0.5 - bounds.min.y;
        let glyph = font
            .outline_glyph(
                font.glyph_id(character_code)
                    .with_scale_and_position(24.0, point(left, top)),
            )
            .expect("glyph outline was present above");

        glyph.draw(|x, y, coverage| {
            let offset = (((cell_y + y) * WIDTH + cell_x + x) * 4) as usize;
            pixels[offset..offset + 3].fill(255);
            pixels[offset + 3] = (coverage * 255.0).round() as u8;
        });
    }

    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_has_visible_printable_glyphs() {
        let pixels = rasterize_atlas();
        assert_eq!(pixels.len(), (WIDTH * HEIGHT * 4) as usize);
        assert!(pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }

    #[test]
    fn unsupported_characters_use_question_mark_cell() {
        assert_eq!(glyph_uv('\u{2603}'), glyph_uv('?'));
    }
}
