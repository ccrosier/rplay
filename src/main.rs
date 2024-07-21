use cursive::utils::Counter;
use cursive::views::{ Dialog, SliderView, SelectView, ProgressBar };
use cursive::Cursive;
use cursive::menu;
use cursive::traits::*;
use rodio::{OutputStream, Sink};
use std::fs;

mod player;

fn set_progress(bar: &mut Counter, progress: (u64, u64)) {
    let set_val = progress.0 / progress.1;
    bar.set(set_val as usize);
}

fn get_file_list(dir: fs::ReadDir) -> Vec<String> {
    let files = dir.into_iter();
    let mut list = Vec::new();
    for file in files {
        match file.map(|f| f.path().into_os_string().into_string()) {
            Ok(p) => list.push(p.unwrap()),
            Err(e) => eprintln!("Error {}", e),
        }
    }
    list
}

fn select_song(s: &mut Cursive, name: &str) {
    s.pop_layer();
    s.with_user_data(|p: &mut player::Player| p.add_to_queue(player::SourceFile { file_path: name.to_string() }));
}


fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let player = player::Player {sink: sink};
    player.add_to_queue(player::SourceFile{ file_path: "C:\\Users\\crcro\\OneDrive - The Ohio State University\\Documents\\Music\\The Strokes\\Room On Fire\\Reptilia.wav".to_string()});

    let mut siv = cursive::default();
    siv.set_user_data(player);

    let mut progress_bar = ProgressBar::new().min(0).max(100).with_name("needle position").fixed_width(50);
    let count = Counter::new(0);
    progress_bar.get_inner_mut().get_mut().set_counter(count.clone());
    siv.add_layer(progress_bar);

    // Main player functions
    let player_dialog = Dialog::text("playing music!")
        .button("rw", |s| s.with_user_data(|p: &mut player::Player| p.rewind(10)).unwrap())
        .button("play/pause", |s| s.with_user_data(|p: &mut player::Player| p.toggle_playback()).unwrap())
        .button("ff", |s| s.with_user_data(|p: &mut player::Player| p.fast_forward(10)).unwrap())
        .button("skip", |s| s.with_user_data(|p: &mut player::Player| p.skip()).unwrap())
        .title("Music Player").full_screen().with_name("_player");
    siv.add_layer(player_dialog);
    
    // Playback -> (Volume, Add to queue)
    siv.menubar().add_subtree("Playback", menu::Tree::new().leaf("Volume", |s| {
        let volume_slider = Dialog::around(
        SliderView::horizontal(20)
            .value(10)
            .on_enter(|s, amount| {
                let vol = amount as f32;
                s.with_user_data(|p: &mut player::Player| p.set_volume(vol / 20.0)).unwrap();
                s.pop_layer();
            })
        );
        s.add_layer(volume_slider);
    }).leaf("Add to queue", |s| {
        let flist = get_file_list(fs::read_dir("../The Strokes/Room On Fire/").unwrap());
        let mut select_view = SelectView::<String>::new().on_submit(select_song).fixed_size((50, 10));
        select_view.get_inner_mut().add_all_str(flist.iter());
        s.add_layer(select_view);
    }));
    
    
    let cp: &mut player::Player = siv.user_data().unwrap();
    let prog = cp.progress();
    let mut prog_count = count.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            set_progress(&mut prog_count, prog);
        }
    });

    let _ = siv.call_on_name("_player", |d: &mut Dialog| d.take_focus(cursive::direction::Direction::none()));
    let _ = siv.focus_name("_player");

    siv.add_global_callback('q', |s| s.quit());
    siv.add_global_callback('m', |s| s.select_menubar());
    siv.add_global_callback('r', |s| s.with_user_data(|p: &mut player::Player| p.restart_track()).unwrap());

    // show menu
    siv.set_autohide_menu(false);
    siv.set_autorefresh(true);

    siv.run();
}
