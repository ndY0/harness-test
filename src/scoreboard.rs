use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u32,
}

fn scores_dir() -> PathBuf {
    let base = if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local").join("share")
    } else {
        PathBuf::from(".")
    };
    base.join("pacman-terminal")
}

fn scores_path() -> PathBuf {
    scores_dir().join("scores.json")
}

pub fn load_scores() -> io::Result<Vec<ScoreEntry>> {
    let path = scores_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<Vec<ScoreEntry>>(&content) {
            Ok(entries) => Ok(entries),
            Err(e) => {
                eprintln!("Warning: scoreboard file corrupt ({}), starting fresh", e);
                Ok(Vec::new())
            }
        },
        Err(e) => {
            eprintln!("Warning: cannot read scoreboard file ({}), starting fresh", e);
            Ok(Vec::new())
        }
    }
}

pub fn save_scores(entries: &[ScoreEntry]) -> io::Result<()> {
    let dir = scores_dir();
    std::fs::create_dir_all(&dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }

    let path = scores_path();
    let tmp_path = path.with_extension("tmp");

    let json = serde_json::to_string_pretty(entries)?;
    {
        let mut f = std::fs::File::create(&tmp_path)?;
        f.write_all(json.as_bytes())?;
        f.flush()?;
    }
    std::fs::rename(&tmp_path, &path)?;
    Ok(())
}

pub fn is_top_10(entries: &[ScoreEntry], score: u32) -> bool {
    if entries.len() < 10 {
        return true;
    }
    if let Some(lowest) = entries.last() {
        score > lowest.score
    } else {
        true
    }
}

pub fn insert_score(entries: &mut Vec<ScoreEntry>, entry: ScoreEntry) {
    entries.push(entry);
    entries.sort_by_key(|b| std::cmp::Reverse(b.score));
    entries.truncate(10);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_scores() {
        let path = scores_path();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        let dir = scores_dir();
        if dir.exists() {
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn test_load_empty_when_no_file() {
        clean_scores();
        let entries = load_scores().unwrap();
        assert!(entries.is_empty());
        clean_scores();
    }

    #[test]
    fn test_save_and_load_scores() {
        clean_scores();
        let entries = vec![
            ScoreEntry { name: "AAA".to_string(), score: 1000 },
            ScoreEntry { name: "BBB".to_string(), score: 500 },
        ];
        save_scores(&entries).unwrap();
        let loaded = load_scores().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "AAA");
        assert_eq!(loaded[1].name, "BBB");
        clean_scores();
    }

    #[test]
    fn test_is_top_10_empty() {
        assert!(is_top_10(&[], 0));
    }

    #[test]
    fn test_is_top_10_not_full() {
        let entries = vec![ScoreEntry { name: "A".to_string(), score: 100 }];
        assert!(is_top_10(&entries, 50));
    }

    #[test]
    fn test_is_top_10_qualifies() {
        let entries: Vec<ScoreEntry> = (0..10)
            .map(|i| ScoreEntry { name: format!("{:0>3}", i), score: (100 * (10 - i)) as u32 })
            .collect();
        // Lowest score is 100
        assert!(is_top_10(&entries, 200));
        assert!(!is_top_10(&entries, 50));
    }

    #[test]
    fn test_insert_score_maintains_top_10() {
        let mut entries = Vec::new();
        for i in 0..12 {
            insert_score(&mut entries, ScoreEntry {
                name: format!("{:0>3}", i),
                score: i * 100,
            });
        }
        assert_eq!(entries.len(), 10);
        // Highest scores should be 1100 down to 200
        assert_eq!(entries[0].score, 1100);
        assert_eq!(entries[9].score, 200);
    }

    #[test]
    fn test_tie_retains_older_entry() {
        let mut entries = vec![
            ScoreEntry { name: "AAA".to_string(), score: 1000 },
        ];
        insert_score(&mut entries, ScoreEntry { name: "BBB".to_string(), score: 1000 });
        assert_eq!(entries.len(), 2);
        // Both have 1000, older entry (AAA) should be first
        assert_eq!(entries[0].name, "AAA");
    }
}
