//! The mode's start check. Warcraft Survivors draws the client's own art, models and sounds, and
//! ships none of them, so without a WoW 1.12.1 `Data` folder the window opens onto a loading screen
//! that never clears. The launch names the folders it looked in instead.

use std::path::{Path, PathBuf};

/// The two MPQs the client cannot boot without. `scripts/fetch-data.sh` uses the same pair.
const CANARIES: [&str; 2] = ["terrain.MPQ", "patch.MPQ"];

/// The message for a launch that has no complete `Data` folder, or `None` when it has one. `found`
/// is the folder the resolver picked, `searched` every candidate it tried, and `is_file` says
/// whether a path is a regular file.
pub(crate) fn data_report(
    found: Option<&Path>,
    searched: &[PathBuf],
    is_file: impl Fn(&Path) -> bool,
) -> Option<String> {
    let Some(dir) = found else {
        let mut msg = String::from(
            "Warcraft Survivors needs a WoW 1.12.1 Data folder, and none was found.\nLooked in:\n",
        );
        if searched.is_empty() {
            msg.push_str("  (none: WOW_DATA is set but empty, which turns the search off)\n");
        }
        for path in searched {
            msg.push_str(&format!("  {}\n", path.display()));
        }
        msg.push_str(
            "Set WOW_DATA to the Data folder that holds terrain.MPQ and patch.MPQ. \
             See docs/windows-development.md.",
        );
        return Some(msg);
    };
    let missing: Vec<&str> = CANARIES
        .iter()
        .copied()
        .filter(|name| !is_file(&dir.join(name)))
        .collect();
    if missing.is_empty() {
        return None;
    }
    Some(format!(
        "Found a Data folder at {} but it is missing {}. Point WOW_DATA at a complete \
         WoW 1.12.1 Data folder. See docs/windows-development.md.",
        dir.display(),
        missing.join(" and "),
    ))
}

/// [`data_report`] for this launch, against the folders the resolver really searches.
pub(crate) fn check_wow_data() -> Option<String> {
    data_report(
        benilla_formats::wow_data().as_deref(),
        &benilla_formats::candidates(),
        |path| path.is_file(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates() -> Vec<PathBuf> {
        vec![PathBuf::from("/env/Data"), PathBuf::from("/exe/Data")]
    }

    #[test]
    fn no_folder_names_every_path_searched() {
        let msg = data_report(None, &candidates(), |_| false).expect("a report");
        assert!(msg.contains("none was found"));
        assert!(msg.contains(&PathBuf::from("/env/Data").display().to_string()));
        assert!(msg.contains(&PathBuf::from("/exe/Data").display().to_string()));
    }

    #[test]
    fn empty_search_says_why() {
        let msg = data_report(None, &[], |_| false).expect("a report");
        assert!(msg.contains("WOW_DATA is set but empty"));
    }

    #[test]
    fn complete_folder_passes() {
        let dir = PathBuf::from("/env/Data");
        let present = |p: &Path| p.starts_with(&dir);
        assert_eq!(data_report(Some(&dir), &candidates(), present), None);
    }

    #[test]
    fn folder_missing_one_canary_names_it() {
        let dir = PathBuf::from("/env/Data");
        let only_terrain = |p: &Path| p == dir.join("terrain.MPQ");
        let msg = data_report(Some(&dir), &candidates(), only_terrain).expect("a report");
        assert!(msg.contains("patch.MPQ"));
        assert!(!msg.contains("terrain.MPQ"));
    }

    #[test]
    fn folder_missing_both_canaries_names_both() {
        let dir = PathBuf::from("/env/Data");
        let msg = data_report(Some(&dir), &candidates(), |_| false).expect("a report");
        assert!(msg.contains("terrain.MPQ and patch.MPQ"));
    }
}
