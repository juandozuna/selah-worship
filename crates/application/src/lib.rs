use selah_domain::*;

pub trait ProjectRepository {
    fn load(&self) -> Result<Option<Project>, String>;
    fn save(&self, project: &Project) -> Result<(), String>;
}
pub trait BibleRepository {
    fn translations(&self) -> Result<Vec<String>, String>;
    fn verses(&self, translation: &str, passage: &Passage) -> Result<Vec<BibleVerse>, String>;
}
pub trait OutputPublisher {
    fn publish(&mut self, width: u32, height: u32, rgba: &[u8]) -> Result<(), String>;
}
pub trait CredentialStore {
    fn get(&self, provider: &str) -> Result<String, String>;
    fn set(&self, provider: &str, value: &str) -> Result<(), String>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Passage {
    pub book: String,
    pub start_chapter: u32,
    pub start_verse: u32,
    pub end_chapter: u32,
    pub end_verse: u32,
}

pub fn load_project(repo: &impl ProjectRepository, seed: Project) -> Result<Project, String> {
    let project = repo.load()?.unwrap_or(seed);
    project.validate()?;
    Ok(project)
}

pub fn save_project(repo: &impl ProjectRepository, project: &Project) -> Result<(), String> {
    project.validate()?;
    repo.save(project)
}

pub fn present(project: &Project, song_id: &str, index: usize) -> Result<LiveSnapshot, String> {
    project.layout.validate()?;
    let song = project
        .library
        .iter()
        .find(|s| s.id == song_id)
        .ok_or("Content not found")?;
    let block = song.blocks.get(index).ok_or("Block not found")?;
    Ok(LiveSnapshot {
        song_id: Some(song_id.into()),
        block_index: Some(index),
        block: Some(block.clone()),
        layout: project.layout.clone(),
        test_pattern: false,
        ai_translation: false,
        motion: None,
        media: project.media.clone(),
        stream_composite: project.stream_composite,
    })
}

pub fn clear(layout: &CanvasLayout) -> LiveSnapshot {
    LiveSnapshot {
        song_id: None,
        block_index: None,
        block: None,
        layout: layout.clone(),
        test_pattern: false,
        ai_translation: false,
        motion: None,
        media: Vec::new(),
        stream_composite: false,
    }
}

/// Clear lyrics while keeping media assets and the stream composite preference for standalone playback.
pub fn clear_with_media(project: &Project) -> LiveSnapshot {
    LiveSnapshot {
        song_id: None,
        block_index: None,
        block: None,
        layout: project.layout.clone(),
        test_pattern: false,
        ai_translation: false,
        motion: None,
        media: project.media.clone(),
        stream_composite: project.stream_composite,
    }
}

pub fn split_lyrics(text: &str, lines_per_block: usize) -> Vec<PresentationBlock> {
    let size = lines_per_block.clamp(1, 8);
    let mut blocks = Vec::new();
    let mut section = "Verse 1".to_string();
    let mut lines: Vec<String> = Vec::new();
    fn flush(
        lines: &mut Vec<String>,
        blocks: &mut Vec<PresentationBlock>,
        section: &str,
        size: usize,
    ) {
        for chunk in lines.chunks(size) {
            blocks.push(PresentationBlock {
                motion: None,
                id: format!("block-{}", blocks.len()),
                section: section.into(),
                primary: chunk.join("\n"),
                secondary: None,
                reference: None,
            });
        }
        lines.clear();
    }
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush(&mut lines, &mut blocks, &section, size);
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            flush(&mut lines, &mut blocks, &section, size);
            section = trimmed[1..trimmed.len() - 1].to_string();
        } else {
            lines.push(trimmed.to_string());
        }
    }
    flush(&mut lines, &mut blocks, &section, size);
    blocks
}

