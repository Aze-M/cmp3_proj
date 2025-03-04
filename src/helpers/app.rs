use anyhow::Result;
use rand::seq::SliceRandom;
use slint::Model;
use std::env;
use std::fs::File;
use std::path::Path;

use crate::*;
use crate::helpers::audio::AudioEngine;

#[allow(unused, unused_variables)]
pub struct App {
}

#[allow(unused, unused_variables)]
impl App {
    pub fn run() -> Result<()> {
        let main_window = MainWindow::new()?;
        let ae = AudioEngine::get_instance();

        //preload sound file.
        let mut sound_path = env::current_dir()?.to_string_lossy().to_string()
        + "\\ui\\sfx\\click.mp3";
        println!("{:?}", sound_path);
        let sound_file = File::open(sound_path)?;

        println!("{:?}", sound_file);

        //predecode sound file and keep loaded.
        let mut ae_lock = ae.lock().unwrap();
        let mut sound = ae_lock.decode(sound_file)?;
        drop(ae_lock);

        //get memory tiles defined in the slint file
        let mut tiles: Vec<TileData> = main_window.get_memory_tiles().iter().collect();
        //duplicate
        tiles.extend(tiles.clone());

        //shuffle
        let mut rng = rand::thread_rng();
        tiles.shuffle(&mut rng);

        let tiles_model = std::rc::Rc::new(slint::VecModel::from(tiles));
        main_window.set_memory_tiles(tiles_model.clone().into());

        //pair checks
        let main_window_weakptr = main_window.as_weak();
        main_window.on_check_if_pair_solved(move || {
            let mut flipped_tiles = tiles_model
                .iter()
                .enumerate()
                .filter(|(_, tile)| tile.image_visible && !tile.solved);

            if let (Some((t1_idx, mut t1)), Some((t2_idx, mut t2))) =
                (flipped_tiles.next(), flipped_tiles.next())
            {
                let solved = t1 == t2;

                if solved {
                    t1.solved = true;
                    t2.solved = true;
                    tiles_model.set_row_data(t1_idx, t1);
                    tiles_model.set_row_data(t2_idx, t2);
                } else {
                    let main_window = main_window_weakptr.unwrap();
                    main_window.set_disable_tiles(true);
                    let tiles_model = tiles_model.clone();
                    slint::Timer::single_shot(std::time::Duration::from_secs(1), move || {
                        main_window.set_disable_tiles(false);
                        t1.image_visible = false;
                        t2.image_visible = false;
                        tiles_model.set_row_data(t1_idx, t1);
                        tiles_model.set_row_data(t2_idx, t2);
                    });
                }
            }
        });

        // this pointer was claimed earlier so we need another definition here
        let main_window_weakptr = main_window.as_weak();
        main_window.on_increment_counter(move || {
            let main_window = main_window_weakptr.unwrap();

            // get clicks
            let clicks = main_window.get_clicks();

            // update move counter to clicks / 2 (as we update moves every time a pair has been clicked)
            let counter = clicks / 2;

            main_window.set_counter(counter);
        });

        //play sound on clicked tile;
        main_window.on_play_sound(move || {
            let ae = AudioEngine::get_instance();
            let ae_lock = ae.lock().unwrap();

            let mut sound_path = env::current_dir().unwrap().to_string_lossy().to_string()
            + "\\ui\\sfx\\click.mp3";

            ae_lock.decode_and_play( Path::new(&sound_path) );
        });

        // always last
        main_window.run()?;

        return Ok(());
    }
}
