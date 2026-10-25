//! The adapters of the dialog of the multiple rename.
//!
//! WHAT: the engine that forms the new names ([`OpsRenameEngine`]) and the
//!       store of the named templates ([`FileTemplateStore`]).
//! WHY: the interface crate may not depend on the crate of the operations,
//!       and it performs no file operation of its own. The adapters are
//!       therefore here, in the composition root — the one crate that may see
//!       the crate of the operations, the crate of the platform and the
//!       crate of the interface at the same time (ADR-0001, option B). The
//!       engine answers the question of the dialog with the names that the
//!       engine of the operations forms, so that the preview the user sees
//!       is the preview the rename will carry out (FR-34).
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::filesystem::HostFileSystem;
//! use phocommander_desktop::rename::OpsRenameEngine;
//! use phocommander_ui::rename::{RenameEngine, RenameRequest};
//!
//! let engine = OpsRenameEngine::new(Box::new(HostFileSystem::new()));
//! let preview = engine.preview(&RenameRequest::new());
//! assert!(preview.rows.is_empty());
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use phocommander_ops::port::FileSystem;
use phocommander_ops::rename::{RenamePair as OpsPair, plan_renames};
use phocommander_ops::template::{
    CounterOrder as OpsCounterOrder, CounterRule as OpsCounterRule, RenameInput, RenameRule,
    ReplaceRule as OpsReplaceRule, SeriesRule as OpsSeriesRule, TemplateError, form_names,
};
use phocommander_ui::rename::{
    Conflict, ConflictKind, CounterOrder as UiCounterOrder, NamedTemplate, PreviewProblem,
    PreviewRow, ProblemKind, RenameEngine, RenameFile, RenamePreview, RenameRequest,
    RenameRule as UiRule, TemplateStore, TemplateStoreError, decode_templates, encode_templates,
};

/// The engine of the desktop build: it forms the new names of a rename.
///
/// WHY: the engine is the adapter between the question of the dialog and the
///       engine of the operations. The dialog asks for the new names and for
///       the conflicts of the rename, and the adapter answers both out of the
///       one place that owns the rules.
// WHY: the derive of Debug cannot be used here: the file system is a trait
//       object, and the trait does not demand Debug. The field that carries
//       the state is written by hand instead.
pub struct OpsRenameEngine {
    /// The file system the conflicts are looked for in.
    fs: Box<dyn FileSystem>,
}

impl std::fmt::Debug for OpsRenameEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpsRenameEngine")
            .finish_non_exhaustive()
    }
}

impl OpsRenameEngine {
    /// Builds the engine of the desktop build.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_desktop::filesystem::HostFileSystem;
    /// use phocommander_desktop::rename::OpsRenameEngine;
    ///
    /// let engine = OpsRenameEngine::new(Box::new(HostFileSystem::new()));
    /// let _ = engine;
    /// ```
    pub fn new(fs: Box<dyn FileSystem>) -> Self {
        Self { fs }
    }
}

impl RenameEngine for OpsRenameEngine {
    fn preview(&self, request: &RenameRequest) -> RenamePreview {
        let rule = rule_of(&request.rule);
        let inputs: Vec<RenameInput> = request.files.iter().map(input_of).collect();

        // WHY: the time of the rename comes from the caller, so that the same
        //       rule and the same files always yield the same names
        //       (FR-33). The preview asks for the moment of now.
        let names = match form_names(&rule, &inputs, SystemTime::now()) {
            Ok(names) => names,
            Err(error) => {
                return RenamePreview {
                    rows: Vec::new(),
                    conflicts: Vec::new(),
                    problem: Some(problem_of(&error)),
                };
            }
        };

        let mut rows: Vec<PreviewRow> = Vec::with_capacity(names.len());
        for name in &names {
            rows.push(match name.name() {
                Some(new_name) => PreviewRow::named(name.source(), new_name),
                None => {
                    let problem = match name.problem() {
                        Some(problem) => problem_of(problem),
                        None => PreviewProblem::empty_format(),
                    };
                    PreviewRow::failed(name.source(), problem)
                }
            });
        }

        let pairs = pairs_of(&rows);
        let conflicts = plan_renames(self.fs.as_ref(), &pairs)
            .into_iter()
            .map(|conflict| Conflict::new(kind_of(conflict.kind), conflict.path))
            .collect();

        RenamePreview {
            rows,
            conflicts,
            problem: None,
        }
    }
}

