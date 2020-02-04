use std::io::prelude::*;
use std::io::{BufRead, BufReader};

use super::issue_tracker::Templates;
use crate::error::Error;
use crate::issue_tracker::Issue;
use crate::Result;

use handlebars::Handlebars;
use regex::Regex;
use unidiff::PatchSet;

lazy_static! {
    static ref RX_COMMENT: Regex =
        Regex::new(r"^\s*//\s{0,1}(?P<type>TODO|FIXME):\s*(?P<details>.+)$").unwrap();
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Comment {
    pub line: u64,
    pub file: String,
    pub details: String,
}

impl Comment {
    pub fn render(&self, template: &str) -> Result<String> {
        Handlebars::new()
            .render_template(template, self)
            .map_err(Error::from)
    }
}

#[derive(Debug, PartialEq)]
pub enum Annotation {
    FixMe(Comment),
    Todo(Comment),
    // ToDelete(Comment),
}

#[derive(Debug)]
struct PatchedFile {
    line: u64,
    filename: String,
    diff: String,
}

// TODO: use to parse entire file
pub fn parse<T>(input_stream: T, filename: String) -> Vec<Annotation>
where
    T: Read,
{
    let mut annotations = Vec::new();
    let input_stream = BufReader::new(input_stream);

    for (line_nb, line) in input_stream.lines().map(|l| l.unwrap()).enumerate() {
        let (comment_type, details) = match RX_COMMENT.captures(&line) {
            Some(matches) => (matches["type"].to_string(), matches["details"].to_string()),
            _ => continue,
        };

        let comment = Comment {
            line: line_nb as u64 + 1,
            details,
            file: filename.clone(),
        };

        let annotation = match &comment_type[..] {
            "FIXME" => Annotation::FixMe(comment),
            "TODO" => Annotation::Todo(comment),
            _ => unreachable!(),
        };

        annotations.push(annotation);
    }

    annotations
}

pub fn parse_git_diff(input: String, templates: &Templates) -> Result<Vec<Issue>> {
    let mut pachset = PatchSet::new();
    pachset.parse(input)?;

    let files = get_patched_or_added_files(pachset);

    let mut issues: Vec<Issue> = Vec::new();
    for file in files {
        let mut issues_to_create = parse_string(file.diff, file.filename, file.line, templates)?;
        issues.append(&mut issues_to_create);
    }

    Ok(issues)
}

pub fn parse_string(
    input: String,
    filename: String,
    offset: u64,
    templates: &Templates,
) -> Result<Vec<Issue>> {
    let mut annotations = Vec::new();

    for (line_nb, line) in input.lines().enumerate() {
        let (comment_type, details) = match RX_COMMENT.captures(&line) {
            Some(matches) => (matches["type"].to_string(), matches["details"].to_string()),
            _ => continue,
        };

        let comment = Comment {
            line: offset + line_nb as u64,
            details,
            file: filename.clone(),
        };

        let annotation = match &comment_type[..] {
            "FIXME" => Annotation::FixMe(comment),
            "TODO" => Annotation::Todo(comment),
            _ => unreachable!(),
        };

        annotations.push(annotation);
    }

    let mut issues = Vec::new();
    for annotation in annotations {
        issues.push(Issue::from(annotation, templates)?);
    }

    Ok(issues)
}

fn get_patched_or_added_files(patchset: PatchSet) -> Vec<PatchedFile> {
    let mut files: Vec<PatchedFile> = Vec::new();

    for patchset_files in [patchset.modified_files(), patchset.added_files()]
        .iter()
        .cloned()
    {
        for modified_file in patchset_files.into_iter() {
            let filename = modified_file.target_file.clone();

            for hunk in modified_file {
                let mut added_lines = String::new();
                let mut target_line_nb = None;

                for line in hunk {
                    if line.is_added() {
                        if target_line_nb.is_none() {
                            target_line_nb = line.target_line_no;
                        }
                        added_lines += &(line.value + "\n");
                    }
                }

                if let Some(target_line_nb) = target_line_nb {
                    files.push(PatchedFile {
                        line: target_line_nb as u64,
                        filename: filename[2..].to_owned(),
                        diff: added_lines.trim_end_matches('\n').to_owned(),
                    });
                }
            }
        }
    }

    files
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_parse() {
        let file = File::open("samples/sample.rs").expect("cannot open sample file");

        let resp = parse(file, String::from("samples/sample.rs"));

        assert_eq!(
            resp,
            vec![Annotation::Todo(Comment {
                line: 1,
                file: String::from("samples/sample.rs"),
                details: r"to delete, definitively because it's a non-sense".to_owned(),
            })]
        )
    }
}
