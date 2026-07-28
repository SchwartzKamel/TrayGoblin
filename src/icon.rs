//! Platform-neutral notification-area icon rendering.
//!
//! Each status state has its own silhouette *and* its own color, so the tray
//! stays readable for color-blind users and on high-contrast themes, as
//! required by `specs/CONSTITUTION.md` §UX & consistency. Glyphs live in
//! `assets/` as plain-text pixel maps so they can be reviewed, diffed, and
//! rasterized on any host without an image toolchain.

use std::fmt;

use crate::status::StatusState;

/// Edge length, in pixels, of every rendered tray icon.
pub const ICON_SIZE: u32 = 32;

const IDLE_GLYPH: &str = include_str!("../assets/icon-idle.txt");
const WORKING_GLYPH: &str = include_str!("../assets/icon-working.txt");
const ATTENTION_GLYPH: &str = include_str!("../assets/icon-attention.txt");

const FILL_PIXEL: char = '#';
const OUTLINE_PIXEL: char = '+';
const TRANSPARENT_PIXEL: char = '.';
const COMMENT_PREFIX: char = '#';

/// Opaque 8-bit-per-channel color used by the glyph palettes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

/// Fill and outline colors for one icon variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IconPalette {
    pub fill: Rgb,
    pub outline: Rgb,
}

/// The three user-visible tray states. Named after the UI vocabulary in
/// `specs/CONTEXT.md` rather than the internal [`StatusState`] variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IconVariant {
    Idle,
    Working,
    AttentionNeeded,
}

impl IconVariant {
    /// Maps internal monitor state to the user-visible tray variant.
    pub fn for_state(state: StatusState) -> Self {
        match state {
            StatusState::Idle => Self::Idle,
            StatusState::Generating => Self::Working,
            StatusState::Error => Self::AttentionNeeded,
        }
    }

    /// Canonical UI label. Always paired with the icon so state is never
    /// communicated by color alone.
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Working => "Working",
            Self::AttentionNeeded => "Attention needed",
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Self::Idle => IDLE_GLYPH,
            Self::Working => WORKING_GLYPH,
            Self::AttentionNeeded => ATTENTION_GLYPH,
        }
    }

    pub fn palette(self) -> IconPalette {
        match self {
            Self::Idle => IconPalette {
                fill: Rgb::new(0x2E, 0xA0, 0x43),
                outline: Rgb::new(0x18, 0x5C, 0x27),
            },
            Self::Working => IconPalette {
                fill: Rgb::new(0xF5, 0xA6, 0x23),
                outline: Rgb::new(0x9A, 0x62, 0x05),
            },
            Self::AttentionNeeded => IconPalette {
                fill: Rgb::new(0xD1, 0x34, 0x38),
                outline: Rgb::new(0x7A, 0x16, 0x1A),
            },
        }
    }

    /// Renders this variant into a straight-alpha RGBA image.
    pub fn render(self) -> Result<IconImage, IconError> {
        render_glyph(self.glyph(), self.palette())
    }
}

/// A rasterized icon: top-down rows of straight-alpha RGBA pixels. Alpha is
/// always fully opaque or fully transparent, so premultiplied consumers may
/// use the same buffer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IconImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl IconImage {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    /// Top-down BGRA bytes, the layout expected by a Win32 32-bit DIB.
    pub fn bgra(&self) -> Vec<u8> {
        self.rgba
            .chunks_exact(4)
            .flat_map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]])
            .collect()
    }

    /// Opacity of each pixel, row-major. Used to compare silhouettes
    /// independently of color.
    pub fn silhouette(&self) -> Vec<bool> {
        self.rgba
            .chunks_exact(4)
            .map(|pixel| pixel[3] > 0)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IconError {
    EmptyGlyph,
    RaggedGlyph {
        row: usize,
        width: usize,
    },
    UnknownPixel {
        row: usize,
        column: usize,
        symbol: char,
    },
    NotSquare {
        width: usize,
        height: usize,
    },
}

impl fmt::Display for IconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGlyph => write!(f, "icon glyph contains no pixel rows"),
            Self::RaggedGlyph { row, width } => {
                write!(f, "icon glyph row {row} has an unexpected width of {width}")
            }
            Self::UnknownPixel {
                row,
                column,
                symbol,
            } => write!(
                f,
                "icon glyph row {row} column {column} uses unsupported symbol '{symbol}'"
            ),
            Self::NotSquare { width, height } => {
                write!(f, "icon glyph is {width}x{height} but must be square")
            }
        }
    }
}

impl std::error::Error for IconError {}

/// Rasterizes a text pixel map. Comment (`# `-prefixed header) and blank
/// lines are ignored so glyph files can document themselves.
fn render_glyph(glyph: &str, palette: IconPalette) -> Result<IconImage, IconError> {
    let rows: Vec<&str> = glyph
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !is_comment(line))
        .collect();

    let height = rows.len();
    let width = rows.first().ok_or(IconError::EmptyGlyph)?.chars().count();
    if width != height {
        return Err(IconError::NotSquare { width, height });
    }

    let mut rgba = Vec::with_capacity(width * height * 4);
    for (row_index, row) in rows.iter().enumerate() {
        let row_width = row.chars().count();
        if row_width != width {
            return Err(IconError::RaggedGlyph {
                row: row_index,
                width: row_width,
            });
        }

        for (column_index, symbol) in row.chars().enumerate() {
            let pixel = match symbol {
                FILL_PIXEL => [
                    palette.fill.red,
                    palette.fill.green,
                    palette.fill.blue,
                    0xFF,
                ],
                OUTLINE_PIXEL => [
                    palette.outline.red,
                    palette.outline.green,
                    palette.outline.blue,
                    0xFF,
                ],
                TRANSPARENT_PIXEL => [0, 0, 0, 0],
                other => {
                    return Err(IconError::UnknownPixel {
                        row: row_index,
                        column: column_index,
                        symbol: other,
                    });
                }
            };
            rgba.extend_from_slice(&pixel);
        }
    }

    Ok(IconImage {
        width: width as u32,
        height: height as u32,
        rgba,
    })
}

