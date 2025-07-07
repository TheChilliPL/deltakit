use std::fmt::Display;
use std::ops::{Add, Sub};
use compact_str::CompactString;
use indoc::indoc;
use log::warn;
use crate::gamedata::armors::display_armor;
use crate::gamedata::items::display_item;
use crate::gamedata::key_items::display_key_item;
use crate::gamedata::lightworld_items::display_lightworld_item;
use crate::gamedata::phone_numbers::display_phone_number;
use crate::gamedata::weapons::display_weapon;
use crate::savefile::{ItemStats, LightworldStats, SaveData, Stats};
use crate::serialize::Serializable;

#[derive(Clone, Debug)]
pub enum MergeResult<T> {
    Resolved(T),
    Conflict {
        ours: T,
        theirs: T,
        ancestor: Option<T>
    },
}

impl <T> MergeResult<T> {
    pub fn map<T2>(self, f: impl Fn(T) -> T2) -> MergeResult<T2> {
        match self {
            MergeResult::Resolved(v) => MergeResult::Resolved(f(v)),
            MergeResult::Conflict { ours, theirs, ancestor } => MergeResult::Conflict {
                ours: f(ours),
                theirs: f(theirs),
                ancestor: ancestor.map(f)
            }
        }
    }

    pub fn map_conflict(self, f: impl FnOnce(T, T, Option<T>) -> MergeResult<T>) -> MergeResult<T> {
        match self {
            MergeResult::Resolved(v) => MergeResult::Resolved(v),
            MergeResult::Conflict { ours, theirs, ancestor } => f(ours, theirs, ancestor),
        }
    }
}

