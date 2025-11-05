use yakui::{button, column, row, textbox, use_state};

#[derive(Debug, Clone, Copy)]
enum Page {
    A,
    B,
}

#[track_caller]
fn row_st_d_ve_textbox(initial_text: &'static str) {
    let text = use_state(|| initial_text.to_string());

    row(|| {
        row(|| {
            row(|| {
                let response = textbox(text.borrow().as_str()).into_inner();

                if let Some(new_text) = response.text {
                    text.set(new_text);
                }
            });
        });
    });
}

pub fn run() {
    let page = use_state(|| Page::A);

    column(|| {
        if button("page a").clicked {
            page.set(Page::A);
        }

        if button("page b").clicked {
            page.set(Page::B);
        }

        match page.get() {
            Page::A => {
                row_st_d_ve_textbox("a");
            }
            Page::B => {
                row_st_d_ve_textbox("b");
            }
        };

        textbox("hi");
    });
}

fn main() {
    bootstrap::start(run as fn());
}
