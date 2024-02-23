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

use std::path::Path;

use csv::Reader;
use serde::Deserialize;

use crate::common::{Check, Icd10GroupSize, Record};

#[derive(Deserialize)]
pub struct OpalRecord {
    #[serde(rename = "cond_id")]
    cond_id: String,
    #[serde(rename = "condcodingcode")]
    cond_coding_code: String,
}

pub struct OpalCsvFile;

impl OpalCsvFile {
    pub fn check(path: &Path) -> Result<Vec<Icd10GroupSize>, ()> {
        let mut reader = Reader::from_path(path).expect("open file");

        let items = reader.deserialize::<OpalRecord>()
            .filter(|record| record.is_ok())
            .map(|record| record.unwrap())
            .map(|record| Record {
                condition_id: record.cond_id,
                icd10_code: record.cond_coding_code
            })
            .collect::<Vec<_>>();

        Check::collect(&items)
    }

}