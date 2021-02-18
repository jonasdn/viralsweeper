extern crate gtk;

use gtk::prelude::*;

use std::cell::{RefCell, RefMut};
use std::cmp;
use std::rc::Rc;

use array2d::Array2D;
use gtk::{
    ApplicationWindow, Builder, Button, ButtonsType, DialogFlags, Grid, Label, MessageDialog,
    MessageType, Stack, Window,
};

use rand::Rng;

pub const GRID_SIZE: usize = 16;
pub const VIRUSES: usize = 30;

#[derive(Clone)]
pub enum Cell {
    Hidden(bool),
    Infected,
    Neighbours(usize),
}

pub struct UI {
    window: ApplicationWindow,
}

struct Field {
    cells: Array2D<Cell>,
    grid: Grid,
    clicks: u32,
}

fn get_neighbours(row: i32, col: i32) -> Vec<(usize, usize)> {
    let mut neighbours = Vec::new();
    let limit: i32 = GRID_SIZE as i32 - 1;

    for y in cmp::max(0, row - 1)..=cmp::min(row + 1, limit) {
        for x in cmp::max(0, col - 1)..=cmp::min(col + 1, limit) {
            if y != row || x != col {
                neighbours.push((y as usize, x as usize));
            }
        }
    }
    neighbours
}

//
// Place randomly put not on first click coordinate.
//
fn insert_viruses(field: &mut RefMut<Field>, clicked_row: usize, clicked_col: usize) {
    let mut rng = rand::thread_rng();
    let mut placed = 0;
    let neighbours = get_neighbours(clicked_row as i32, clicked_col as i32);

    while placed < VIRUSES {
        let row = rng.gen_range(0..GRID_SIZE);
        let col = rng.gen_range(0..GRID_SIZE);

        if row == clicked_row && col == clicked_col {
            continue;
        }

        if neighbours.iter().any(|&tuple| tuple == (row, col)) {
            continue;
        }

        match field.cells[(row, col)] {
            Cell::Hidden(false) => {
                field.cells[(row, col)] = Cell::Hidden(true);
                placed += 1;
            }
            _ => continue,
        }
    }
}

//
// The algorithm is ...
//  ... if cell has neighbours with virus then ...
//    ... open it and show the count of viral neighbours.
//  ... else if cell has no viral neighbours ...
//    ... open it and call explode on all neighbours.
//
fn explode(field: &mut RefMut<Field>, row: usize, col: usize) {
    let neighbours = get_neighbours(row as i32, col as i32);
    let infected_neighbours = neighbours
        .iter()
        .filter(|&n| match field.cells[(n.0, n.1)] {
            Cell::Hidden(true) => true,
            _ => false,
        })
        .count();

    field.cells[(row, col)] = Cell::Neighbours(infected_neighbours);
    if infected_neighbours == 0 {
        for tuple in neighbours {
            match field.cells[tuple] {
                Cell::Hidden(false) => explode(field, tuple.0, tuple.1),
                _ => continue,
            }
        }
    }
}
//
// Translate the Array2D<Cell> representation to Gtk UI.
// The GtkStack is either a button, a label or a viral image.
//
fn update_ui(field: &RefMut<Field>) {
    let mut hidden = 0;
    for (row_idx, row) in field.cells.rows_iter().enumerate() {
        for (col_idx, cell) in row.enumerate() {
            let widget = field
                .grid
                .get_child_at(col_idx as i32, row_idx as i32)
                .unwrap();
            let stack = widget.downcast::<Stack>().unwrap();

            match cell {
                Cell::Neighbours(n) => {
                    let widget = stack.get_child_by_name("label").unwrap();
                    let label = widget.downcast::<Label>().unwrap();
                    if n > &0 {
                        label.set_markup(
                            &match n {
                                1 => format!("<span foreground='blue'>{}</span>", n),
                                2 => format!("<span foreground='green'>{}</span>", n),
                                3 => format!("<span foreground='red'>{}</span>", n),
                                4 => format!("<span foreground='purple'>{}</span>", n),
                                5 => format!("<span foreground='maroon'>{}</span>", n),
                                6 => format!("<span foreground='turquoise'>{}</span>", n),
                                7 => format!("<span foreground='black'>{}</span>", n),
                                8 => format!("<span foreground='gray'>{}</span>", n),
                                _ => format!("{}", n),
                            }[..],
                        )
                    }
                    stack.set_visible_child_name("label");
                }
                Cell::Infected => {
                    stack.set_visible_child_name("virus");

                    let dialog = MessageDialog::new(
                        None::<&Window>,
                        DialogFlags::empty(),
                        MessageType::Info,
                        ButtonsType::Ok,
                        "<span foreground='darkgreen'>You have been infected!</span>",
                    );
                    dialog.set_property_use_markup(true);
                    dialog.connect_response(|dialog, _| {
                        dialog.close();
                        std::process::exit(0x0);
                    });
                    dialog.show_all();
                }
                Cell::Hidden(_) => hidden += 1,
            }
        }
    }
    if hidden == VIRUSES {
        let dialog = MessageDialog::new(
            None::<&Window>,
            DialogFlags::empty(),
            MessageType::Info,
            ButtonsType::Ok,
            "<span foreground='darkblue'>You have won!</span>",
        );
        dialog.set_property_use_markup(true);
        dialog.connect_response(|dialog, _| {
            dialog.close();
            std::process::exit(0x0);
        });
        dialog.show_all();
    }
}

