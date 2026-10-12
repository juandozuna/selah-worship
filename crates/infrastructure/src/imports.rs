use quick_xml::{events::Event, Reader};
use regex::Regex;
use selah_domain::{BibleVerse, PresentationBlock};

pub fn openlyrics(text: &str) -> Result<Vec<PresentationBlock>, String> {
    let mut reader = Reader::from_str(text);
    let mut section = String::new();
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut in_lines = false;
    let mut blocks = Vec::new();
    loop {
        match reader.read_event().map_err(|e| e.to_string())? {
            Event::Start(e) if e.local_name().as_ref() == b"verse" => {
                section = e
                    .attributes()
                    .filter_map(Result::ok)
                    .find(|a| a.key.as_ref() == b"name")
                    .and_then(|a| String::from_utf8(a.value.into_owned()).ok())
                    .unwrap_or("Verse".into());
            }
            Event::Start(e) if e.local_name().as_ref() == b"lines" => {
                in_lines = true;
                line.clear();
            }
            Event::Empty(e) if e.local_name().as_ref() == b"br" && in_lines => {
                lines.push(std::mem::take(&mut line));
            }
            Event::Text(e) if in_lines => line.push_str(&e.unescape().map_err(|e| e.to_string())?),
            Event::End(e) if e.local_name().as_ref() == b"lines" => {
                if !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                }
                for chunk in lines.chunks(2) {
                    blocks.push(PresentationBlock {
                        motion: None,
                        id: format!("block-{}", blocks.len()),
                        section: section.clone(),
                        primary: chunk.join("\n"),
                        secondary: None,
                        reference: None,
                    });
                }
                lines.clear();
                in_lines = false;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if blocks.is_empty() {
        Err("No OpenLyrics verses found".into())
    } else {
        Ok(blocks)
    }
}

pub fn usfm(text: &str, translation: &str) -> Result<Vec<BibleVerse>, String> {
    let marker = Regex::new(r"\\(id|c|v)\s+([^\s\\]+)").unwrap();
    let notes = Regex::new(r"(?s)\\(?:f|x)\s.*?\\(?:f|x)\*").unwrap();
    let text = notes.replace_all(text, "");
    let headings = Regex::new(r"(?m)^\\(?:h|toc[1-3]|mt[0-9]*|s[0-9]*|r|d)\s+[^\n]*").unwrap();
    let text = headings.replace_all(&text, "");
    let words = Regex::new(r"\\w\s+([^|\\]+)(?:\|[^\\]*)?\\w\*").unwrap();
    let text = words.replace_all(&text, "$1");
    let clean = Regex::new(r"\\[a-z0-9]+\*?\s?").unwrap();
    let captures: Vec<_> = marker.captures_iter(&text).collect();
    let mut book = String::new();
    let mut chapter = 0;
    let mut verses = Vec::new();
    for (i, c) in captures.iter().enumerate() {
        let value = c[2].trim();
        match &c[1] {
            "id" => book = value.into(),
            "c" => chapter = value.parse().map_err(|_| "Invalid USFM chapter")?,
            "v" => {
                let verse = value.parse::<u32>().map_err(|_| {
                    "Verse bridges need to be separated before importing (for example 1-2)"
                })?;
                let start = c.get(0).unwrap().end();
                let end = captures
                    .get(i + 1)
                    .map(|next| next.get(0).unwrap().start())
                    .unwrap_or(text.len());
                let raw = clean.replace_all(&text[start..end], "");
                let body = raw.split_whitespace().collect::<Vec<_>>().join(" ");
                if book.is_empty() || chapter == 0 || body.is_empty() {
                    return Err("Invalid USFM verse".into());
                }
                verses.push(BibleVerse {
                    translation: translation.into(),
                    book: book.clone(),
                    chapter,
                    verse,
                    text: body,
                });
            }
            _ => {}
        }
    }
    if verses.is_empty() {
        Err("No USFM verses found".into())
    } else {
        Ok(verses)
    }
}

pub fn vpl(text: &str, translation: &str) -> Result<Vec<BibleVerse>, String> {
    let re = Regex::new(r"^([A-Za-z0-9]+)\s+(\d+):(\d+)\s+(.+)$").unwrap();
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(c) = re.captures(line.trim_start_matches('\u{feff}').trim()) {
            let alias = match c[1].to_ascii_lowercase().as_str() {
                "sol" => "SNG",
                "eze" => "EZK",
                "joe" => "JOL",
                "nah" => "NAM",
                "phi" => "PHP",
                "jam" => "JAS",
                "1jo" => "1JN",
                "2jo" => "2JN",
                "3jo" => "3JN",
                _ => &c[1],
            };
            let book = selah_application::parse_passage(&format!("{alias} 1"))?.book;
            out.push(BibleVerse {
                translation: translation.into(),
                book,
                chapter: c[2].parse().map_err(|_| "Invalid chapter")?,
                verse: c[3].parse().map_err(|_| "Invalid verse")?,
                text: c[4].into(),
            });
        }
    }
    if out.is_empty() {
        Err("No verse-per-line records found".into())
    } else {
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xml_preserves_entities_and_breaks() {
        let b=openlyrics("<song><lyrics><verse name='c'><lines>Santo &amp; fiel<br/>Holy Lord</lines></verse></lyrics></song>").unwrap();
        assert_eq!(b[0].primary, "Santo & fiel\nHoly Lord");
    }
    #[test]
    fn usfm_excludes_notes() {
        let b = usfm(
            "\\id JHN\n\\c 3\n\\v 16 For God \\f + \\ft a footnote\\f* loved.\n\\v 17 Next verse.",
            "WEB",
        )
        .unwrap();
        assert_eq!(b.len(), 2);
        assert!(!b[0].text.contains("footnote"));
        assert_eq!(b[0].text, "For God loved.");
    }
    #[test]
    fn usfm_keeps_inline_words_and_poetry() {
        let b=usfm("\\id PSA\n\\c 1\n\\v 1 Blessed \\w man|lemma=man\\w*\n\\q1 who walks.\n\\s A heading\n\\v 2 Next.","WEB").unwrap();
        assert_eq!(b[0].text, "Blessed man who walks.");
    }
    #[test]
    fn bibleworks_aliases_map_to_canonical_books() {
        let verses = vpl("Joh 3:16 For God loved.\nPhi 1:1 Paul\nSol 1:1 Song", "WEB").unwrap();
        assert_eq!(
            verses.iter().map(|v| v.book.as_str()).collect::<Vec<_>>(),
            vec!["JHN", "PHP", "SNG"]
        );
    }
}