impl <T: Display> MergeResult<T> {
    pub fn to_merge_string(&self, conflict_marker_length: usize) -> String {
        match self {
            MergeResult::Resolved(v) => v.to_string(),
            MergeResult::Conflict { ours, theirs, ancestor: Some(ancestor) } => {
                let ours_str = ours.to_string();
                let theirs_str = theirs.to_string();
                let ancestor_str = ancestor.to_string();
                let marker_start = "<".repeat(conflict_marker_length);
                let marker_o_a = "|".repeat(conflict_marker_length);
                let marker_a_t = "=".repeat(conflict_marker_length);
                let marker_end = ">".repeat(conflict_marker_length);

                format!(
                    indoc!{"
                        {} ours
                        {}
                        {} ancestor
                        {}
                        {}
                        {}
                        {} theirs"},
                    marker_start,
                    ours_str,
                    marker_o_a,
                    ancestor_str,
                    marker_a_t,
                    theirs_str,
                    marker_end,
                )
            }
            MergeResult::Conflict { ours, theirs, ancestor: None } => {
                let ours_str = ours.to_string();
                let theirs_str = theirs.to_string();
                let marker_start = "<".repeat(conflict_marker_length);
                let marker_o_t = "=".repeat(conflict_marker_length);
                let marker_end = ">".repeat(conflict_marker_length);

                format!(
                    indoc!{"
                        {} ours
                        {}
                        {}
                        {}
                        {} theirs"},
                    marker_start,
                    ours_str,
                    marker_o_t,
                    theirs_str,
                    marker_end,
                )
            }
        }
    }

    pub fn to_merge_compact_string(&self, conflict_marker_length: usize) -> CompactString {
        self.to_merge_string(conflict_marker_length).serialize()
    }
}

/// Merges two values with a simple strategy.
///
/// If both ours and theirs are the same, it's resolved.
/// If ours and theirs are different, but only one changed relative to the ancestor, apply the
/// change.
/// Otherwise, conflict.
fn merge_simple<T: PartialEq>(
    ours: T,
    theirs: T,
    ancestor: Option<T>,
) -> MergeResult<T> {
    match (ours, theirs, ancestor) {
        (o, t, _) if o == t => MergeResult::Resolved(o),
        (o, t, Some(a)) if a == o => MergeResult::Resolved(t),
        (o, t, Some(a)) if a == t => MergeResult::Resolved(o),
        (o, t, a) => MergeResult::Conflict {
            ours: o,
            theirs: t,
            ancestor: a
        }
    }
}

fn merge_same<T: PartialEq>(
    ours: T,
    theirs: T,
    ancestor: Option<T>,
) -> MergeResult<T> {
    match (ours, theirs, ancestor) {
        (o, t, _) if o == t => MergeResult::Resolved(o),
        (o, t, a) => MergeResult::Conflict {
            ours: o,
            theirs: t,
            ancestor: a
        }
    }
}

fn merge_values<
    T: PartialEq
        + Copy
        + Add<Output = T>
        + Sub<Output = T>
        + Ord
        + Display
>(
    ours: T,
    theirs: T,
    ancestor: Option<T>,
    min: Option<T>,
    max: Option<T>,
    merge_name: &str,
) -> MergeResult<T> {
    match (ours, theirs, ancestor) {
        (o, t, _) if o == t => MergeResult::Resolved(o),
        (o, t, Some(a)) => {
            let delta_o = o - a;
            let delta_t = t - a;

            let mut value = a + delta_o + delta_t;

            warn!(
                "merging {}: <<< o {} ||| a {} === t {} >>> -> {}",
                merge_name, o, a, t, value
            );

            if min.is_some_and(|m| value < m) {
                value = min.unwrap();
            } else if max.is_some_and(|m| value > m) {
                value = max.unwrap();
            }

            MergeResult::Resolved(value)
        }
        (o, t, None) => {
            warn!("merging {}: set to max of {}, {}", merge_name, o, t);
            MergeResult::Resolved(if t > o { t } else { o })
        }
    }
}

fn merge_max<
    T: PartialEq
    + PartialOrd
>(
    ours: T,
    theirs: T,
) -> MergeResult<T> {
    MergeResult::Resolved(if theirs > ours { theirs } else { ours })
}

fn merge_item_stats(
    output: &mut Vec<MergeResult<CompactString>>,
    ours: &ItemStats,
    theirs: &ItemStats,
    ancestor: Option<&ItemStats>,
    chapter: i32,
) -> Result<(), &'static str> {
    output.push(merge_simple(ours.attack, theirs.attack, ancestor.map(|a| a.attack)).map(|v| v.serialize()));
    output.push(merge_simple(ours.defense, theirs.defense, ancestor.map(|a| a.defense)).map(|v| v.serialize()));
    output.push(merge_simple(ours.magic, theirs.magic, ancestor.map(|a| a.magic)).map(|v| v.serialize()));
    output.push(merge_simple(ours.bolts, theirs.bolts, ancestor.map(|a| a.bolts)).map(|v| v.serialize()));
    output.push(merge_simple(ours.graze_amount, theirs.graze_amount, ancestor.map(|a| a.graze_amount)).map(|v| v.serialize()));
    output.push(merge_simple(ours.graze_size, theirs.graze_size, ancestor.map(|a| a.graze_size)).map(|v| v.serialize()));
    output.push(merge_simple(ours.bolts_speed, theirs.bolts_speed, ancestor.map(|a| a.bolts_speed)).map(|v| v.serialize()));
    output.push(merge_simple(ours.item_special, theirs.item_special, ancestor.map(|a| a.item_special)).map(|v| v.serialize()));
    if chapter > 1 {
        output.push(merge_simple(ours.item_element, theirs.item_element, ancestor.map(|a| a.item_element)).map(|v| v.serialize()));
        output.push(merge_simple(ours.item_element_amount, theirs.item_element_amount, ancestor.map(|a| a.item_element_amount)).map(|v| v.serialize()));
    }

    Ok(())
}

fn merge_stats(
    output: &mut Vec<MergeResult<CompactString>>,
    ours: &Stats,
    theirs: &Stats,
    ancestor: Option<&Stats>,
    chapter: i32,
) -> Result<(), &'static str> {
    let max_hp = merge_simple(ours.max_hp, theirs.max_hp, ancestor.map(|a| a.max_hp))
        .map(|v| v.serialize());

    output.push(max_hp.clone()); // HP, healed
    output.push(max_hp);
    output.push(merge_simple(ours.attack, theirs.attack, ancestor.map(|a| a.attack)).map(|v| v.serialize()));
    output.push(merge_simple(ours.defense, theirs.defense, ancestor.map(|a| a.defense)).map(|v| v.serialize()));
    output.push(merge_simple(ours.magic, theirs.magic, ancestor.map(|a| a.magic)).map(|v| v.serialize()));
    output.push(merge_simple(ours.guts, theirs.guts, ancestor.map(|a| a.guts)).map(|v| v.serialize()));
    output.push(MergeResult::Resolved(ours.weapon.serialize()));
    output.push(MergeResult::Resolved(ours.armor1.serialize()));
    output.push(MergeResult::Resolved(ours.armor2.serialize()));
    output.push(MergeResult::Resolved(ours.weapon_style.clone()));