pub fn source_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| {
            let s = l.trim();
            !s.is_empty() && !(s.starts_with('[') && s.ends_with(']'))
        })
        .collect()
}
pub fn apply_analysis(
    text: &str,
    groups: &[AnalysisGroup],
) -> Result<Vec<PresentationBlock>, String> {
    let lines: Vec<&str> = source_lines(text);
    if groups.is_empty() {
        return Err("AI returned no groups".into());
    }
    let mut next = 0;
    let mut blocks = Vec::new();
    for g in groups {
        if g.start != next || g.end <= g.start || g.end > lines.len() || g.section.len() > 100 {
            return Err("AI must cover every source line once, in order".into());
        }
        blocks.push(PresentationBlock {
            motion: None,
            id: format!("block-{}", blocks.len()),
            section: g.section.clone(),
            primary: lines[g.start..g.end].join("\n"),
            secondary: None,
            reference: None,
        });
        next = g.end;
    }
    if next != lines.len() {
        return Err("AI omitted source lines".into());
    }
    Ok(blocks)
}

/// Enforce the operator's line limit after validating the AI's exact source coverage.
pub fn apply_analysis_with_lines(
    text: &str,
    groups: &[AnalysisGroup],
    limit: usize,
) -> Result<Vec<PresentationBlock>, String> {
    if !(1..=8).contains(&limit) {
        return Err("Choose 1–8 lines per block".into());
    }
    apply_analysis(text, groups)?;
    let mut section: Option<String> = None;
    let mut sections = Vec::new();
    let mut boundaries = std::collections::HashSet::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            boundaries.insert(sections.len());
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = Some(trimmed[1..trimmed.len() - 1].to_string());
            boundaries.insert(sections.len());
        } else {
            sections.push(section.clone());
        }
    }
    let lines = source_lines(text);
    let mut blocks = Vec::new();
    for group in groups {
        let mut start = group.start;
        while start < group.end {
            let mut end = (start + limit).min(group.end);
            if let Some(boundary) = (start + 1..end).find(|i| boundaries.contains(i)) {
                end = boundary;
            }
            blocks.push(PresentationBlock {
                id: format!("block-{}", blocks.len()),
                primary: lines[start..end].join("\n"),
                section: sections[start]
                    .clone()
                    .unwrap_or_else(|| group.section.clone()),
                secondary: None,
                reference: None,
                motion: None,
            });
            start = end;
        }
    }
    Ok(blocks)
}

