mod id;

pub use id::*;

use std::{
    borrow::Cow,
    error::Error,
    ffi::OsString,
    fmt::Display,
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use fluent::*;
use fluent_syntax::parser::ParserError;
use unic_langid::{LanguageIdentifier, LanguageIdentifierError};

pub struct TextRepository {
    bundle: FluentBundle<FluentResource>,
}

#[derive(Debug)]
pub enum LoadTranslationError {
    ResourceRegistration(Vec<FluentError>),
    Io(io::Error),
    Parsing(FluentResource, Vec<ParserError>),
}

#[derive(Debug)]
pub enum TextRepositoryCreationError {
    ResourceRegistration(Vec<FluentError>),
    BadTranslationDirectoryName(OsString),
    InvalidTranslationDirectoryName(LanguageIdentifierError),
    Io(io::Error),
    Parsing(FluentResource, Vec<ParserError>),
    ZeroTranslationDirectories,
}

#[derive(Debug)]
pub enum TextRetrievalError {
    EmptyMessage, // ?
    Formatting(Vec<FluentError>),
    NotFound(String),
}

fn calc_langid_similarity(x: &LanguageIdentifier, y: &LanguageIdentifier) -> Option<i32> {
    // todo: employ subtag similarity
    if x.language != y.language {
        return None;
    }
    let mut score = 0;
    match (x.script, y.script) {
        (None, None) => score += 1,
        (None, Some(_)) => score -= 1,
        (Some(_), None) => score -= 1,
        (Some(xs), Some(ys)) => score += if xs == ys { 2 } else { -6 },
    }
    match (x.region, y.region) {
        (None, None) => score += 1,
        (None, Some(_)) => score -= 2,
        (Some(_), None) => score -= 2,
        (Some(xr), Some(yr)) => score += if xr == yr { 4 } else { -3 },
    }
    for &variant in x.variants() {
        if y.has_variant(variant) {
            score += 2;
        } else {
            score -= 1;
        }
    }
    for &variant in y.variants() {
        if !x.has_variant(variant) {
            score -= 1;
        }
    }
    Some(score)
}

fn load_directory(
    bundle: &mut FluentBundle<FluentResource>,
    path: PathBuf,
) -> Result<(), LoadTranslationError> {
    let dir_iter = std::fs::read_dir(path).map_err(LoadTranslationError::Io)?;
    for dir_entry in dir_iter {
        let dir_entry = dir_entry.map_err(LoadTranslationError::Io)?;
        let entry_path = dir_entry.path();
        if entry_path.is_file() {
            let source = std::fs::read_to_string(entry_path).map_err(LoadTranslationError::Io)?;
            let resource = FluentResource::try_new(source)
                .map_err(|(r, e)| LoadTranslationError::Parsing(r, e))?;
            bundle
                .add_resource(resource)
                .map_err(LoadTranslationError::ResourceRegistration)?;
        } else if entry_path.is_dir() {
            load_directory(bundle, entry_path)?;
        }
    }
    Ok(())
}

impl TextRepository {
    pub fn new() -> Self {
        TextRepository {
            bundle: FluentBundle::new(vec![unic_langid::langid!("en")]),
        }
    }

    pub fn from_directory(path: &Path) -> Result<Self, TextRepositoryCreationError> {
        let default_langid = unic_langid::langid!("en");
        let mut dir_iter = std::fs::read_dir(path).map_err(TextRepositoryCreationError::Io)?;
        let mut max_similar_path = match dir_iter.next() {
            Some(Ok(dir_entry)) => dir_entry.path(),
            Some(Err(e)) => return Err(TextRepositoryCreationError::Io(e)),
            None => return Err(TextRepositoryCreationError::ZeroTranslationDirectories),
        };
        let mut max_similarity = {
            let dir_name = max_similar_path.file_name().unwrap();
            let dir_name = dir_name.to_str().ok_or_else(|| {
                TextRepositoryCreationError::BadTranslationDirectoryName(dir_name.to_owned())
            })?;
            let entry_langid = LanguageIdentifier::from_str(dir_name)
                .map_err(TextRepositoryCreationError::InvalidTranslationDirectoryName)?;
            calc_langid_similarity(&default_langid, &entry_langid)
        };
        for dir_entry in dir_iter {
            let dir_entry = dir_entry.map_err(TextRepositoryCreationError::Io)?;
            let entry_path = dir_entry.path();
            if !entry_path.is_dir() {
                continue;
            }
            let entry_name = dir_entry.file_name();
            let entry_name = entry_name.to_str().ok_or_else(|| {
                TextRepositoryCreationError::BadTranslationDirectoryName(entry_name.to_owned())
            })?;
            let entry_langid = LanguageIdentifier::from_str(entry_name)
                .map_err(TextRepositoryCreationError::InvalidTranslationDirectoryName)?;
            let entry_similarity = calc_langid_similarity(&default_langid, &entry_langid);
            if entry_similarity > max_similarity {
                max_similarity = entry_similarity;
                max_similar_path = dir_entry.path();
            }
        }
        let mut bundle = FluentBundle::new(vec![default_langid]);
        load_directory(&mut bundle, max_similar_path).map_err(|e| match e {
            LoadTranslationError::ResourceRegistration(es) => {
                TextRepositoryCreationError::ResourceRegistration(es)
            }
            LoadTranslationError::Io(e) => TextRepositoryCreationError::Io(e),
            LoadTranslationError::Parsing(r, es) => TextRepositoryCreationError::Parsing(r, es),
        })?;
        Ok(TextRepository { bundle })
    }

    pub fn switch_translation(
        &mut self,
        langid: LanguageIdentifier,
        path: PathBuf,
    ) -> Result<(), LoadTranslationError> {
        self.bundle = FluentBundle::new(vec![langid]);
        load_directory(&mut self.bundle, path)
    }

    /// Lists subdirectories' names parsed as language identifiers along with their full paths.
    pub fn available_translations(
        dir: &Path,
    ) -> anyhow::Result<Vec<(LanguageIdentifier, PathBuf)>> {
        let mut translations = Vec::new();
        for dir_entry in std::fs::read_dir(dir)? {
            let dir_entry = dir_entry.map_err(TextRepositoryCreationError::Io)?;
            let entry_path = dir_entry.path();
            if !entry_path.is_dir() {
                continue;
            }
            let entry_name = dir_entry.file_name();
            let entry_name = entry_name.to_str().ok_or_else(|| {
                TextRepositoryCreationError::BadTranslationDirectoryName(entry_name.to_owned())
            })?;
            let entry_langid = LanguageIdentifier::from_str(entry_name)
                .map_err(TextRepositoryCreationError::InvalidTranslationDirectoryName)?;
            translations.push((entry_langid, entry_path));
        }
        Ok(translations)
    }

    pub fn get<'a>(
        &'a self,
        id: &TextIdRef,
        args: Option<&'a FluentArgs<'_>>,
    ) -> Result<Cow<'a, str>, TextRetrievalError> {
        let msg = self
            .bundle
            .get_message(&id.0)
            .ok_or_else(|| TextRetrievalError::NotFound(id.report()))?;
        let mut errors = Vec::new();
        let cow = self.bundle.format_pattern(
            msg.value().ok_or(TextRetrievalError::EmptyMessage)?,
            args,
            &mut errors,
        );
        if !errors.is_empty() {
            return Err(TextRetrievalError::Formatting(errors));
        }
        Ok(cow)
    }
}

