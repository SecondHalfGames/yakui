use std::{cell::Cell, sync::Arc};

use bootstrap::OPENMOJI;
use yakui::{column, font::Fonts, text, util::widget, widget::Widget, Vec2};
use yakui_widgets::cosmic_text::fontdb;

#[derive(Debug)]
struct LoadFontsWidget {
    loaded: Cell<bool>,
}

impl Widget for LoadFontsWidget {
    type Props<'a> = ();

    type Response = ();

    fn new() -> Self {
        Self {
            loaded: Cell::default(),
        }
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {}

    fn layout(
        &self,
        ctx: yakui::widget::LayoutContext<'_>,
        _constraints: yakui::Constraints,
    ) -> yakui::Vec2 {
        if !self.loaded.get() {
            let fonts = ctx.dom.get_global_or_init(Fonts::default);

            fonts.load_font_source(fontdb::Source::Binary(Arc::from(&OPENMOJI)));

            self.loaded.set(true);
        }

        Vec2::ZERO
    }
}

pub fn run() {
    widget::<LoadFontsWidget>(());

    column(|| {
        text(16.0, "I like to render اللغة العربية in Rust!

عندما يريد العالم أن \u{202a}يتكلّم \u{202c} ، فهو يتحدّث بلغة يونيكود. تسجّل الآن لحضور المؤتمر الدولي العاشر ليونيكود (Unicode Conference)، الذي سيعقد في 10-12 آذار 1997 بمدينة مَايِنْتْس، ألمانيا. و سيجمع المؤتمر بين خبراء من كافة قطاعات الصناعة على الشبكة العالمية انترنيت ويونيكود، حيث ستتم، على الصعيدين الدولي والمحلي على حد سواء مناقشة سبل استخدام يونكود في النظم القائمة وفيما يخص التطبيقات الحاسوبية، الخطوط، تصميم النصوص والحوسبة متعددة اللغات.");

        text(16.0, "I want more terminals to be able to handle ZWJ sequence emoji characters. For example, the service dog emoji 🐕‍🦺 is actually 3 Unicode characters. Kitty handles this fairly well. All VTE-based terminals, however, show \"🐶🦺\".");

        text(
            16.0,
            "
    《施氏食狮史》
石室诗士施氏，嗜狮，誓食十狮。
氏时时适市视狮。
十时，适十狮适市。
是时，适施氏适市。
氏视是十狮，恃矢势，使是十狮逝世。
氏拾是十狮尸，适石室。
石室湿，氏使侍拭石室。
石室拭，氏始试食是十狮。
食时，始识是十狮尸，实十石狮尸。
试释是事。
",
        );
    });
}

fn main() {
    bootstrap::start(run as fn());
}
