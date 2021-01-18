mod bevy_impls;
mod vec;

#[allow(unreachable_pub)] // it _is_ imported, but rustc does not seem to realize that
pub use vec::Vec2dAttributes;

use crate::{Inspectable, Options};
use bevy::render::color::Color;
use bevy_egui::egui;
use egui::widgets;

#[derive(Debug, Clone)]
pub struct NumberAttributes<T> {
    pub min: T,
    pub max: T,
    pub step: T,
    pub prefix: String,
    pub suffix: String,
}
impl<T: Default> Default for NumberAttributes<T> {
    fn default() -> Self {
        NumberAttributes {
            min: T::default(),
            max: T::default(),
            step: T::default(),
            prefix: "".into(),
            suffix: "".into(),
        }
    }
}
impl<T> NumberAttributes<T> {
    fn map<U>(&self, f: impl Fn(&T) -> U) -> NumberAttributes<U> {
        NumberAttributes {
            min: f(&self.min),
            max: f(&self.max),
            step: f(&self.step),
            prefix: self.prefix.clone(),
            suffix: self.suffix.clone(),
        }
    }
}

macro_rules! impl_for_num {
    ($ty:ident) => {
        impl Inspectable for $ty {
            type FieldOptions = NumberAttributes<f32>;

            fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
                let mut widget = widgets::DragValue::$ty(self);

                if !options.custom.prefix.is_empty() {
                    widget = widget.prefix(options.custom.prefix);
                }
                if !options.custom.suffix.is_empty() {
                    widget = widget.suffix(options.custom.suffix);
                }

                if options.custom.min != options.custom.max {
                    widget = widget.range(options.custom.min..=options.custom.max);
                }

                if options.custom.step != 0. {
                    widget = widget.speed(options.custom.step);
                }

                ui.add(widget);
            }
        }
    };

    ($($ty:ident),*) => {
        $(impl_for_num!($ty);)*
    }
}

impl_for_num!(f32, f64, u8, i32);

#[derive(Clone, Debug, Default)]
pub struct StringAttributes {
    pub multiline: bool,
}

impl Inspectable for String {
    type FieldOptions = StringAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
        let widget = match options.custom.multiline {
            false => widgets::TextEdit::singleline(self),
            true => widgets::TextEdit::multiline(self),
        };

        ui.add(widget);
    }
}

impl Inspectable for bool {
    type FieldOptions = ();
    fn ui(&mut self, ui: &mut egui::Ui, _: Options<Self::FieldOptions>) {
        ui.checkbox(self, "");
    }
}

#[derive(Default, Debug, Clone)]
pub struct ColorAttributes {
    pub alpha: bool,
}

impl Inspectable for Color {
    type FieldOptions = ColorAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
        let old: [f32; 4] = (*self).into();

        if options.custom.alpha {
            let mut color = egui::color::Color32::from_rgba_premultiplied(
                (old[0] * u8::MAX as f32) as u8,
                (old[1] * u8::MAX as f32) as u8,
                (old[2] * u8::MAX as f32) as u8,
                (old[3] * u8::MAX as f32) as u8,
            );
            ui.color_edit_button_srgba(&mut color);
            let [r, g, b, a] = color.to_array();
            *self = Color::rgba_u8(r, g, b, a);
        } else {
            let mut color = [old[0], old[1], old[2]];
            ui.color_edit_button_rgb(&mut color);
            let [r, g, b] = color;
            *self = Color::rgb(r, g, b);
        }
    }
}

impl<T> Inspectable for Vec<T>
where
    T: Inspectable + Default,
    T::FieldOptions: Clone,
{
    type FieldOptions = <T as Inspectable>::FieldOptions;

    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
        ui.vertical(|ui| {
            let mut to_delete = None;

            for (i, val) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(i.to_string());
                    val.ui(ui, options.clone());
                    if ui.button("-").clicked {
                        to_delete = Some(i);
                    }
                });
            }

            ui.vertical_centered_justified(|ui| {
                if ui.button("+").clicked {
                    self.push(T::default());
                }
            });

            if let Some(i) = to_delete {
                self.remove(i);
            }
        });
    }
}

#[cfg(feature = "nightly")]
impl<T: Inspectable, const N: usize> Inspectable for [T; N]
where
    T::FieldOptions: Clone,
{
    type FieldOptions = <T as Inspectable>::FieldOptions;

    fn ui(&mut self, ui: &mut egui::Ui, options: Options<Self::FieldOptions>) {
        ui.vertical(|ui| {
            for (i, val) in self.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(i.to_string());
                    val.ui(ui, options.clone());
                });
            }
        });
    }
}