/// The store of the named templates in a file.
///
/// WHY: the templates of the user live in a file of the configuration
///       directory, so that they are there after a restart of the program.
///       The file is written in the format of the interface crate, and the
///       composition root only moves the bytes.
#[derive(Debug, Clone)]
pub struct FileTemplateStore {
    /// The path of the file of the templates.
    path: PathBuf,
}

impl FileTemplateStore {
    /// Builds the store of the templates in the given file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_desktop::rename::FileTemplateStore;
    ///
    /// let store = FileTemplateStore::new("/tmp/rename-templates.txt");
    /// assert!(store.path().ends_with("rename-templates.txt"));
    /// ```
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The path of the file of the templates.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl TemplateStore for FileTemplateStore {
    fn templates(&self) -> Result<Vec<NamedTemplate>, TemplateStoreError> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            // WHY: a store that does not exist yet holds no templates. That
            //       is not an error — the user has simply not stored one.
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(TemplateStoreError::Io {
                    path: self.path.clone(),
                    message: error.to_string(),
                });
            }
        };
        decode_templates(&text)
    }

    fn store(&self, template: &NamedTemplate) -> Result<(), TemplateStoreError> {
        let mut templates = self.templates()?;
        match templates
            .iter()
            .position(|other| other.name == template.name)
        {
            Some(index) => templates[index] = template.clone(),
            None => templates.push(template.clone()),
        }
        let text = encode_templates(&templates);

        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).map_err(|error| TemplateStoreError::Io {
                path: parent.to_path_buf(),
                message: error.to_string(),
            })?;
        }
        fs::write(&self.path, text).map_err(|error| TemplateStoreError::Io {
            path: self.path.clone(),
            message: error.to_string(),
        })?;
        Ok(())
    }
}

/// The rule of the dialog as the rule of the engine.
fn rule_of(rule: &UiRule) -> RenameRule {
    let mut ops = RenameRule::new(rule.effective_format());
    for replacement in &rule.replacements {
        ops = ops.with_replacement(OpsReplaceRule {
            search: replacement.search.clone(),
            replacement: replacement.replacement.clone(),
            regex: replacement.regex,
        });
    }
    if let Some(counter) = &rule.counter {
        ops = ops.with_counter(OpsCounterRule {
            start: counter.start,
            step: counter.step,
            width: counter.width,
            order: order_of(counter.order),
        });
    }
    if let Some(series) = &rule.series {
        ops = ops.with_series(OpsSeriesRule {
            series_name: series.series_name.clone(),
            season: series.season,
            episode: series.episode,
            format: series.format.clone(),
            season_width: series.season_width,
            episode_width: series.episode_width,
        });
    }
    ops
}

/// The order of the counter of the dialog as the order of the engine.
fn order_of(order: UiCounterOrder) -> OpsCounterOrder {
    match order {
        UiCounterOrder::Selection => OpsCounterOrder::Selection,
        UiCounterOrder::Name => OpsCounterOrder::Name,
        UiCounterOrder::Date => OpsCounterOrder::Date,
    }
}

/// The facts of a file of the dialog as the facts of the engine.
fn input_of(file: &RenameFile) -> RenameInput {
    let mut input = RenameInput::new(file.path.clone());
    input.size = file.size;
    input.modified = file.modified;
    input.checksum = file.checksum;
    input
}

/// The pairs of the rename, without the files that keep their name.
///
/// WHY: a file that keeps its name is not renamed. Such a pair would be a
///       rename of a file onto itself, and the engine refuses it.
fn pairs_of(rows: &[PreviewRow]) -> Vec<OpsPair> {
    let mut pairs = Vec::with_capacity(rows.len());
    for row in rows {
        let Some(target) = row.target.as_ref() else {
            continue;
        };
        if *target == row.source {
            continue;
        }
        pairs.push(OpsPair::new(row.source.clone(), target.clone()));
    }
    pairs
}

/// The reason of the engine as the reason of the dialog.
fn problem_of(error: &TemplateError) -> PreviewProblem {
    match error {
        TemplateError::EmptyFormat => PreviewProblem::empty_format(),
        TemplateError::UnknownValue { placeholder, .. } => {
            PreviewProblem::of(ProblemKind::UnknownValue, *placeholder)
        }
        TemplateError::InvalidPattern { pattern, .. } => {
            PreviewProblem::of(ProblemKind::InvalidPattern, pattern)
        }
        TemplateError::InvalidName { name, .. } => {
            PreviewProblem::of(ProblemKind::InvalidName, name)
        }
    }
}

