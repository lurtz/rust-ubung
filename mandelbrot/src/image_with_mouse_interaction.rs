use conrod::{self, widget, Colorable, Dimensions, Labelable, Point, Positionable, Widget};

pub struct ImageWithMouseInteraction {
//    image : conrod::widget::Image,
    common: widget::CommonBuilder,
    color : Option<conrod::Color>,
    style: Style,
}

widget_style!{
    /// Represents the unique styling for our CircularButton widget.
    style Style {
        /// Color of the button.
        - color: conrod::Color { theme.shape_color }
        /// Color of the button's label.
        - label_color: conrod::Color { theme.label_color }
        /// Font size of the button's label.
        - label_font_size: conrod::FontSize { theme.font_size_medium }
    }
}

widget_ids! {
    struct Ids {
        image,
    }
}

pub struct State {
    ids: Ids,
}

impl ImageWithMouseInteraction {
    pub fn new() -> ImageWithMouseInteraction {
        ImageWithMouseInteraction {
            common: widget::CommonBuilder::new(),
            color: Option::None,
            style: Style::new(),
        }
    }
}

impl ImageWithMouseInteraction {
    builder_method!(pub color { color = Some(conrod::Color) });
}

impl Widget for ImageWithMouseInteraction {
    type State = State;
    type Style = Style;
    type Event = Option<()>;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State { ids: Ids::new(id_gen) }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        None
    }


}
