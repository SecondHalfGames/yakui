use bootstrap::load_common_fonts;
use yakui::style::TextAlignment;
use yakui::widgets::Text;
use yakui::{column, row, scroll_vertical, spacer, text};

const TEXT_WITH_EMOJI: &str ="I want more terminals to be able to handle ZWJ sequence emoji characters. For example, the service dog emoji 🐕‍🦺 is actually 3 Unicode characters. Kitty handles this fairly well. All VTE-based terminals, however, show \"🐶🦺\".
";

const ARABIC: &str = "I like to render اللغة العربية in Rust!

عندما يريد العالم أن \u{202a}يتكلّم \u{202c} ، فهو يتحدّث بلغة يونيكود. تسجّل الآن لحضور المؤتمر الدولي العاشر ليونيكود (Unicode Conference)، الذي سيعقد في 10-12 آذار 1997 بمدينة مَايِنْتْس، ألمانيا. و سيجمع المؤتمر بين خبراء من كافة قطاعات الصناعة على الشبكة العالمية انترنيت ويونيكود، حيث ستتم، على الصعيدين الدولي والمحلي على حد سواء مناقشة سبل استخدام يونكود في النظم القائمة وفيما يخص التطبيقات الحاسوبية، الخطوط، تصميم النصوص والحوسبة متعددة اللغات.
";

const STONE_LION_RIDDLE: &str = "《施氏食狮史》
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
";

pub fn run() {
    load_common_fonts();

    scroll_vertical(|| {
        column(|| {
            text(20.0, "You should be able to scroll down!\n");

            text(20.0, "Here's some RTL text:\n");
            text(16.0, ARABIC);

            text(
                20.0,
                "These are put in a row, and they should be centered:\n",
            );
            row(|| {
                Text::new(16.0, STONE_LION_RIDDLE)
                    .style(|style| style.align(TextAlignment::Center))
                    .show();
                spacer(1);
                Text::new(16.0, STONE_LION_RIDDLE)
                    .style(|style| style.align(TextAlignment::Center))
                    .show();
            });

            text(20.0, "This one should be centered:\n");
            Text::new(16.0, TEXT_WITH_EMOJI)
                .style(|style| style.align(TextAlignment::Center))
                .show();
            text(20.0, "This one should be on the right:\n");
            Text::new(16.0, TEXT_WITH_EMOJI)
                .style(|style| style.align(TextAlignment::End))
                .show();

            text(
                20.0,
                "This one should be centered but only within the text itself:\n",
            );
            Text::new(16.0, TEXT_WITH_EMOJI)
                .inline(true)
                .style(|style| style.align(TextAlignment::Center))
                .show();
            text(
                20.0,
                "This one should be on the right but only within the text itself:\n",
            );
            Text::new(16.0, TEXT_WITH_EMOJI)
                .inline(true)
                .style(|style| style.align(TextAlignment::End))
                .show();

            text(
                20.0,
                "Arabic is written RTL, this one should be 'on the left':\n",
            );
            Text::new(16.0, ARABIC)
                .style(|style| style.align(TextAlignment::End))
                .show();

            text(
                20.0,
                "Same as the above, but only within the text itself:\n",
            );
            Text::new(16.0, ARABIC)
                .inline(true)
                .style(|style| style.align(TextAlignment::End))
                .show();
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
