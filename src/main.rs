use cursive::views::{ Dialog, SliderView, SelectView, TextView };
use cursive::Cursive;
use cursive::menu;
use cursive::event::Key;
use cursive::traits::*;
use rodio::{OutputStream, Sink};
use std::fs;

mod player;

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
    if name == "<cancel selection>" {
        s.pop_layer();
    } else if std::path::Path::new(name).is_dir() || name == ".." {
        s.call_on_name("_select", |view: &mut SelectView| {
            view.clear();
            let mut new_files = get_file_list(fs::read_dir(name).unwrap());
            new_files.insert(0, "..".to_string());
            new_files.insert(0, "<cancel selection>".to_string());
            view.add_all_str(new_files);
        });
    } else {
        s.pop_layer();
        s.with_user_data(|p: &mut player::Player| p.add_to_queue(player::SourceFile { file_path: name.to_string() }));
        s.call_on_name("_track_info", |view: &mut TextView| {
            view.set_content(format!("Current track: {}", name.split(std::path::MAIN_SEPARATOR_STR).last().unwrap()));
        });
    }
}


fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let player = player::Player {sink: sink};
    let mut siv = cursive::default();
    siv.set_user_data(player);
    
    let track_view = TextView::new("Current track: None").with_name("_track_info");

    // Main player functions
    let mut player_dialog = Dialog::text("playing music!")
        .button("rw", |s| s.with_user_data(|p: &mut player::Player| p.rewind(10)).unwrap())
        .button("play/pause", |s| s.with_user_data(|p: &mut player::Player| p.toggle_playback()).unwrap())
        .button("ff", |s| s.with_user_data(|p: &mut player::Player| p.fast_forward(10)).unwrap())
        .button("skip", |s| {
            s.with_user_data(|p: &mut player::Player| p.skip()).unwrap();
            s.call_on_name("_track_info", |view: &mut TextView| {
                view.set_content("Current track: None")
            });
        })
        .title("Music Player").full_screen();
    player_dialog.get_inner_mut().set_content(track_view);
    siv.add_layer(player_dialog);
    
    // Playback -> (Volume, Add to queue)
    let mut flist = get_file_list(fs::read_dir("./").unwrap());
    flist.insert(0, "..".to_string());
    flist.insert(0, "<cancel selection>".to_string());
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
    }).leaf("Add to queue", move |s| {
        let mut select_view = SelectView::<String>::new().on_submit(select_song).with_name("_select");
        select_view.get_mut().add_all_str(flist.clone());
        s.add_layer(select_view.scrollable().fixed_size((40, 5)));
    }).leaf("Exit", |s| {
        s.quit();
    }));
    
    let _ = siv.call_on_name("_player", |d: &mut Dialog| d.take_focus(cursive::direction::Direction::none()));
    let _ = siv.focus_name("_player");

    siv.add_global_callback('q', |s| s.quit());
    siv.add_global_callback('m', |s| s.select_menubar());
    siv.add_global_callback(Key::Esc, |s| s.select_menubar());
    siv.add_global_callback('r', |s| s.with_user_data(|p: &mut player::Player| p.restart_track()).unwrap());

    // show menu
    siv.set_autohide_menu(false);
    siv.set_autorefresh(true);

    siv.run();
}