/// The kind of a conflict of the engine as the kind of the dialog.
fn kind_of(kind: phocommander_ops::rename::RenameConflictKind) -> ConflictKind {
    match kind {
        phocommander_ops::rename::RenameConflictKind::TargetExists => ConflictKind::TargetExists,
        phocommander_ops::rename::RenameConflictKind::DuplicateTarget => {
            ConflictKind::DuplicateTarget
        }
        phocommander_ops::rename::RenameConflictKind::MissingSource => ConflictKind::MissingSource,
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use phocommander_ops::fake_fs::FakeFileSystem;

    /// The engine of the test with the given files.
    fn engine_with(files: &[(&str, &[u8])]) -> OpsRenameEngine {
        let fs = FakeFileSystem::new();
        for (path, content) in files {
            fs.write_file(Path::new(path), content)
                .unwrap_or_else(|error| panic!("the file should have been written: {error}"));
        }
        OpsRenameEngine::new(Box::new(fs))
    }

    /// The engine of the test with two files.
    fn engine() -> OpsRenameEngine {
        engine_with(&[("/photos/a.jpg", b"a"), ("/photos/b.jpg", b"b")])
    }

    /// The question of the test.
    fn request() -> RenameRequest {
        RenameRequest::with_files(
            UiRule::new("[name]_[n].[ext]"),
            vec![
                RenameFile::new("/photos/a.jpg"),
                RenameFile::new("/photos/b.jpg"),
            ],
        )
    }

    #[test]
    fn the_preview_is_what_the_engine_forms() {
        // arrange
        let engine = engine();

        // act
        let preview = engine.preview(&request());

        // assert — the names are the names that the engine of the operations
        // forms for the same rule and the same files.
        let names = form_names(
            &rule_of(&request().rule),
            &[
                input_of(&RenameFile::new("/photos/a.jpg")),
                input_of(&RenameFile::new("/photos/b.jpg")),
            ],
            SystemTime::now(),
        )
        .unwrap_or_default();
        assert_eq!(preview.rows.len(), names.len());
        assert_eq!(preview.rows[0].name.as_deref(), names[0].name());
        assert_eq!(preview.rows[1].name.as_deref(), names[1].name());
    }

    #[test]
    fn a_target_that_exists_is_a_conflict() {
        // arrange
        let engine = engine_with(&[
            ("/photos/a.jpg", b"a"),
            ("/photos/b.jpg", b"b"),
            ("/photos/a_001.jpg", b"taken"),
        ]);

        // act
        let preview = engine.preview(&request());

        // assert
        assert_eq!(preview.conflicts.len(), 1);
        assert_eq!(preview.conflicts[0].kind, ConflictKind::TargetExists);
    }

    #[test]
    fn two_files_that_want_the_same_name_are_a_conflict() {
        // arrange
        let engine = engine();
        let question = RenameRequest::with_files(
            UiRule::new("fixed.[ext]"),
            vec![
                RenameFile::new("/photos/a.jpg"),
                RenameFile::new("/photos/b.jpg"),
            ],
        );

        // act — the rule names every file "fixed.jpg".
        let preview = engine.preview(&question);

        // assert
        assert_eq!(preview.conflicts.len(), 1);
        assert_eq!(preview.conflicts[0].kind, ConflictKind::DuplicateTarget);
    }

    #[test]
    fn a_rule_without_a_format_is_a_problem_of_the_whole_rule() {
        // arrange
        let engine = engine();
        let question =
            RenameRequest::with_files(UiRule::new(""), vec![RenameFile::new("/photos/a.jpg")]);

        // act
        let preview = engine.preview(&question);

        // assert
        assert!(preview.problem.is_some());
        assert_eq!(
            preview.problem.map(|problem| problem.kind),
            Some(ProblemKind::EmptyFormat)
        );
    }

    #[test]
    fn a_stored_template_is_read_back() {
        // arrange
        let directory = std::env::temp_dir().join("phocommander-template-test");
        let path = directory.join("templates.txt");
        let _ = fs::remove_file(&path);
        let store = FileTemplateStore::new(path.clone());
        let template = NamedTemplate::new("Camera", UiRule::new("[name]_[n].[ext]"));

        // act
        store
            .store(&template)
            .unwrap_or_else(|error| panic!("the template should have been stored: {error}"));
        let read = store
            .templates()
            .unwrap_or_else(|error| panic!("the templates should have been read: {error}"));

        // assert
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].name, "Camera");
        assert_eq!(read[0].rule.format, "[name]_[n].[ext]");
        let _ = fs::remove_file(&path);
    }
}
