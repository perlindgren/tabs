use tabs::{
    drums::DrumMapping,
    drums_gui::{DrumChart, DrumView},
};
fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([540.0, 1080.0]),
        ..Default::default()
    };
    eframe::run_native(
        "TABS Drums",
        options,
        Box::new(|cc| {
            Ok(Box::new(DrumView::new(
                cc,
                DrumChart::default(),
                DrumMapping::new(),
            )))
        }),
    )
    .ok();
}
