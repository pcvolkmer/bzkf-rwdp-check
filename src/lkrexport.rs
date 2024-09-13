/*
 * This file is part of bzkf-rwdp-check
 *
 * Copyright (C) 2024 the original author or authors.
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

use std::fs;
use std::path::Path;
use std::str::FromStr;

use itertools::Itertools;
use regex::Regex;

pub struct LkrExportProtocolFile {
    pub patients: Vec<Patient>,
}

impl LkrExportProtocolFile {
    pub fn parse_file(path: &Path) -> Result<LkrExportProtocolFile, ()> {
        let xml_file_content = fs::read_to_string(path).map_err(|_| ())?;
        Self::parse(&xml_file_content)
    }

    pub fn parse(content: &str) -> Result<LkrExportProtocolFile, ()> {
        let re = Regex::new(r"(?s)(?<patient><Patient>(.*?)</Patient>)").unwrap();

        if re.is_match(content) {
            let patients = re
                .find_iter(content)
                .map(|m| Patient {
                    raw_value: m.as_str().to_string(),
                })
                .collect_vec();
            return Ok(LkrExportProtocolFile { patients });
        }

        Err(())
    }

    pub fn meldungen(&self) -> Vec<Meldung> {
        self.patients
            .iter()
            .flat_map(|patient| patient.meldungen())
            .collect_vec()
    }
}

pub struct Patient {
    pub raw_value: String,
}

impl Patient {
    pub fn meldungen(&self) -> Vec<Meldung> {
        let re = Regex::new(r"(?s)(?<meldung><Meldung(.*?)</Meldung>)").unwrap();

        if re.is_match(&self.raw_value) {
            return re
                .find_iter(&self.raw_value)
                .map(|m| Meldung {
                    raw_value: m.as_str().to_string(),
                })
                .collect_vec();
        }
        vec![]
    }
}

pub struct Meldung {
    pub raw_value: String,
}

impl FromStr for Meldung {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Meldung {
            raw_value: s.to_string(),
        })
    }
}

#[allow(unused)]
impl Meldung {
    pub fn id(&self) -> Option<String> {
        let re = Regex::new(r#"Meldung_ID="(?<meldung_id>(.*?))""#).unwrap();

        if re.is_match(&self.raw_value) {
            let caps = re.captures(&self.raw_value).unwrap();
            return Some(caps["meldung_id"].to_string());
        }

        None
    }

    pub fn icd10(&self) -> Option<String> {
        let re = Regex::new(r"(?s)<Primaertumor_ICD_Code>(?<icd10>(.*?))</Primaertumor_ICD_Code>")
            .unwrap();

        if re.is_match(&self.raw_value) {
            let caps = re.captures(&self.raw_value).unwrap();
            return Some(caps["icd10"].to_string());
        }

        None
    }

    pub fn database_id(&self) -> Option<String> {
        match self.id() {
            Some(id) => to_database_id(&id),
            _ => None,
        }
    }

    pub fn sanitized_xml_string(&self) -> String {
        let re = Regex::new(r"[\r|\n]+\s*").unwrap();
        let content = re.replace_all(&self.raw_value, "").trim().to_string();

        let re = Regex::new(r"<[^>]+/>").unwrap();
        if re.is_match(&content) {
            let mut c = content.to_string();
            re.find_iter(&content)
                .map(|m| m.as_str().to_string().replace('<', "").replace("/>", ""))
                .for_each(|tag| {
                    c = c.replace(&format!("<{}/>", tag), &format!("<{}></{}>", tag, tag));
                });
            return c;
        }

        content
    }
}

pub fn to_database_id(id: &str) -> Option<String> {
    let re1 = Regex::new(r"^(?<id>[0-9A-F]+)").unwrap();
    let re2 = Regex::new(r"(?<id>[0-9]+)$").unwrap();

    if re1.is_match(id) {
        match re1.find(id).map(|m| m.as_str().to_string()) {
            Some(val) => match u64::from_str_radix(&val, 16) {
                Ok(val) => Some(val.to_string()),
                _ => None,
            },
            _ => None,
        }
    } else if re2.is_match(id) {
        re2.find(id).map(|m| m.as_str().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::lkrexport::{LkrExportProtocolFile, Meldung};

    #[test]
    fn should_read_xml_file_content() {
        let actual = LkrExportProtocolFile::parse(include_str!("../testdaten/testdaten_1.xml"));

        assert!(actual.is_ok());
        assert_eq!(actual.unwrap().patients.len(), 2);
    }

    #[test]
    fn should_get_meldungen() {
        let actual = LkrExportProtocolFile::parse(include_str!("../testdaten/testdaten_1.xml"));

        assert!(actual.is_ok());

        let patients = actual.unwrap().patients;

        assert_eq!(patients[0].meldungen().len(), 1);
        assert_eq!(patients[1].meldungen().len(), 1);
    }

    #[test]
    fn should_get_meldung_id() {
        let actual = LkrExportProtocolFile::parse(include_str!("../testdaten/testdaten_1.xml"));

        assert!(actual.is_ok());

        let patients = actual.unwrap().patients;

        assert_eq!(
            patients[0].meldungen()[0].id(),
            Some("TEST1727528".to_string())
        );
        assert_eq!(
            patients[1].meldungen()[0].id(),
            Some("001A5D50-TEST".to_string())
        );
    }

    #[test]
    fn should_get_meldung_database_id() {
        let actual = LkrExportProtocolFile::parse(include_str!("../testdaten/testdaten_1.xml"));

        assert!(actual.is_ok());

        let patients = actual.unwrap().patients;

        assert_eq!(
            patients[0].meldungen()[0].database_id(),
            Some("1727528".to_string())
        );
        assert_eq!(
            patients[1].meldungen()[0].database_id(),
            Some("1727824".to_string())
        );
    }

    #[test]
    fn should_get_meldung_icd10() {
        let actual = LkrExportProtocolFile::parse(include_str!("../testdaten/testdaten_1.xml"));

        assert!(actual.is_ok());

        let patients = actual.unwrap().patients;

        assert_eq!(
            patients[0].meldungen()[0].icd10(),
            Some("C17.1".to_string())
        );
        assert_eq!(
            patients[1].meldungen()[0].icd10(),
            Some("C17.2".to_string())
        );
    }

    #[test]
    fn should_get_meldung_with_trimmed_margin() {
        let meldung = Meldung {
            raw_value: "  <Test>\n  <Test2>TestInhalt 3</Test2>\n</Test>\n".into(),
        };

        assert_eq!(
            meldung.sanitized_xml_string(),
            "<Test><Test2>TestInhalt 3</Test2></Test>".to_string()
        );
    }

    #[test]
    fn should_get_meldung_without_self_closing_tags() {
        let meldung = Meldung {
            raw_value:
                "  <Test>\n  <Test2/>\n  <Content>Test</Content>\n  <Test3/>\n  <Test2/>\n</Test>\n"
                    .into(),
        };

        assert_eq!(
            meldung.sanitized_xml_string(),
            "<Test><Test2></Test2><Content>Test</Content><Test3></Test3><Test2></Test2></Test>"
                .to_string()
        );
    }
}