impl Display for LoadTranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadTranslationError::ResourceRegistration(es) => {
                write!(f, "Resource registration failed with ")?;
                crate::util::display_each(f, es.iter(), "; ", ".")
            }
            LoadTranslationError::Io(e) => e.fmt(f),
            LoadTranslationError::Parsing(r, es) => {
                write!(f, "Parsing of {} failed with ", r.source())?;
                crate::util::display_each(f, es.iter(), "; ", ".")
            }
        }
    }
}

impl Error for LoadTranslationError {}

impl Display for TextRepositoryCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextRepositoryCreationError::ResourceRegistration(es) => {
                write!(f, "Resource registration failed with ")?;
                crate::util::display_each(f, es.iter(), "; ", ".")
            }
            TextRepositoryCreationError::BadTranslationDirectoryName(name) => {
                write!(
                    f,
                    "Translation directory name can't be converted to UTF-8: \"{}\".",
                    name.to_string_lossy()
                )
            }
            TextRepositoryCreationError::InvalidTranslationDirectoryName(e) => {
                write!(f, "Invalid translation directory name: {e}")
            }
            TextRepositoryCreationError::Io(e) => e.fmt(f),
            TextRepositoryCreationError::Parsing(r, es) => {
                write!(f, "Parsing of {} failed with ", r.source())?;
                crate::util::display_each(f, es.iter(), "; ", ".")
            }
            TextRepositoryCreationError::ZeroTranslationDirectories => {
                write!(f, "Zero translation directories.")
            }
        }
    }
}

impl Error for TextRepositoryCreationError {}

impl Display for TextRetrievalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextRetrievalError::EmptyMessage => write!(f, "Message is empty."),
            TextRetrievalError::Formatting(es) => {
                write!(f, "Formatting message failed with ")?;
                crate::util::display_each(f, es.iter(), "; ", ".")
            }
            TextRetrievalError::NotFound(id) => write!(f, "Message \"{id}\" not found."),
        }
    }
}

impl Error for TextRetrievalError {}
