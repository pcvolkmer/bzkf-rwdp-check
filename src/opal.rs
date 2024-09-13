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

use std::path::Path;

use csv::Reader;
use serde::Deserialize;

use crate::common::{Check, ExportData, Icd10GroupSize, Record};

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

        let items = reader
            .deserialize::<OpalRecord>()
            .filter_map(|record| record.ok())
            .map(|record| Record {
                condition_id: record.cond_id,
                icd10_code: record.cond_coding_code,
            })
            .collect::<Vec<_>>();

        Check::collect(&items)
    }

    pub fn export(path: &Path) -> Result<Vec<ExportData>, ()> {
        let mut reader = Reader::from_path(path).expect("open file");

        let items = reader
            .deserialize::<ExportData>()
            .filter_map(|record| record.ok())
            .collect::<Vec<_>>();

        Ok(items)
    }
}
