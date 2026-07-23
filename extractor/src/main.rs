mod kvparser;

use crate::kvparser::{KVParser, KVParserBuilder};
use clap::Parser;
use dotenv::dotenv;
use eyre::eyre;
use log::{LevelFilter, debug, info, trace, warn};
use regex::{Regex, regex};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{File, create_dir_all, read_to_string};
use std::io::Write;
use std::num::NonZero;
use std::path::{Path, PathBuf, absolute};
use std::process::{Command, Stdio};
use tempfile::{TempDir, tempdir};

const CHAPTER_COUNT: u32 = 5;

#[derive(Parser)]
struct Cli {
    /// Path to the UTMT CLI executable.
    ///
    /// Download from https://github.com/UnderminersTeam/UndertaleModTool/releases/latest.
    #[arg(long, env)]
    utmt_cli_path: PathBuf,
    /// Path to the DELTARUNE game directory.
    #[arg(long, env)]
    deltarune_path: PathBuf,
    /// Where UTMT should output extracted data.
    ///
    /// By default, uses a temporary directory removed afterwards. When overriding,
    /// the directory is *not* removed.
    #[arg(short, long, env = "DELTAKIT_EXTRACTOR_UTMT_OUTPUT_PATH")]
    utmt_output_path: Option<PathBuf>,
}

fn get_data_win_path(options: &Cli, chapter: u32) -> PathBuf {
    absolute(&options.deltarune_path)
        .unwrap()
        .join(format!("chapter{}_windows", chapter))
        .join("data.win")
}

fn get_scripts<const N: usize>(
    options: &Cli,
    chapter: u32,
    script_names: &[&str; N],
    tempdir: &Path,
) -> eyre::Result<[String; N]> {
    let mut cmd = Command::new(&options.utmt_cli_path);
    cmd.arg("dump")
        .arg(get_data_win_path(options, chapter))
        .arg("-o")
        .arg(tempdir)
        .stdout(Stdio::null());

    for script in script_names {
        cmd.arg("-c");
        cmd.arg(script);
    }

    debug!("Calling UTMT: {:?}", cmd);

    let status = cmd.status()?;

    if !status.success() {
        return Err(eyre!(
            "UTMT dump exited with non-successful status code: {status}."
        ));
    }

    let mut output: [String; N] = [const { String::new() }; N];

    for (i, script) in script_names.iter().enumerate() {
        let script_file_path = tempdir.join("CodeEntries").join(format!("{script}.gml"));

        if !script_file_path.exists() {
            Err(eyre!("UTMT did not dump out the script file for {script}."))?;
        }

        output[i] = read_to_string(script_file_path)?;
    }

    Ok(output)
}

fn get_script(
    options: &Cli,
    chapter: u32,
    script_name: &str,
    tempdir: &Path,
) -> eyre::Result<String> {
    get_scripts(options, chapter, &[script_name], tempdir).map(|[c]| c)
}

/// Gets the asset names in order from the specific chapter.
///
/// Uses `ExportAssetOrder.csx` script that should be included with any UTMT installation.
///
/// Returns a [`HashMap`] mapping asset types (`sounds`, `sprites`, `backgrounds`, `paths`, `scripts`,
/// `fonts`, `objects`, `timelines`, `rooms`, `shaders`, `extensions`) to a [`Vec`]
/// of asset names.
fn get_asset_order_txt(
    options: &Cli,
    chapter: u32,
    tempdir: &Path,
) -> eyre::Result<HashMap<String, Vec<String>>> {
    let script_path = options
        .utmt_cli_path
        .join("../Scripts/Technical Scripts/ExportAssetOrder.csx");
    let out_txt_path = tempdir.join("asset_order.txt");

    let mut cmd = Command::new(&options.utmt_cli_path);
    cmd.arg("load")
        .arg(get_data_win_path(options, chapter))
        .arg("-s")
        .arg(script_path)
        .stdout(Stdio::null())
        .stdin(Stdio::piped());

    debug!("Calling UTMT: {:?}", cmd);

    let mut child = cmd.spawn()?;

    {
        let mut stdin = child.stdin.take().unwrap();
        writeln!(stdin, "{}\r", out_txt_path.display())?;
    }

    let status = child.wait()?;

    if !status.success() {
        return Err(eyre!(
            "UTMT load exited with non-successful status code: {status}."
        ));
    }

    let output = read_to_string(out_txt_path)?;

    let mut hashmap: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_asset_type: Option<String> = None;
    let mut current_asset_list: Vec<String> = Vec::new();

    for line in output.lines() {
        if line.starts_with("@@") && line.ends_with("@@") {
            if let Some(prev_asset_type) = current_asset_type.take()
                && !current_asset_list.is_empty()
            {
                let prev_asset_list = current_asset_list;
                current_asset_list = Vec::new();
                hashmap.insert(prev_asset_type, prev_asset_list);
            }

            let asset_type = &line[2..line.len() - 2];
            current_asset_type = Some(asset_type.to_string());
        } else {
            current_asset_list.push(line.to_string());
        }
    }

    if let Some(last_asset_type) = current_asset_type.take()
        && !current_asset_list.is_empty()
    {
        let last_asset_list = current_asset_list;
        hashmap.insert(last_asset_type, last_asset_list);
    }

    Ok(hashmap)
}

