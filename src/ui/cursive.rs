use cursive::event::Key;
use cursive::menu::MenuTree;
use cursive::theme::{BaseColor::*, Color::*, PaletteColor::*};
use cursive::traits::*;
use cursive::view::Margins;
use cursive::views::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::definitions;
use cursive_tabs::TabPanel;

fn parameter_list() -> LinearLayout {
    LinearLayout::horizontal()
        .child(
            LinearLayout::vertical()
                .child(
                    EditView::new()
                        .with_name("parameter/search")
                        .fixed_width(20),
                )
                .child(
                    SelectView::new()
                        .with(|items| {
                            for param in definitions::all() {
                                items.add_item(param.name.clone(), param)
                            }
                        })
                        .on_select(|s, param| {
                            s.call_on_name("parameter/description", |view: &mut TextView| {
                                view.set_content(
                                    format!("{}\n\n{}", param.display_name, param.description)
                                        .clone(),
                                );
                            });
                        })
                        .with_name("parameter/list"),
                ),
        )
        .child(LinearLayout::vertical().child(TextView::new("").with_name("parameter/description")))
}

fn parameter_editor() {}

pub fn event_loop() {
    // Creates the cursive root - required for every application.
    let mut siv = cursive::default();

    let counter = AtomicUsize::new(1);

    let mut group = RadioGroup::new();

    siv.menubar()
        // We add a new "File" tree
        .add_subtree(
            "File",
            MenuTree::new()
                // Trees are made of leaves, with are directly actionable...
                .leaf("New", move |s| {
                    // Here we use the counter to add an entry
                    // in the list of "Recent" items.
                    let i = counter.fetch_add(1, Ordering::Relaxed);
                    let filename = format!("New {}", i);
                    s.menubar()
                        .find_subtree("File")
                        .unwrap()
                        .find_subtree("Recent")
                        .unwrap()
                        .insert_leaf(0, filename, |_| ());

                    s.add_layer(Dialog::info("New file!"));
                })
                // ... and of sub-trees, which open up when selected.
                .subtree(
                    "Recent",
                    // The `.with()` method can help when running loops
                    // within builder patterns.
                    MenuTree::new().with(|tree| {
                        for i in 1..100 {
                            // We don't actually do anything here,
                            // but you could!
                            tree.add_leaf(format!("Item {}", i), |_| ())
                        }
                    }),
                )
                // Delimiter are simple lines between items,
                // and cannot be selected.
                .delimiter()
                .with(|tree| {
                    for i in 1..10 {
                        tree.add_leaf(format!("Option {}", i), |_| ());
                    }
                }),
        )
        .add_subtree(
            "Help",
            MenuTree::new()
                .subtree(
                    "Help",
                    MenuTree::new()
                        .leaf("General", |s| s.add_layer(Dialog::info("Help message!")))
                        .leaf("Online", |s| {
                            let text = "Google it yourself!\n\
                                        Kids, these days...";
                            s.add_layer(Dialog::info(text))
                        }),
                )
                .leaf("About", |s| s.add_layer(Dialog::info("Cursive v0.0.0"))),
        )
        .add_delimiter()
        .add_leaf("Quit", |s| s.quit());

    siv.add_global_callback(Key::Esc, |s| s.select_menubar());
    siv.set_autohide_menu(false);

    //siv.add_layer(Dialog::text("Hit <Esc> to show the menu!"));
    let mut panel = TabPanel::new()
        .with_tab("Parameter Editor", parameter_list())
        .with_tab("First", TextView::new("This is the first view!"))
        .with_tab("Second", TextView::new("This is the second view!"))
        .with_tab(
            "Third",
            LinearLayout::horizontal().child(
                LinearLayout::vertical()
                    // The color group uses the label itself as stored value
                    // By default, the first item is selected.
                    .child(group.button_str("Red"))
                    .child(group.button_str("Green"))
                    .child(group.button_str("Blue")),
            ),
        );
    panel.set_active_tab("Parameter Editor").unwrap();

    siv.add_layer(ResizedView::with_full_screen(panel));

    let mut palette = cursive::theme::Palette::default();
    //palette[Background] = TerminalDefault;
    //palette[View] = TerminalDefault;
    //palette[Primary] = TerminalDefault;

    siv.set_theme(cursive::theme::Theme {
        shadow: false,
        borders: cursive::theme::BorderStyle::Simple,
        palette,
    });
    siv.run();
}