/// A header line is a comment only when it is followed by whitespace, so a
/// leading `#` fill pixel is never mistaken for prose.
fn is_comment(line: &str) -> bool {
    let mut characters = line.chars();
    characters.next() == Some(COMMENT_PREFIX) && characters.next().is_some_and(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::{ICON_SIZE, IconError, IconPalette, IconVariant, Rgb, render_glyph};
    use crate::status::StatusState;

    const VARIANTS: [IconVariant; 3] = [
        IconVariant::Idle,
        IconVariant::Working,
        IconVariant::AttentionNeeded,
    ];

    fn test_palette() -> IconPalette {
        IconPalette {
            fill: Rgb::new(1, 2, 3),
            outline: Rgb::new(4, 5, 6),
        }
    }

    // This protects the Win32 icon contract: every glyph must rasterize to
    // the same square, fully populated RGBA buffer.
    #[test]
    fn every_variant_renders_a_square_rgba_image() {
        for variant in VARIANTS {
            let image = variant.render().expect("bundled glyph should rasterize");

            assert_eq!(image.width(), ICON_SIZE);
            assert_eq!(image.height(), ICON_SIZE);
            assert_eq!(image.rgba().len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
        }
    }

    // This is the "not color alone" contract: the shapes themselves differ,
    // so the state stays readable in high-contrast or color-blind settings.
    #[test]
    fn variants_have_distinct_silhouettes() {
        for (index, left) in VARIANTS.iter().enumerate() {
            for right in VARIANTS.iter().skip(index + 1) {
                let left_shape = left.render().unwrap().silhouette();
                let right_shape = right.render().unwrap().silhouette();

                assert_ne!(
                    left_shape, right_shape,
                    "{:?} and {:?} must not share a silhouette",
                    left, right
                );
            }
        }
    }

    // Color is the secondary cue and must still be unambiguous.
    #[test]
    fn variants_have_distinct_colors() {
        for (index, left) in VARIANTS.iter().enumerate() {
            for right in VARIANTS.iter().skip(index + 1) {
                assert_ne!(left.palette().fill, right.palette().fill);
                assert_ne!(left.palette().outline, right.palette().outline);
            }
        }
    }

    // This protects the label pairing that makes each icon self-describing.
    #[test]
    fn variants_have_distinct_labels() {
        assert_eq!(IconVariant::Idle.label(), "Idle");
        assert_eq!(IconVariant::Working.label(), "Working");
        assert_eq!(IconVariant::AttentionNeeded.label(), "Attention needed");
    }

    // This is the monitor-to-tray mapping every state transition depends on.
    #[test]
    fn monitor_state_maps_to_the_matching_variant() {
        assert_eq!(IconVariant::for_state(StatusState::Idle), IconVariant::Idle);
        assert_eq!(
            IconVariant::for_state(StatusState::Generating),
            IconVariant::Working
        );
        assert_eq!(
            IconVariant::for_state(StatusState::Error),
            IconVariant::AttentionNeeded
        );
    }

    // Win32 DIBs are BGRA; a swapped channel order would silently ship the
    // wrong colors, so the conversion is pinned here.
    #[test]
    fn bgra_swaps_red_and_blue_only() {
        let image = render_glyph("#.\n.#", test_palette()).unwrap();
        let bgra = image.bgra();

        assert_eq!(&bgra[0..4], &[3, 2, 1, 0xFF]);
        assert_eq!(&bgra[4..8], &[0, 0, 0, 0]);
    }

    // Glyph assets are data; a malformed asset must be a typed error rather
    // than a panic inside the tray message loop.
    #[test]
    fn ragged_glyph_is_a_typed_error() {
        let error = render_glyph("##\n#", test_palette()).unwrap_err();

        assert_eq!(error, IconError::RaggedGlyph { row: 1, width: 1 });
    }

    // An unsupported symbol must also fail explicitly instead of guessing.
    #[test]
    fn unknown_symbol_is_a_typed_error() {
        let error = render_glyph("#?\n##", test_palette()).unwrap_err();

        assert_eq!(
            error,
            IconError::UnknownPixel {
                row: 0,
                column: 1,
                symbol: '?'
            }
        );
    }

    // An empty or non-square asset would produce an invalid Win32 icon.
    #[test]
    fn empty_and_non_square_glyphs_are_typed_errors() {
        assert_eq!(
            render_glyph("# header only\n", test_palette()).unwrap_err(),
            IconError::EmptyGlyph
        );
        assert_eq!(
            render_glyph("###\n###", test_palette()).unwrap_err(),
            IconError::NotSquare {
                width: 3,
                height: 2
            }
        );
    }
}