type AssetName = String;
type RoomId = NonZero<u32>;

#[derive(Debug, Serialize)]
struct ChapterData {
    armors: Vec<Option<String>>,
    items: Vec<Option<String>>,
    key_items: Vec<Option<String>>,
    light_world_items: Vec<Option<String>>,
    phone_numbers: Vec<Option<String>>,
    room_assets: Vec<AssetName>,
    room_ids: Vec<Option<RoomId>>,
    room_names: Vec<Option<String>>,
    spells: Vec<Option<String>>,
    weapons: Vec<Option<String>>,
}

impl ChapterData {
    fn load(options: &Cli, chapter: u32, tempdir: &Path) -> eyre::Result<ChapterData> {
        let [
            items_code,
            key_items_code,
            phone_numbers_code,
            light_world_items_code,
            armors_code,
            spells_code,
            weapons_code,
            room_names_code,
            room_ids_code,
        ] = get_scripts(
            options,
            chapter,
            &[
                "gml_GlobalScript_scr_iteminfo",
                "gml_GlobalScript_scr_keyiteminfo",
                "gml_GlobalScript_scr_phonename",
                "gml_GlobalScript_scr_litemname",
                "gml_GlobalScript_scr_armorinfo",
                "gml_GlobalScript_scr_spellinfo",
                "gml_GlobalScript_scr_weaponinfo",
                "gml_GlobalScript_scr_roomname",
                "gml_GlobalScript_scr_get_room_by_id",
            ],
            tempdir,
        )?;

        let room_assets = {
            let mut asset_order = get_asset_order_txt(options, chapter, tempdir)?;
            asset_order
                .remove("rooms")
                .ok_or_else(|| eyre!("couldn't get room asset order"))?
        };

        let mut room_ids = vec![None; room_assets.len()];

        let rooms_assets_ids = Self::find_room_assets_and_ids(&room_ids_code);
        for (asset_name, id) in rooms_assets_ids {
            let index = room_assets
                .iter()
                .position(|it| it == &asset_name)
                .ok_or_else(|| eyre!("couldn't find index of room {asset_name}"))?;

            room_ids[index] = Some(id);
        }

        // debug!("Rooms: {:?}", room_ids.iter().zip(&room_assets).collect::<Vec<_>>());

        let armors = KVParser::builder()
            .with_scope_patterns((regex!(r"function scr_armorinfo\(arg0\)"), regex!("$")))
            .with_iter_patterns((regex!(r"case (?:\d+):"), regex!("break;")))
            .with_key_pattern(regex!(r"^case (\d+):"))
            .with_value_pattern(regex!(r"armornametemp ?= ?(.*);\n"))
            .parse(&armors_code)
            .unwrap();
        let armors = Self::deindex(armors);

        let items = KVParser::builder()
            .with_scope_patterns((regex!(r"function scr_iteminfo\(arg0\)"), regex!("$")))
            .with_iter_patterns((regex!(r"case (?:\d+):"), regex!("break;")))
            .with_key_pattern(regex!(r"^case (\d+):"))
            .with_value_pattern(regex!(r"itemnameb ?= ?(.*);\n"))
            .parse(&items_code)
            .unwrap();
        let items = Self::deindex(items);

        let key_items = KVParser::builder()
            .with_scope_patterns((regex!(r"function scr_keyiteminfo\(arg0\)"), regex!("$")))
            .with_iter_patterns((regex!(r"case (?:\d+):"), regex!("break;")))
            .with_key_pattern(regex!(r"^case (\d+):"))
            .with_value_pattern(regex!(r"tempkeyitemname ?= ?(.*);\n"))
            .parse(&key_items_code)
            .unwrap();
        let key_items = Self::deindex(key_items);

        let light_world_items = KVParser::builder()
            .with_scope_patterns((regex!(r"function scr_litemname\(\)"), regex!("$")))
            .with_iter_patterns((regex!(r"if ?\(itemid ?== ?(?:\d+)\)"), regex!(r"\n {8}}")))
            .with_key_pattern(regex!(r"^if ?\(itemid ?== ?(\d+)\)"))
            .with_value_pattern(regex!(r"global\.litemname\[i] ?= ?(.*);\n"))
            .parse(&light_world_items_code)
            .unwrap();
        let light_world_items = Self::deindex(light_world_items);

        let phone_numbers = KVParser::builder()
            .with_scope_patterns((regex!(r"function scr_phonename\(\)"), regex!("$")))
            .with_iter_patterns((regex!(r"case (?:\d+):"), regex!("break;")))
            .with_key_pattern(regex!(r"^case (\d+):"))
            .with_value_pattern(regex!(r"global\.phonename\[i] ?= ?(.*);\n"))
            .parse(&phone_numbers_code)
            .unwrap();
        let phone_numbers = Self::deindex(phone_numbers);

        let room_names = KVParser::builder()
            .with_scope_patterns((regex!(r"function scr_roomname\(arg0\)"), regex!("$")))
            .with_iter_patterns((regex!(r"if ?\(arg0 ?== ?(?:\d+)\)"), regex!(r"\n    }")))
            .with_key_pattern(regex!(r"^if ?\(arg0 ?== ?(\d+)\)"))
            .with_value_pattern(regex!(r#"roomname ?= ?(.*);"#))
            .parse(&room_names_code)
            .unwrap();
        let room_names = Self::deindex(room_names);

        let spells = KVParser::builder()
            .with_scope_patterns((regex!(r"function scr_spellinfo\(arg0\)"), regex!("$")))
            .with_iter_patterns((regex!(r"case (?:\d+):"), regex!("break;")))
            .with_key_pattern(regex!(r"^case (\d+):"))
            .with_value_pattern(regex!(r#"spellname ?= ?(.*);"#))
            .parse(&spells_code)
            .unwrap();
        let spells = Self::deindex(spells);

        let weapons = KVParser::builder()
            .with_scope_patterns((regex!(r"function scr_weaponinfo\(arg0\)"), regex!("$")))
            .with_iter_patterns((regex!(r"case (?:\d+):"), regex!("break;")))
            .with_key_pattern(regex!(r"^case (\d+):"))
            .with_value_pattern(regex!(r#"weaponnametemp ?= ?(.*);"#))
            .parse(&weapons_code)
            .unwrap();
        let weapons = Self::deindex(weapons);

        Ok(ChapterData {
            armors,
            items,
            key_items,
            light_world_items,
            phone_numbers,
            room_assets,
            room_ids,
            room_names,
            spells,
            weapons,
        })
    }

    fn find_room_assets_and_ids(room_ids_code: &str) -> Vec<(AssetName, RoomId)> {
        let start_regex = regex!(r"\nfunction scr_get_room_list\(\)\n");
        let Some(start_match) = start_regex.find(room_ids_code) else {
            return Vec::new();
        };
        let start_index = start_match.end();

        let iter_regex = regex!(r"new scr_room\((\w+), (\d+)\)");
        let matches = iter_regex.captures_iter(&room_ids_code[start_index..]);

        matches
            .map(|mat| {
                let asset_name = mat[1].to_string();
                let room_id = mat[2].to_string().parse().unwrap();
                (asset_name, room_id)
            })
            .collect()
    }

    fn deindex(input: Vec<(&str, &str)>) -> Vec<Option<String>> {
        let input = input
            .into_iter()
            .map(|(k, v)| (k.parse::<usize>().unwrap(), v));
        let max_index = input.clone().map(|(k, v)| k).max().unwrap();
        let mut output = vec![None; max_index + 1];

        for (k, v) in input {
            output[k] = Some(v.to_string());
        }

        output
    }
}

fn main() -> eyre::Result<()> {
    dotenv().ok();
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let cli = Cli::parse();

    let (_tempdir, tempdir_path) = if let Some(tempdir_path) = &cli.utmt_output_path {
        let path = absolute(tempdir_path)?;
        debug!("Using UTMT output dir at {}.", path.display());
        create_dir_all(&path)?;
        (None, path)
    } else {
        let tempdir = tempdir()?;
        debug!("Created temp dir at {}.", tempdir.path().display());
        let path = absolute(tempdir.path())?;
        (Some(tempdir), path)
    };

    for ch in 1..=CHAPTER_COUNT {
        info!("Extracting chapter {}.", ch);
        let data = ChapterData::load(&cli, ch, &tempdir_path)?;

        let file_path = format!("chapter_{}.json", ch);
        let file = File::create(&file_path)?;
        serde_json::to_writer(&file, &data)?;
        info!("Chapter {} data saved in {}.", ch, file_path);
    }

    Ok(())
}
