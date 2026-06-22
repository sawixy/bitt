

use eframe::egui;
use std::path::PathBuf;
use rfd::FileDialog;

pub async fn render() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 350.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Bitt",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

#[derive(Default)]
struct MyApp {
    sfiles: Option<PathBuf>,
    progress: f32
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add(
                egui::Image::new(egui::include_image!("../../images/bitt.png"))
                .max_width(180.0),
            );


            ui.heading("Welcome to Bitt!");
            ui.label(format!("Bitt is a minimalistic and fast BitTorrent client written in Rust"));
            ui.separator();
            if ui.button("Choose files").clicked() {
                let files = FileDialog::new()
                .add_filter("torrent", & ["torrent", "txt"])
                .set_directory("/")
                .pick_file();
                self.sfiles = files
            }


            ui.label(format!("{:#?}", self.sfiles));

            if ui.button("Progressing").clicked() {
                self.progress += 0.67;
                if self.progress > 1.0 {
                    self.progress = 0.0;
                }

            }


            let bar = egui::ProgressBar::new(self.progress)
            .show_percentage();
            // .animate(true);
            ui.add(bar);



            //            ui.image(egui::include_image!("bitt.png"))
            //           .on_hover_text_at_pointer("BITT!");

        });
    }
}