pub const BOOKS: &[(&str, &str, &str)] = &[
    ("GEN", "Genesis", "Génesis"),
    ("EXO", "Exodus", "Éxodo"),
    ("LEV", "Leviticus", "Levítico"),
    ("NUM", "Numbers", "Números"),
    ("DEU", "Deuteronomy", "Deuteronomio"),
    ("JOS", "Joshua", "Josué"),
    ("JDG", "Judges", "Jueces"),
    ("RUT", "Ruth", "Rut"),
    ("1SA", "1 Samuel", "1 Samuel"),
    ("2SA", "2 Samuel", "2 Samuel"),
    ("1KI", "1 Kings", "1 Reyes"),
    ("2KI", "2 Kings", "2 Reyes"),
    ("1CH", "1 Chronicles", "1 Crónicas"),
    ("2CH", "2 Chronicles", "2 Crónicas"),
    ("EZR", "Ezra", "Esdras"),
    ("NEH", "Nehemiah", "Nehemías"),
    ("EST", "Esther", "Ester"),
    ("JOB", "Job", "Job"),
    ("PSA", "Psalms", "Salmos"),
    ("PRO", "Proverbs", "Proverbios"),
    ("ECC", "Ecclesiastes", "Eclesiastés"),
    ("SNG", "Song of Songs", "Cantares"),
    ("ISA", "Isaiah", "Isaías"),
    ("JER", "Jeremiah", "Jeremías"),
    ("LAM", "Lamentations", "Lamentaciones"),
    ("EZK", "Ezekiel", "Ezequiel"),
    ("DAN", "Daniel", "Daniel"),
    ("HOS", "Hosea", "Oseas"),
    ("JOL", "Joel", "Joel"),
    ("AMO", "Amos", "Amós"),
    ("OBA", "Obadiah", "Abdías"),
    ("JON", "Jonah", "Jonás"),
    ("MIC", "Micah", "Miqueas"),
    ("NAM", "Nahum", "Nahúm"),
    ("HAB", "Habakkuk", "Habacuc"),
    ("ZEP", "Zephaniah", "Sofonías"),
    ("HAG", "Haggai", "Hageo"),
    ("ZEC", "Zechariah", "Zacarías"),
    ("MAL", "Malachi", "Malaquías"),
    ("MAT", "Matthew", "Mateo"),
    ("MRK", "Mark", "Marcos"),
    ("LUK", "Luke", "Lucas"),
    ("JHN", "John", "Juan"),
    ("ACT", "Acts", "Hechos"),
    ("ROM", "Romans", "Romanos"),
    ("1CO", "1 Corinthians", "1 Corintios"),
    ("2CO", "2 Corinthians", "2 Corintios"),
    ("GAL", "Galatians", "Gálatas"),
    ("EPH", "Ephesians", "Efesios"),
    ("PHP", "Philippians", "Filipenses"),
    ("COL", "Colossians", "Colosenses"),
    ("1TH", "1 Thessalonians", "1 Tesalonicenses"),
    ("2TH", "2 Thessalonians", "2 Tesalonicenses"),
    ("1TI", "1 Timothy", "1 Timoteo"),
    ("2TI", "2 Timothy", "2 Timoteo"),
    ("TIT", "Titus", "Tito"),
    ("PHM", "Philemon", "Filemón"),
    ("HEB", "Hebrews", "Hebreos"),
    ("JAS", "James", "Santiago"),
    ("1PE", "1 Peter", "1 Pedro"),
    ("2PE", "2 Peter", "2 Pedro"),
    ("1JN", "1 John", "1 Juan"),
    ("2JN", "2 John", "2 Juan"),
    ("3JN", "3 John", "3 Juan"),
    ("JUD", "Jude", "Judas"),
    ("REV", "Revelation", "Apocalipsis"),
];
fn normalized(s: &str) -> String {
    s.to_lowercase()
        .replace(['á', 'à'], "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace('ú', "u")
        .replace([' ', '.'], "")
}
pub fn parse_passage(input: &str) -> Result<Passage, String> {
    let input = input.trim().replace('–', "-");
    let space = input
        .rfind(' ')
        .ok_or("Use a reference such as John 3:16-18 or Juan 3-4")?;
    let name = normalized(&input[..space]);
    let exact = BOOKS
        .iter()
        .find(|(code, en, es)| [*code, *en, *es].iter().any(|s| normalized(s) == name));
    let matches: Vec<_> = BOOKS
        .iter()
        .filter(|(_, en, es)| [*en, *es].iter().any(|s| normalized(s).starts_with(&name)))
        .collect();
    let book = if let Some(b) = exact {
        b.0
    } else if name.len() >= 2 && matches.len() == 1 {
        matches[0].0
    } else {
        return Err("Unknown or ambiguous Bible book".into());
    };
    fn part(s: &str) -> Result<(u32, Option<u32>), String> {
        let mut p = s.split(':');
        let ch = p
            .next()
            .unwrap_or("")
            .parse::<u32>()
            .map_err(|_| "Invalid chapter")?;
        let vs = p
            .next()
            .map(|v| v.parse::<u32>().map_err(|_| "Invalid verse"))
            .transpose()?;
        if ch == 0 || ch > 150 || vs == Some(0) || vs.is_some_and(|v| v > 200) || p.next().is_some()
        {
            return Err("Invalid reference".into());
        }
        Ok((ch, vs))
    }
    let mut range = input[space + 1..].split('-');
    let (start_chapter, verse) = part(range.next().unwrap_or(""))?;
    let (end_chapter, end_verse) = if let Some(end) = range.next() {
        if verse.is_some() && !end.contains(':') {
            (
                start_chapter,
                end.parse::<u32>().map_err(|_| "Invalid ending verse")?,
            )
        } else {
            let (c, v) = part(end)?;
            (c, v.unwrap_or(200))
        }
    } else {
        (start_chapter, verse.unwrap_or(200))
    };
    let start_verse = verse.unwrap_or(1);
    if range.next().is_some()
        || end_verse == 0
        || end_verse > 200
        || (end_chapter, end_verse) < (start_chapter, start_verse)
    {
        return Err("Invalid passage range".into());
    }
    Ok(Passage {
        book: book.into(),
        start_chapter,
        start_verse,
        end_chapter,
        end_verse,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ai_line_limits_preserve_source_coverage_and_sections() {
        let text = "[Verse]\nUno\nDos\nTres\nCuatro\nCinco\n[Chorus]\nSanto\nSanto";
        let groups = vec![
            AnalysisGroup {
                section: "Verse".into(),
                start: 0,
                end: 5,
            },
            AnalysisGroup {
                section: "Chorus".into(),
                start: 5,
                end: 7,
            },
        ];
        for limit in 1..=8 {
            let blocks = apply_analysis_with_lines(text, &groups, limit).unwrap();
            assert!(blocks.iter().all(|b| b.primary.lines().count() <= limit));
            assert_eq!(
                blocks
                    .iter()
                    .map(|b| b.primary.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
                source_lines(text).join("\n")
            );
            assert_eq!(blocks.last().unwrap().section, "Chorus");
            assert_eq!(
                blocks
                    .iter()
                    .map(|b| &b.id)
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                blocks.len()
            );
        }
        assert!(apply_analysis_with_lines(text, &groups, 0).is_err());
        assert!(apply_analysis_with_lines(text, &groups, 9).is_err());
        assert!(apply_analysis_with_lines(text, &groups[1..], 3).is_err());
    }
    #[test]
    fn spanish_and_cross_chapter_ranges() {
        assert_eq!(parse_passage("Génesis 1:31-2:3").unwrap().end_chapter, 2);
        assert_eq!(parse_passage("Juan 3-4").unwrap().book, "JHN");
        assert_eq!(parse_passage("1 Cor 13:4-7").unwrap().end_verse, 7);
        assert!(parse_passage("John 3:18-16").is_err());
        assert!(parse_passage("Jo 3").is_err());
    }
    #[test]
    fn analysis_cannot_omit_repeat_or_rewrite() {
        let text = "Santo\nes el Señor\nHoly\nis the Lord";
        let good = vec![
            AnalysisGroup {
                section: "Coro".into(),
                start: 0,
                end: 2,
            },
            AnalysisGroup {
                section: "Chorus".into(),
                start: 2,
                end: 4,
            },
        ];
        assert_eq!(
            apply_analysis(text, &good).unwrap()[0].primary,
            "Santo\nes el Señor"
        );
        assert!(apply_analysis(text, &good[..1]).is_err());
        assert!(apply_analysis(
            text,
            &[AnalysisGroup {
                section: "x".into(),
                start: 1,
                end: 4
            }]
        )
        .is_err());
    }
    #[test]
    fn section_headers_remain_metadata() {
        let b = apply_analysis(
            "[Coro]\nÉl es fiel\nSanto es",
            &[AnalysisGroup {
                section: "Coro".into(),
                start: 0,
                end: 2,
            }],
        )
        .unwrap();
        assert_eq!(b[0].primary, "Él es fiel\nSanto es");
    }
    #[test]
    fn labels_and_accents_survive_split() {
        let b = split_lyrics(
            "[Coro]\nGloria a Dios\nÉl es fiel\n\n[Verse 2]\nOne\nTwo\nThree",
            2,
        );
        assert_eq!(b.len(), 3);
        assert_eq!(b[0].section, "Coro");
        assert!(b[0].primary.contains("Él"));
    }
}
