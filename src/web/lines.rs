use std::{borrow::Cow, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Lines {
    pub start: usize,
    pub end: usize,
}

impl FromStr for Lines {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n = |s: &str| -> Result<usize, Cow<'static, str>> {
            if !s.starts_with('L') {
                return Err("line numbers should start with 'L'".into());
            }
            let s = &s[1..];

            let n = s.parse::<usize>().map_err(|err| err.to_string())?;
            Ok(n)
        };

        match s.split_once('-') {
            Some((start, end)) => {
                let start = n(start)?;
                let end = n(end)?;
                Ok(Self { start, end })
            }
            None => {
                let both = n(s)?;
                Ok(Self {
                    start: both,
                    end: both,
                })
            }
        }
    }
}

impl fmt::Display for Lines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "L{}", self.start)
        } else {
            write!(f, "L{}-L{}", self.start, self.end)
        }
    }
}

impl<'de> Deserialize<'de> for Lines {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for Lines {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
