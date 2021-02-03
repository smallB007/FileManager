use cursive::align::HAlign;
use cursive::direction::Direction;
use cursive::event::*;
//use cursive::rect::Rect;
use cursive::theme::ColorStyle;
use cursive::view::View;
use cursive::Vec2;
use cursive::{Cursive, Printer};
use unicode_width::UnicodeWidthStr;

/// Simple text label with a callback when <Enter> is pressed.
///
/// A button shows its content in a single line and has a fixed size.
///
/// # Examples
///
/// ```
/// use cursive_core::views::Atomic_Button;
///
/// let quit_button = Atomic_Button::new("Quit", |s| s.quit());
/// ```
pub struct Atomic_Button {
    label: String,
    callback: Callback,
    enabled: bool,
    last_size: Vec2,

    invalidated: bool,
}

impl Atomic_Button {
    //   impl_enabled!(self.enabled);

    /// Creates a new button with the given content and callback.
    pub fn new<F, S>(label: S, cb: F) -> Self
    where
        F: 'static + Fn(&mut Cursive),
        S: Into<String>,
    {
        let label = label.into();
        Self::new_raw(format!("<{}>", label), cb)
    }

    /// Creates a new button without angle brackets.
    ///
    /// # Examples
    ///
    /// ```
    /// use cursive_core::views::Atomic_Button;
    ///
    /// let button = Atomic_Button::new_raw("[ Quit ]", |s| s.quit());
    /// ```
    pub fn new_raw<F, S: Into<String>>(label: S, cb: F) -> Self
    where
        F: 'static + Fn(&mut Cursive),
    {
        Atomic_Button {
            label: label.into(),
            callback: Callback::from_fn(cb),
            enabled: true,
            last_size: Vec2::zero(),
            invalidated: true,
        }
    }

    /// Sets the function to be called when the button is pressed.
    ///
    /// Replaces the previous callback.
    pub fn set_callback<F>(&mut self, cb: F)
    where
        F: Fn(&mut Cursive) + 'static,
    {
        self.callback = Callback::from_fn(cb);
    }

    /// Returns the label for this button.
    ///
    /// Includes brackets.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cursive_core::views::Atomic_Button;
    /// let button = Atomic_Button::new("Quit", |s| s.quit());
    /// assert_eq!(button.label(), "<Quit>");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the label to the given value.
    ///
    /// This will include brackets.
    ///
    /// # Examples
    ///
    /// ```
    /// use cursive_core::views::Atomic_Button;
    ///
    /// let mut button = Atomic_Button::new("Quit", |s| s.quit());
    /// button.set_label("Escape");
    /// ```
    pub fn set_label<S>(&mut self, label: S)
    where
        S: Into<String>,
    {
        self.set_label_raw(format!("<{}>", label.into()));
    }

    /// Sets the label exactly to the given value.
    ///
    /// This will not include brackets.
    pub fn set_label_raw<S>(&mut self, label: S)
    where
        S: Into<String>,
    {
        self.label = label.into();
        self.invalidate();
    }

    fn req_size(&self) -> Vec2 {
        Vec2::new(self.label.width(), 1)
    }

    fn invalidate(&mut self) {
        self.invalidated = true;
    }
}

impl View for Atomic_Button {
    fn draw(&self, printer: &Printer) {
        if printer.size.x == 0 {
            return;
        }

        let style = if !(self.enabled && printer.enabled) {
            ColorStyle::secondary()
        } else if printer.focused {
            ColorStyle::highlight()
        } else {
            ColorStyle::primary()
        };

        let offset = HAlign::Center.get_offset(self.label.width(), printer.size.x);

        printer.with_color(style, |printer| {
            printer.print((offset, 0), &self.label);
        });
    }

    fn layout(&mut self, size: Vec2) {
        self.last_size = size;
        self.invalidated = false;
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        // Meh. Fixed size we are.
        self.req_size()
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        if !self.enabled || event == Event::Key(Key::Tab) {
            return EventResult::Ignored;
        }

        // eprintln!("{:?}", event);
        // eprintln!("{:?}", self.req_size());
        let width = self.label.width();
        let self_offset = HAlign::Center.get_offset(width, self.last_size.x);
        match event {
            Event::Key(Key::Tab) => EventResult::Ignored,
            Event::Key(Key::Enter) => EventResult::Consumed(Some(self.callback.clone())),
            Event::Mouse {
                event: MouseEvent::Release(MouseButton::Left),
                position,
                offset,
            } if position.fits_in_rect(offset + (self_offset, 0), self.req_size()) => EventResult::Consumed(Some(self.callback.clone())),
            _ => EventResult::Ignored,
        }
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        self.enabled
        //        false
    }

    fn important_area(&self, view_size: Vec2) -> cursive::Rect {
        let width = self.label.width();
        let offset = HAlign::Center.get_offset(width, view_size.x);

        cursive::Rect::from_size((offset, 0), (width, 1))
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated
    }
}