    for i in 0..4 {
        output.push(MergeResult::Resolved(ours.item_stats[i].attack.serialize()));
        output.push(MergeResult::Resolved(ours.item_stats[i].defense.serialize()));
        output.push(MergeResult::Resolved(ours.item_stats[i].magic.serialize()));
        output.push(MergeResult::Resolved(ours.item_stats[i].bolts.serialize()));
        output.push(MergeResult::Resolved(ours.item_stats[i].graze_amount.serialize()));
        output.push(MergeResult::Resolved(ours.item_stats[i].graze_size.serialize()));
        output.push(MergeResult::Resolved(ours.item_stats[i].bolts_speed.serialize()));
        output.push(MergeResult::Resolved(ours.item_stats[i].item_special.serialize()));
        if chapter > 1 {
            output.push(MergeResult::Resolved(ours.item_stats[i].item_element.serialize()));
            output.push(MergeResult::Resolved(ours.item_stats[i].item_element_amount.serialize()));
        }
    }

    for i in 0..12 {
        output.push(
            merge_same(ours.spells[i], theirs.spells[i], ancestor.map(|a| a.spells[i]))
                .map(|v| v.serialize())
        );
    }

    Ok(())
}

fn count_in_array<T : PartialEq>(
    array: &[T],
    value: T,
) -> usize {
    array.iter().filter(|v| **v == value).count()
}

fn add_to_inventory<T: Copy + Default + PartialEq>(
    inventory: &mut [T],
    item: T,
    start_index: usize,
) -> Result<usize, ()> {
    let no_item = T::default();

    for i in start_index..inventory.len() {
        if inventory[i] == no_item {
            inventory[i] = item;
            return Ok(i);
        }
    }

    for i in (0..start_index).rev() {
        if inventory[i] == no_item {
            inventory[i] = item;
            return Ok(i);
        }
    }

    Err(())
}

fn merge_inventories<T : Copy + Default + PartialEq>(
    ours: &mut [T],
    theirs: &[T],
    item_kind: &str,
    display: impl Fn(T) -> CompactString,
) {
    let no_item = T::default();
    let mut theirs = theirs.to_owned();

    for i in 0..theirs.len() {
        if theirs[i] == no_item { continue; }

        let count_ours = count_in_array(ours, theirs[i]);
        let count_theirs = count_in_array(&theirs, theirs[i]);

        if count_theirs > count_ours {
            let res = add_to_inventory(ours, theirs[i], i);

            theirs[i] = no_item;

            if res.is_err() {
                warn!("could not add {} {} to inventory", item_kind, display(theirs[i]));
            }
        }
    }
}

fn merge_lightworld_stats(
    output: &mut Vec<MergeResult<CompactString>>,
    ours: &LightworldStats,
    theirs: &LightworldStats,
    ancestor: Option<&LightworldStats>,
) {
    output.push(merge_simple(ours.weapon, theirs.weapon, ancestor.map(|a| a.weapon)).map(|v| v.serialize()));
    output.push(merge_simple(ours.armor, theirs.armor, ancestor.map(|a| a.armor)).map(|v| v.serialize()));
    output.push(merge_simple(ours.xp, theirs.xp, ancestor.map(|a| a.xp)).map(|v| v.serialize()));
    output.push(merge_simple(ours.lv, theirs.lv, ancestor.map(|a| a.lv)).map(|v| v.serialize()));
    output.push(merge_simple(ours.gold, theirs.gold, ancestor.map(|a| a.gold)).map(|v| v.serialize()));
    output.push(merge_simple(ours.hp, theirs.hp, ancestor.map(|a| a.hp)).map(|v| v.serialize()));
    output.push(merge_simple(ours.max_hp, theirs.max_hp, ancestor.map(|a| a.max_hp)).map(|v| v.serialize()));
    output.push(merge_simple(ours.attack, theirs.attack, ancestor.map(|a| a.attack)).map(|v| v.serialize()));
    output.push(merge_simple(ours.defense, theirs.defense, ancestor.map(|a| a.defense)).map(|v| v.serialize()));
    output.push(merge_simple(ours.wstrength, theirs.wstrength, ancestor.map(|a| a.wstrength)).map(|v| v.serialize()));
    output.push(merge_simple(ours.adef, theirs.adef, ancestor.map(|a| a.adef)).map(|v| v.serialize()));
}