//
// We place viruses after first click, since first click should never be on
// a virus. Each click triggers the algorihm in explode which works on
// the Array2D in the fields variable. The UI is update from that model.
//
fn click(field_cell: Rc<RefCell<Field>>, row: usize, col: usize) {
    let mut field = field_cell.borrow_mut();
    field.clicks += 1;

    if field.clicks == 1 {
        insert_viruses(&mut field, row, col);
    }

    match field.cells[(row, col)] {
        Cell::Hidden(true) => field.cells[(row, col)] = Cell::Infected,
        Cell::Hidden(false) => explode(&mut field, row, col),
        _ => (),
    }

    update_ui(&field);
}

impl UI {
    //
    // We build the UI from the xml files in resources/
    // Each cell is a GtkStack.
    //
    pub fn new(application: &gtk::Application) -> Self {
        let resources_bytes = include_bytes!("resources/resources.gresource");
        let resource_data = glib::Bytes::from(&resources_bytes[..]);
        let res = gio::Resource::from_data(&resource_data).unwrap();
        gio::resources_register(&res);

        let builder = Builder::from_resource("/org/viralSweeper/main.ui");

        let window: ApplicationWindow = builder.get_object("mainWindow").unwrap();
        let grid: Grid = builder.get_object("mainGrid").unwrap();

        window.set_application(Some(application));

        let field = Field {
            cells: Array2D::filled_with(Cell::Hidden(false), GRID_SIZE, GRID_SIZE),
            grid,
            clicks: 0,
        };

        let wrapper: Rc<RefCell<Field>> = Rc::new(RefCell::new(field));

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let builder = Builder::from_resource("/org/viralSweeper/cell.ui");
                let stack: Stack = builder.get_object("cellStack").unwrap();
                let button: Button = builder.get_object("cellButton").unwrap();

                let clone = wrapper.clone();
                clone
                    .borrow()
                    .grid
                    .attach(&stack, col as i32, row as i32, 1, 1);

                button.connect_clicked(glib::clone!(@strong wrapper => move |_b| {
                    click(wrapper.clone(), row, col);
                }));
            }
        }
        UI { window }
    }

    pub fn show(&self) {
        self.window.show_all();
    }
}

pub fn run(application: &gtk::Application) {
    UI::new(application).show();
}
