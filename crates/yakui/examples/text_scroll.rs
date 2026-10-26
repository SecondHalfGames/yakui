use yakui::cosmic_text::Scroll;
use yakui::style::TextStyle;
use yakui::widgets::RenderText;
use yakui::{button, column, constrained, row, use_state, Constraints};

pub fn run() {
    let style = use_state(TextStyle::label);
    let scroll = use_state(|| Scroll::new(0, 0.0, 0.0));

    column(|| {
        if button("Increase text size").clicked {
            style.borrow_mut().font_size += 2.0;
        }

        if button("Decrease text size").clicked {
            style.borrow_mut().font_size -= 2.0;
        }

        row(|| {
            if button("Start").clicked {
                style.borrow_mut().align = yakui::style::TextAlignment::Start;
            }
            if button("Center").clicked {
                style.borrow_mut().align = yakui::style::TextAlignment::Center;
            }
            if button("End").clicked {
                style.borrow_mut().align = yakui::style::TextAlignment::End;
            }
        });

        if button("Scroll up").clicked {
            scroll.borrow_mut().vertical -= 12.0;
        }

        if button("Scroll down").clicked {
            scroll.borrow_mut().vertical += 12.0;
        }

        constrained(Constraints::height(0.0, 100.0), || {
            RenderText::new().show_with_scroll(
            "This is a natively scrollable text, which doesn't use yakui's scrolling, but instead uses the RenderText widget's own scrolling!

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum egestas iaculis tortor ut condimentum. Vivamus ultrices elementum pharetra. Donec in elit nisi. Sed dapibus vitae risus in aliquet. In hac habitasse platea dictumst. Integer ultricies eu diam a ullamcorper. Donec viverra felis nec lacinia tempor. Vestibulum non suscipit lectus. Donec et lacus rutrum, maximus nunc eget, tincidunt sapien. Donec ac pulvinar nunc. Vestibulum tincidunt nisi at lacinia pellentesque. Sed suscipit urna vitae nunc efficitur, in eleifend augue cursus. Morbi venenatis sed orci eget vestibulum. Phasellus eleifend libero quam, a lacinia nulla efficitur sit amet. Vestibulum viverra auctor tortor vel suscipit. Nullam in purus in justo consequat vulputate.
Donec tempus vulputate nisl, nec blandit urna aliquam sit amet. Nunc non mi leo. Pellentesque dapibus, lacus ac aliquet venenatis, nunc orci tempor elit, quis faucibus turpis metus at nunc. Proin rhoncus, metus nec tristique feugiat, elit mi aliquam nisl, a elementum odio mi cursus mauris. Ut sed convallis nisl. Vivamus aliquam lacinia tincidunt. Nulla sed consectetur magna. Sed a lectus nec erat consectetur vulputate. Mauris ut tempus odio, ac aliquet dui. Morbi vehicula arcu quis tortor dictum blandit. Nullam fermentum dolor ut ex eleifend aliquam. Nunc ut dictum ante. Morbi ultricies sem sed feugiat mattis. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Sed tristique eleifend mi, non iaculis sem commodo ut. In id tellus eu massa accumsan malesuada.", 
            &style.borrow(),
            Some(scroll.get())
        );
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
