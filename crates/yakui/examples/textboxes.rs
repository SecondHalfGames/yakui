use yakui::widgets::{List, TextBox};
use yakui::{label, use_state};

pub fn run() {
    List::column().item_spacing(8.0).show(|| {
        label("Organized into four rows of two:");
        four_rows();

        label("Organized into two columns of four:");
        two_columns();
    });
}

fn two_columns() {
    List::row().item_spacing(8.0).show(|| {
        for _ in 0..2 {
            List::column().item_spacing(8.0).show(|| {
                for _ in 0..4 {
                    let text = use_state(|| "".to_owned());

                    let response = TextBox::new(text.borrow().as_str())
                        .placeholder("placeholder")
                        .show()
                        .into_inner();

                    if let Some(new_text) = response.text {
                        text.set(new_text);
                    }
                }
            });
        }
    });
}

fn four_rows() {
    List::column().item_spacing(8.0).show(|| {
        for _ in 0..4 {
            List::row().item_spacing(8.0).show(|| {
                for _ in 0..2 {
                    let text = use_state(|| "".to_owned());

                    let response = TextBox::new(text.borrow().as_str())
                        .placeholder("placeholder")
                        .show()
                        .into_inner();

                    if let Some(new_text) = response.text {
                        text.set(new_text);
                    }
                }
            });
        }
    });
}

fn main() {
    bootstrap::start(run as fn());
}
