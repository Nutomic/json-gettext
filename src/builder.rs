use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};
use std::fs::File;
use std::io;
use std::path::Path;

use crate::serde::Serialize;
use crate::serde_json::{Error as JSONError, Map, Value};

use crate::{Context, JSONGetText, JSONGetTextValue};

#[derive(Debug)]
pub enum JSONGetTextBuildError {
    DefaultKeyNotFound,
    TextInKeyNotInDefaultKey {
        key: String,
        text: String,
    },
    DuplicatedKey(String),
    IOError(io::Error),
    SerdeJSONError(JSONError),
}

impl Display for JSONGetTextBuildError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            JSONGetTextBuildError::DefaultKeyNotFound => {
                f.write_str("The default key is not found.")
            }
            JSONGetTextBuildError::TextInKeyNotInDefaultKey {
                key,
                text,
            } => {
                f.write_fmt(format_args!(
                    "The text `{}` in the key `{}` is not found in the default key.",
                    text, key
                ))
            }
            JSONGetTextBuildError::DuplicatedKey(s) => f.write_str(s),
            JSONGetTextBuildError::IOError(err) => Display::fmt(err, f),
            JSONGetTextBuildError::SerdeJSONError(err) => Display::fmt(err, f),
        }
    }
}

impl Error for JSONGetTextBuildError {}

impl From<io::Error> for JSONGetTextBuildError {
    #[inline]
    fn from(v: io::Error) -> JSONGetTextBuildError {
        JSONGetTextBuildError::IOError(v)
    }
}

impl From<JSONError> for JSONGetTextBuildError {
    #[inline]
    fn from(v: JSONError) -> JSONGetTextBuildError {
        JSONGetTextBuildError::SerdeJSONError(v)
    }
}

/// To build a JSONGetText instance, this struct can help you do that step by step.
#[derive(Debug, Clone)]
pub struct JSONGetTextBuilder<'a> {
    default_key: String,
    context: Context<'a>,
}

impl<'a> JSONGetTextBuilder<'a> {
    /// Create a new `JSONGetTextBuilder` instance. You need to decide your default key at the stage.
    #[inline]
    pub fn new<S: Into<String>>(default_key: S) -> JSONGetTextBuilder<'a> {
        JSONGetTextBuilder {
            default_key: default_key.into(),
            context: HashMap::new(),
        }
    }

    /// Add a JSON string to the context for a specify key. The JSON string must represent a map object (key-value).
    pub fn add_json<K: AsRef<str> + Into<String>, J: AsRef<str> + ?Sized>(
        &mut self,
        key: K,
        json: &'a J,
    ) -> Result<&mut Self, JSONGetTextBuildError> {
        if self.context.contains_key(key.as_ref()) {
            return Err(JSONGetTextBuildError::DuplicatedKey(key.into()));
        }

        let map: HashMap<String, JSONGetTextValue<'a>> = serde_json::from_str(json.as_ref())?;

        let key = key.into();

        self.context.insert(key, map);

        Ok(self)
    }

    /// Add a JSON string to the context for a specify key. The JSON string must represent a map object (key-value).
    pub fn add_json_owned<K: AsRef<str> + Into<String>, J: AsRef<str>>(
        &mut self,
        key: K,
        json: J,
    ) -> Result<&mut Self, JSONGetTextBuildError> {
        if self.context.contains_key(key.as_ref()) {
            return Err(JSONGetTextBuildError::DuplicatedKey(key.into()));
        }

        let value: Map<String, Value> = serde_json::from_str(json.as_ref())?;

        let mut map: HashMap<String, JSONGetTextValue<'static>> =
            HashMap::with_capacity(value.len());

        for (k, v) in value {
            map.insert(k, JSONGetTextValue::from_json_value(v));
        }

        let key = key.into();

        self.context.insert(key, map);

        Ok(self)
    }

    /// Add a JSON file to the context for a specify key. The JSON file must represent a map object (key-value).
    pub fn add_json_file<K: AsRef<str> + Into<String>, P: AsRef<Path>>(
        &mut self,
        key: K,
        path: P,
    ) -> Result<&mut Self, JSONGetTextBuildError> {
        if self.context.contains_key(key.as_ref()) {
            return Err(JSONGetTextBuildError::DuplicatedKey(key.into()));
        }

        let path = path.as_ref();

        let value: Map<String, Value> = serde_json::from_reader(File::open(&path)?)?;

        let mut map: HashMap<String, JSONGetTextValue<'static>> =
            HashMap::with_capacity(value.len());

        for (k, v) in value {
            map.insert(k, JSONGetTextValue::from_json_value(v));
        }

        let key = key.into();

        self.context.insert(key, map);

        Ok(self)
    }

    /// Add any serializable value to the context for a specify key. The value must represent a map object (key-value).
    pub fn add_serialize<K: AsRef<str> + Into<String>, S: Serialize>(
        &mut self,
        key: K,
        value: S,
    ) -> Result<&mut Self, JSONGetTextBuildError> {
        if self.context.contains_key(key.as_ref()) {
            return Err(JSONGetTextBuildError::DuplicatedKey(key.into()));
        }

        let value: Value = serde_json::to_value(value)?;

        match value {
            Value::Object(value) => {
                let mut map: HashMap<String, JSONGetTextValue<'static>> =
                    HashMap::with_capacity(value.len());

                for (k, v) in value {
                    map.insert(k, JSONGetTextValue::from_json_value(v));
                }

                let key = key.into();

                self.context.insert(key, map);

                Ok(self)
            }
            _ => {
                serde_json::from_str::<Map<String, Value>>("\"MagicLen\"")?;

                unreachable!()
            }
        }
    }

    /// Add a map to the context.
    pub fn add_map<K: AsRef<str> + Into<String>>(
        &mut self,
        key: K,
        map: HashMap<String, JSONGetTextValue<'a>>,
    ) -> Result<&mut Self, JSONGetTextBuildError> {
        if self.context.contains_key(key.as_ref()) {
            return Err(JSONGetTextBuildError::DuplicatedKey(key.into()));
        }

        let key = key.into();

        self.context.insert(key, map);

        Ok(self)
    }

    /// Build a `JSONGetText` instance.
    pub fn build(self) -> Result<JSONGetText<'a>, JSONGetTextBuildError> {
        JSONGetText::from_context_with_default_key(self.default_key, self.context)
    }
}

impl<'a> From<String> for JSONGetTextBuilder<'a> {
    #[inline]
    fn from(v: String) -> JSONGetTextBuilder<'a> {
        JSONGetTextBuilder::new(v)
    }
}
