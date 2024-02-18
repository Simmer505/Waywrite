use relm4::prelude::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt, GestureSingleExt};
use gtk::cairo::{Context, Operator};
use relm4::drawing::DrawHandler;


#[derive(Debug)]
enum AppInput {
    Increment,
    NewLine((f64, f64)),
    AddPoint((f64, f64)),
    Reset,
}

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
    new_line: bool,
}

#[derive(Debug)]
struct AppModel {
    counter: u8,
    points: Vec<Point>,
    handler: DrawHandler,
}

#[derive(Debug)]
struct UpdatePoints {}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = u8;
    type Input = AppInput;
    type Output = UpdatePoints;

    view! {
        gtk::Window {
            set_title: Some("Simple"),
            set_default_width: 400,
            set_default_height: 100,

            gtk::Box{
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,
                set_margin_all: 2,
                set_hexpand: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 5,
                    set_vexpand: true,

                    gtk::Label {
                        set_label: "Draw something",
                    },

                    #[local_ref]
                    area -> gtk::DrawingArea {
                        set_vexpand: true,
                        set_hexpand: true,

                        add_controller = gtk::GestureStylus {
                            set_button: 0,
                            connect_motion[sender] => move |_, x, y| {
                                sender.input(AppInput::AddPoint((x, y)));
                            },
                            connect_down[sender] => move |_, x, y| {
                                sender.input(AppInput::NewLine((x, y)));
                            }
                        }
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 4,
                    set_margin_all: 2,
                    set_halign: gtk::Align::End,

                    gtk::Button {
                        set_label: "Increment",
                        connect_clicked[sender] => move |_| {
                            sender.input(AppInput::Increment);
                        }
                    },

                    gtk::Button {
                        set_label: "Decrement",
                        connect_clicked[sender] => move |_| {
                            sender.input(AppInput::Reset);
                        }
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &format!("Counter: {}", model.counter),
                        set_margin_all: 5,
                    },
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) ->relm4::ComponentParts<Self> {
        let model = AppModel {
            counter: 0,
            points: Vec::new(),
            handler: DrawHandler::new(),
        };

        let area = model.handler.drawing_area();
        let widgets = view_output!();

        ComponentParts { model, widgets }

    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        let cx = self.handler.get_context();

        match message {
            AppInput::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            AppInput::AddPoint((x, y)) => {
                self.points.push(Point { x, y, new_line: false })
            }
            AppInput::NewLine((x, y)) => {
                self.points.push(Point { x, y, new_line: true })
            }
            AppInput::Reset => {
                cx.set_operator(Operator::Clear);
                cx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                cx.paint().expect("Could not fill context");
                self.points = Vec::new();
            }
        }

        draw(&cx, &self.points);
    }
}

fn draw(cx: &Context, points: &[Point]) {
    for (i, point) in points.into_iter().enumerate().filter(|(i, _)| *i != 0) {
        
        if !point.new_line {
            let last_point = &points[i - 1];

            cx.move_to(last_point.x, last_point.y);
            cx.line_to(point.x, point.y);
            cx.set_source_rgb(200.0, 200.0, 200.0);
            cx.set_line_width(2.0);
            cx.stroke().expect("Failed to draw line");
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.test.simple");
    app.run::<AppModel>(0);
}
