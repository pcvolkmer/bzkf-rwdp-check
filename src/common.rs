/*
 * This file is part of bzkf-rwdp-check
 *
 * Copyright (C) 2024 Comprehensive Cancer Center Mainfranken and contributors.
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

use itertools::Itertools;
use serde::{Deserialize, Serialize};

pub struct Icd10GroupSize {
    pub name: String,
    pub size: usize,
}

pub struct Record {
    pub condition_id: String,
    pub icd10_code: String,
}

pub struct DiffRecord {
    pub pat_id: Option<String>,
    pub condition_id: String,
    pub diagnosis_date: String,
    pub csv_icd10_code: String,
    pub db_icd10_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportData {
    #[serde(rename = "pat_id")]
    pub pat_id: Option<String>,
    #[serde(rename = "cond_id")]
    pub condition_id: String,
    #[serde(rename = "conditiondate")]
    pub diagnosis_date: String,
    #[serde(rename = "condcodingcode")]
    pub icd_10_code: String,
}

pub struct Check;

impl Check {
    /// Collects all records by grouping by ICD10 group
    pub fn collect(records: &[Record]) -> Result<Vec<Icd10GroupSize>, ()> {
        let items = records
            .iter()
            .map(|record| Record {
                condition_id: record.condition_id.to_string(),
                icd10_code: Self::map_icd_code(&record.icd10_code),
            })
            .sorted_by_key(|record| record.icd10_code.to_string())
            .group_by(|record| record.icd10_code.to_string())
            .into_iter()
            .map(|(icd10, group)| (icd10, group.collect::<Vec<_>>()))
            .map(|record| Icd10GroupSize {
                name: record.0,
                size: record.1.iter().count(),
            })
            .collect::<Vec<_>>();

        Ok(items)
    }

    pub fn is_relevant(code: &str) -> bool {
        match Self::map_icd_code(code).as_str() {
            "Other" => false,
            _ => true,
        }
    }

    fn map_icd_code(code: &str) -> String {
        let icd10 = match code {
            "D39.1" | "D09.0" | "D41.4" => code,
            _ => code.split('.').collect::<Vec<_>>().first().unwrap(),
        };

        match icd10 {
            "C00" | "C01" | "C02" | "C03" | "C04" | "C05" | "C06" | "C07" | "C08" | "C09"
            | "C10" | "C11" | "C12" | "C13" | "C14" => "C00-C14",
            "C15" => "C15",
            "C16" => "C16",
            "C18" | "C19" | "C20" | "C21" => "C18-C21",
            "C22" => "C22",
            "C23" | "C24" => "C23-C24",
            "C25" => "C25",
            "C32" => "C32",
            "C33" | "C34" => "C33-C34",
            "C43" => "C43",
            "C50" | "D05" => "C50, D05",
            "C53" | "D06" => "C53, D06",
            "C54" | "C55" => "C54-C55",
            "C56" | "D39.1" => "C56, D39.1",
            "C61" => "C61",
            "C62" => "C62",
            "C64" => "C64",
            "C67" | "D09.0" | "D41.4" => "C67, D09.0, D41.4",
            "C70" | "C71" | "C72" => "C70-C72",
            "C73" => "C73",
            "C81" => "C81",
            "C82" | "C83" | "C84" | "C85" | "C86" | "C87" | "C88" | "C96" => "C82-C88, C96",
            "C90" => "C90",
            "C91" | "C92" | "C93" | "C94" | "C95" => "C91-C95",
            _ => "Other",
        }
        .to_string()
    }
}
