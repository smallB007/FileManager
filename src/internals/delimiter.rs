use cursive::align::HAlign;
use cursive::theme::ColorStyle;
use cursive::view::View;
use cursive::{Printer, Vec2};
use unicode_width::UnicodeWidthStr;
pub struct Delimiter {
    title: String,
    title_position: HAlign,
}
impl Delimiter {
    pub fn new(title: &str) -> Self {
        Delimiter {
            title: String::from(title),
            title_position: HAlign::Right,
        }
    }
    fn draw_title(&self, printer: &Printer) {
        if !self.title.is_empty() {
            let len = self.title.width();
            let spacing = 3; //minimum distance to borders
            let spacing_both_ends = 2 * spacing;
            if len + spacing_both_ends > printer.size.x {
                return;
            }
            let x = spacing + self.title_position.get_offset(len, printer.size.x - spacing_both_ends);
            printer.with_low_border(false, |printer| {
                printer.print((x - 2, 0), "┤ ");
                printer.print((x + len, 0), " ├");
            });

            printer.with_color(ColorStyle::title_primary(), |p| p.print((x, 0), &self.title));
        }
    }
}
impl View for Delimiter {
    /// This is the only *required* method to implement.
    fn draw(&self, printer: &Printer) {
        printer.print_hdelim(Vec2::new(0, 0), printer.size.pair().0);

        self.draw_title(printer);
    }
}
