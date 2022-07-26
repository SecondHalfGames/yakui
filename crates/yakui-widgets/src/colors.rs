//! Defines built-in colors that look pretty good.
#![allow(missing_docs)]

use yakui_core::geometry::Color;

macro_rules! colors {
    (
        $(
            $name:ident = ( $r:literal, $g:literal, $b:literal );
        )*
    ) => {
        $(
            #[doc = concat!("rgb(", $r, ", ", $g, ", ", $b, ")")]
            pub const $name: Color = Color::rgb($r, $g, $b);
        )*
    };
}

colors! {
    BACKGROUND_1 = (31, 31, 31);
    BACKGROUND_2 = (42, 42, 42);
    BACKGROUND_3 = (54, 54, 54);
    TEXT = (255, 255, 255);
    TEXT_MUTED = (147, 147, 147);
}
