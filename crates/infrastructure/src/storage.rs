use rusqlite::{params, Connection};
use selah_application::{BibleRepository, Passage, ProjectRepository};
use selah_domain::{BibleVerse, Project};
use std::path::Path;

pub struct SqliteStore {
    connection: Connection,
}
impl SqliteStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|e| e.to_string())?;
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if version > 1 {
            return Err("This database was created by a newer Selah version".into());
        }
        connection.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS project(id INTEGER PRIMARY KEY CHECK(id=1), data TEXT NOT NULL); CREATE TABLE IF NOT EXISTS verses(translation TEXT NOT NULL, book TEXT NOT NULL, chapter INTEGER NOT NULL, verse INTEGER NOT NULL, text TEXT NOT NULL, PRIMARY KEY(translation,book,chapter,verse)); PRAGMA user_version=1;").map_err(|e|e.to_string())?;
        Ok(Self { connection })
    }
    pub fn import_verses(&mut self, verses: &[BibleVerse]) -> Result<usize, String> {
        if verses.is_empty() {
            return Err("No verses found".into());
        }
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        for v in verses {
            if v.translation.is_empty()
                || v.book.is_empty()
                || v.chapter == 0
                || v.verse == 0
                || v.text.is_empty()
            {
                return Err("Invalid verse".into());
            }
            tx.execute(
                "INSERT OR REPLACE INTO verses VALUES(?1,?2,?3,?4,?5)",
                params![v.translation, v.book, v.chapter, v.verse, v.text],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(verses.len())
    }
}
impl ProjectRepository for SqliteStore {
    fn load(&self) -> Result<Option<Project>, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT data FROM project WHERE id=1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
        match rows.next().map_err(|e| e.to_string())? {
            Some(row) => {
                let json: String = row.get(0).map_err(|e| e.to_string())?;
                serde_json::from_str(&json)
                    .map(Some)
                    .map_err(|e| e.to_string())
            }
            None => Ok(None),
        }
    }
    fn save(&self, project: &Project) -> Result<(), String> {
        project.validate()?;
        let data = serde_json::to_string(project).map_err(|e| e.to_string())?;
        self.connection
            .execute(
                "INSERT INTO project VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
                [data],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
impl BibleRepository for SqliteStore {
    fn translations(&self) -> Result<Vec<String>, String> {
        self.connection
            .prepare("SELECT DISTINCT translation FROM verses ORDER BY translation")
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
    fn verses(&self, translation: &str, p: &Passage) -> Result<Vec<BibleVerse>, String> {
        let mut stmt=self.connection.prepare("SELECT chapter,verse,text FROM verses WHERE translation=?1 AND book=?2 AND (chapter>?3 OR (chapter=?3 AND verse>=?4)) AND (chapter<?5 OR (chapter=?5 AND verse<=?6)) ORDER BY chapter,verse").map_err(|e|e.to_string())?;
        let verses = stmt
            .query_map(
                params![
                    translation,
                    p.book,
                    p.start_chapter,
                    p.start_verse,
                    p.end_chapter,
                    p.end_verse
                ],
                |r| {
                    Ok(BibleVerse {
                        translation: translation.into(),
                        book: p.book.clone(),
                        chapter: r.get(0)?,
                        verse: r.get(1)?,
                        text: r.get(2)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if verses.is_empty() {
            return Err(
                "This passage is not installed. Import or download the translation first.".into(),
            );
        }
        Ok(verses)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verse_ranges_and_failed_import_are_atomic() {
        let mut s = SqliteStore::open(Path::new(":memory:")).unwrap();
        let v = |ch, vs, text: &str| BibleVerse {
            translation: "WEB".into(),
            book: "JHN".into(),
            chapter: ch,
            verse: vs,
            text: text.into(),
        };
        s.import_verses(&[v(3, 16, "one"), v(3, 17, "two"), v(4, 1, "three")])
            .unwrap();
        let p = selah_application::parse_passage("Juan 3:17-4:1").unwrap();
        assert_eq!(s.verses("WEB", &p).unwrap().len(), 2);
        assert!(s.import_verses(&[v(5, 1, "four"), v(5, 0, "bad")]).is_err());
        assert!(s
            .verses("WEB", &selah_application::parse_passage("John 5").unwrap())
            .is_err());
    }
}
