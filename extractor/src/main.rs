use std::collections::HashMap;
use std::io::Write;
use clap::Parser;
use dotenv::dotenv;
use eyre::eyre;
use log::{LevelFilter, debug, info, trace, warn};
use regex::{Regex, regex};
use std::fs::{File, create_dir_all, read_to_string};
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
    let script_path = options.utmt_cli_path.join("../Scripts/Technical Scripts/ExportAssetOrder.csx");
    let out_txt_path = tempdir.join("asset_order.txt");

    let mut cmd = Command::new(&options.utmt_cli_path);
    cmd.arg("load")
        .arg(get_data_win_path(options,chapter))
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
            if let Some(prev_asset_type) = current_asset_type.take() && !current_asset_list.is_empty() {
                let prev_asset_list = current_asset_list;
                current_asset_list = Vec::new();
                hashmap.insert(prev_asset_type, prev_asset_list);
            }

            let asset_type = &line[2..line.len()-2];
            current_asset_type = Some(asset_type.to_string());
        } else {
            current_asset_list.push(line.to_string());
        }
    }

    if let Some(last_asset_type) = current_asset_type.take() && !current_asset_list.is_empty() {
        let last_asset_list = current_asset_list;
        hashmap.insert(last_asset_type, last_asset_list);
    }

    Ok(hashmap)
}

type AssetName = String;
type RoomId = NonZero<u32>;

#[derive(Debug)]
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
            asset_order.remove("rooms").ok_or_else(|| eyre!("couldn't get room asset order"))?
        };

        let mut room_ids = vec![None; room_assets.len()];

        let rooms_assets_ids = Self::find_room_assets_and_ids(&room_ids_code);
        for (asset_name, id) in rooms_assets_ids {
            let index = room_assets.iter()
                .position(|it| it == &asset_name)
                .ok_or_else(|| eyre!("couldn't find index of room {asset_name}"))?;

            room_ids[index] = Some(id);
        }

        debug!("Rooms: {:?}", room_ids.iter().zip(&room_assets).collect::<Vec<_>>());

        let armors = Self::find_strings(
            &armors_code,
            regex!(r"function scr_armorinfo\(arg0\)"),
            regex!("$"),
            regex!(r"case (\d+):"),
            regex!(r#"armornametemp ?= ?(?:stringsetloc\("(.*)", ".*"\)|"(.*)");"#),
        )?;
        let armors = Self::deindex(armors);

        let items = Self::find_strings(
            &items_code,
            regex!(r"function scr_iteminfo\(arg0\)"),
            regex!("$"),
            regex!(r"case (\d+):"),
            regex!(r#"itemnameb ?= ?(?:stringsetloc\("(.*)", ".*"\)|"(.*)");"#),
        )?;
        let items = Self::deindex(items);

        let key_items = Self::find_strings(
            &key_items_code,
            regex!(r"function scr_keyiteminfo\(arg0\)"),
            regex!("$"),
            regex!(r"case (\d+):"),
            regex!(r#"tempkeyitemname ?= ?(?:stringsetloc\("(.*)", ".*"\)|"(.*)");"#),
        )?;
        let key_items = Self::deindex(key_items);

        let light_world_items = Self::find_strings(
            &light_world_items_code,
            regex!(r"function scr_litemname\(\)"),
            regex!("$"),
            regex!(r"if ?\(itemid ?== ?(\d+)\)"),
            regex!(r#"global\.litemname\[i] ?= ?(?:stringsetloc\("(.*)", ".*"\)|"(.*)");"#),
        )?;
        let light_world_items = Self::deindex(light_world_items);

        let phone_numbers = Self::find_strings(
            &phone_numbers_code,
            regex!(r"function scr_phonename\(\)"),
            regex!("$"),
            regex!(r"case (\d+):"),
            regex!(r#"global\.phonename\[i] ?= ?(?:stringsetloc\("(.*)", ".*"\)|"(.*)");"#),
        )?;
        let phone_numbers = Self::deindex(phone_numbers);

        let room_names = Self::find_strings(
            &room_names_code,
            regex!(r"function scr_roomname\(arg0\)"),
            regex!("$"),
            regex!(r"if ?\(arg0 ?== ?(\d+)\)"),
            regex!(r#"roomname ?= ?(?:stringsetloc\("(.*)", ".*"\)|"(.*)");"#),
        )?;
        let room_names = Self::deindex(room_names);

        let spells = Self::find_strings(
            &spells_code,
            regex!(r"function scr_spellinfo\(arg0\)"),
            regex!("$"),
            regex!(r"case (\d+):"),
            regex!(r#"spellname ?= ?(?:stringsetloc\("(.*)", ".*"\)|"(.*)");"#),
        )?;
        let spells = Self::deindex(spells);

        let weapons = Self::find_strings(
            &weapons_code,
            regex!(r"function scr_weaponinfo\(arg0\)"),
            regex!("$"),
            regex!(r"case (\d+):"),
            regex!(r#"weaponnametemp ?= ?(?:stringsetloc\("(.*)", ".*"\)|"(.*)");"#),
        )?;
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

        matches.map(|mat| {
            let asset_name = mat[1].to_string();
            let room_id = mat[2].to_string().parse().unwrap();
            (asset_name, room_id)
        }).collect()
    }

    fn find_strings(code: &str, start_pat: &Regex, end_pat: &Regex, iter_pat: &Regex, str_pat: &Regex) -> eyre::Result<Vec<(String, String)>> {
        debug_assert!(start_pat.static_captures_len() == Some(1), "start_pat shouldn't have capture groups");
        debug_assert!(end_pat.static_captures_len() == Some(1), "end_pat shouldn't have capture groups");
        debug_assert!(iter_pat.static_captures_len() == Some(2), "iter_pat pattern does not have exactly one capture group");
        debug_assert!(str_pat.static_captures_len() == Some(2), "str_pat pattern does not have exactly one capture group");

        let start_match = start_pat.find(code).ok_or_else(|| eyre!("couldn't find starting pattern"))?;
        let end_match = end_pat.find_at(code, start_match.end()).ok_or_else(|| eyre!("couldn't find ending pattern"))?;

        let code = &code[start_match.end()..end_match.start()];

        let iter_matches = iter_pat.captures_iter(code);

        let mut vec = Vec::with_capacity(iter_matches.size_hint().0);

        for mat in iter_matches {
            let key = &mat[1];
            let str_match = str_pat.captures_at(code, mat.get_match().end()).ok_or_else(|| eyre!("couldn't find value for key {}", key))?;

            let value = str_match.iter().skip(1).flatten().next().unwrap().as_str();

            vec.push((key.to_string(), value.to_string()));
        }

        Ok(vec)
    }

    fn deindex(input: Vec<(String, String)>) -> Vec<Option<String>> {
        let input = input.into_iter().map(|(k, v)| (k.parse::<usize>().unwrap(), v));
        let max_index = input.clone().map(|(k,v)| k).max().unwrap();
        let mut output = vec![None; max_index + 1];

        for (k, v) in input {
            output[k] = Some(v);
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

    let ch = ChapterData::load(&cli, 5, &tempdir_path)?;

    info!("Chapter data: {ch:#?}");

    Ok(())
}