pub fn merge_savefiles(
    ours: &SaveData,
    theirs: &SaveData,
    ancestor: Option<&SaveData>,
) -> Result<Vec<MergeResult<CompactString>>, &'static str> {
    let chapter = ours.chapter;

    if theirs.chapter > chapter || ancestor.is_some_and(|a| a.chapter > chapter) {
        return Err("Chapter mismatch")
    }

    let mut data: Vec<MergeResult<CompactString>> = Vec::with_capacity(if chapter == 1 { 10318 } 
    else { 3055 });

    data.push(MergeResult::Resolved(ours.true_name.serialize()));

    for i in 0..6 {
        data.push(MergeResult::Resolved(ours.vessel_names[i].serialize()));
    }

    for i in 0..3 {
        data.push(MergeResult::Resolved(ours.party[i].serialize()));
    }

    data.push(
        merge_values(
            ours.dark_dollars,
            theirs.dark_dollars, 
            ancestor.map(|a| a.dark_dollars),
            Some(0),
            None,
            "dark dollars",
        ).map(|v| v.serialize())
    );

    data.push(
        merge_max(
            ours.xp,
            theirs.xp,
        ).map(|v| v.serialize())
    );

    data.push(
        merge_max(
            ours.level,
            theirs.level,
        ).map(|v| v.serialize())
    );

    data.push(
        // merge_simple(
        //     ours.inv,
        //     theirs.inv,
        //     ancestor.map(|a| a.inv),
        // ).map(|v| v.serialize())
        MergeResult::Resolved(ours.inv.serialize())
    );

    data.push(
        // merge_simple(
        //     ours.invc,
        //     theirs.invc,
        //     ancestor.map(|a| a.invc),
        // ).map(|v| v.serialize())
        MergeResult::Resolved(ours.invc.serialize())
    );

    data.push(
        MergeResult::Resolved(if ours.is_darkworld { "1" } else { "0" }.serialize())
    );

    // stats
    let stat_blocks = match chapter {
        1 => 4,
        _ => 5,
    };

    for i in 0..stat_blocks {
        merge_stats(
            &mut data,
            &ours.stats[i],
            &theirs.stats[i],
            ancestor.map(|a| &a.stats[i]),
            chapter,
        )?;
    }

    data.push(merge_simple(ours.bolt_speed, theirs.bolt_speed, ancestor.map(|a| a.bolt_speed)).map(|v| v.serialize()));
    data.push(merge_simple(ours.graze_amount, theirs.graze_amount, ancestor.map(|a| a.graze_amount)).map(|v| v.serialize()));
    data.push(merge_simple(ours.graze_size, theirs.graze_size, ancestor.map(|a| a.graze_size)).map(|v| v.serialize()));

    // INVENTORY, STORAGE...

    let mut inventory_and_storage = Vec::with_capacity(12 + if chapter > 1 { 72 } else { 0 });
    inventory_and_storage.extend_from_slice(&ours.inventory[0..12]);
    if chapter > 1 {
        inventory_and_storage.extend_from_slice(ours.storage.as_ref().unwrap());
    }

    let mut their_inventory_and_storage = Vec::with_capacity(12 + if chapter > 1 { 72 } else { 0 });
    their_inventory_and_storage.extend_from_slice(&theirs.inventory[0..12]);
    if chapter > 1 {
        their_inventory_and_storage.extend_from_slice(theirs.storage.as_ref().unwrap());
    }

    let mut key_items = ours.key_items;
    let mut weapons = ours.weapons.clone();
    let mut their_weapons = theirs.weapons.clone();
    for (i, stats) in theirs.stats.iter().enumerate() {
        if stats.weapon != ours.stats[i].weapon {
            their_weapons.push(stats.weapon);
            // TODO Ensure it doesn't clone weapons and armors transferred between
            //      characters
        }
    }
    let mut armors = ours.armors.clone();
    for (i, stats) in theirs.stats.iter().enumerate() {
        if stats.armor1 != ours.stats[i].armor1 {
            armors.push(stats.armor1);
        }
        if stats.armor2 != ours.stats[i].armor2 {
            armors.push(stats.armor2);
        }
    }

    merge_inventories(
        &mut inventory_and_storage,
        &their_inventory_and_storage,
        "item",
        display_item,
    );

    merge_inventories(
        &mut key_items,
        &theirs.key_items,
        "key item",
        display_key_item,
    );

    merge_inventories(
        &mut weapons,
        &their_weapons,
        "weapon",
        display_weapon,
    );

    merge_inventories(
        &mut armors,
        &their_weapons,
        "armor",
        display_armor,
    );

    for i in 0..12 {
        data.push(MergeResult::Resolved(inventory_and_storage[i].serialize()));
        data.push(MergeResult::Resolved(key_items[i].serialize()));

        if chapter == 1 {
            data.push(MergeResult::Resolved(weapons[i].serialize()));
            data.push(MergeResult::Resolved(armors[i].serialize()));
        }
    }

    data.push(MergeResult::Resolved(ours.inventory[12].serialize()));
    data.push(MergeResult::Resolved(key_items[12].serialize()));

    if chapter == 1 {
        data.push(MergeResult::Resolved(weapons[12].serialize()));
        data.push(MergeResult::Resolved(armors[12].serialize()));
    }

    if chapter != 1 {
        for i in 0..48 {
            data.push(MergeResult::Resolved(weapons[i].serialize()));
            data.push(MergeResult::Resolved(armors[i].serialize()));
        }

        for i in 0..72 {
            data.push(MergeResult::Resolved(inventory_and_storage[i + 12].serialize()));
        }
    }

    data.push(merge_simple(ours.tension, theirs.tension, ancestor.map(|a| a.tension)).map(|v| v.serialize()));
    data.push(merge_simple(ours.max_tension, theirs.max_tension, ancestor.map(|a| a.max_tension)).map(|v| v.serialize()));

    merge_lightworld_stats(
        &mut data,
        &ours.lightworld_stats,
        &theirs.lightworld_stats,
        ancestor.map(|a| &a.lightworld_stats),
    );

    let mut lightworld_items = ours.lightworld_items;
    let mut phone_numbers = ours.lightworld_phone;

    merge_inventories(
        &mut lightworld_items,
        &theirs.lightworld_items,
        "lightworld item",
        display_lightworld_item,
    );

    merge_inventories(
        &mut phone_numbers,
        &theirs.lightworld_phone,
        "phone number",
        display_phone_number,
    );

    for i in 0..8 {
        data.push(MergeResult::Resolved(lightworld_items[i].serialize()));
        data.push(MergeResult::Resolved(phone_numbers[i].serialize()));
    }

    for i in 0..2500 {
        if ancestor.is_some() {
            data.push(merge_simple(ours.flags[i], theirs.flags[i], ancestor.map(|a| a.flags[i])).map(|v| v.serialize()));
        } else {
            data.push(merge_max(ours.flags[i], theirs.flags[i]).map(|v| v.serialize()));
        }
    }

    if chapter == 1 {
        for _ in 2500..9999 {
            data.push(MergeResult::Resolved("0".serialize()));
        }
    }

    data.push(MergeResult::Resolved(ours.plot_value.serialize()));
    data.push(MergeResult::Resolved(ours.room_id.serialize()));
    if ancestor.is_some() {
        data.push(merge_values(
            ours.time_played.as_secs() * 30,
            theirs.time_played.as_secs() * 30,
            ancestor.map(|a| a.time_played.as_secs() * 30),
            None, None,
            "time played"
        ).map(|v| v.serialize()));
    } else {
        data.push(merge_max(ours.time_played.as_secs() * 30, theirs.time_played.as_secs() * 30).map(|v| v.serialize()));
    }

    assert_eq!(data.len(), if chapter == 1 { 10318 } else { 3055 }, "Unexpected line count");

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_markers() {
        let conflict = MergeResult::Conflict {
            ours: "A",
            theirs: "B",
            ancestor: Some("C")
        };

        assert_eq!(
            conflict.to_merge_string(5),
            indoc!{"
                <<<<< ours
                A
                ||||| ancestor
                C
                =====
                B
                >>>>> theirs"},
        );
    }

    #[test]
    fn test_merge_inventories() {
        let mut ours = [0, 1, 0, 2, 0, 3];
        let theirs = [1, 1, 2, 3, 4];

        let expected = [1, 1, 0, 2, 4, 3];

        merge_inventories(
            &mut ours,
            &theirs,
            "",
            |_| CompactString::default(),
        );

        assert_eq!(ours, expected);
    }
}
