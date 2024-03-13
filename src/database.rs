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

use mysql::prelude::Queryable;
use mysql::{params, Pool};
use std::time::Duration;

use crate::common::{ExportData, Icd10GroupSize};
use crate::resources::{EXPORT_QUERY, SQL_QUERY};

pub struct DatabaseSource(String);

impl DatabaseSource {
    pub fn new(database: &str, host: &str, password: &str, port: u16, user: &str) -> Self {
        let password = urlencoding::encode(password);
        let url = format!("mysql://{user}:{password}@{host}:{port}/{database}");
        DatabaseSource(url)
    }

    pub fn check(&self, year: &str, ignore_exports_since: &str) -> Result<Vec<Icd10GroupSize>, ()> {
        match Pool::new(self.0.as_str()) {
            Ok(pool) => {
                if let Ok(mut connection) = pool.get_conn() {
                    return match connection.exec_map(
                        SQL_QUERY,
                        params! {"year" => year, "ignore_exports_since" => ignore_exports_since},
                        |(icd10_group, count)| Icd10GroupSize {
                            name: icd10_group,
                            size: count,
                        },
                    ) {
                        Ok(result) => Ok(result),
                        Err(_) => Err(()),
                    };
                }
            }
            Err(_) => {
                return Err(());
            }
        }

        Err(())
    }

    pub fn export(
        &self,
        year: &str,
        ignore_exports_since: &str,
        use_pat_id: bool,
    ) -> Result<Vec<ExportData>, ()> {
        match Pool::new(self.0.as_str()) {
            Ok(pool) => {
                if let Ok(mut connection) = pool.get_conn() {
                    return match connection.exec_map(
                        EXPORT_QUERY,
                        params! {"year" => year, "ignore_exports_since" => ignore_exports_since},
                        |(condition_id, icd_10_code, diagnosis_date, pat_id)| ExportData {
                            condition_id,
                            icd_10_code,
                            diagnosis_date,
                            pat_id: if use_pat_id { Some(pat_id) } else { None },
                        },
                    ) {
                        Ok(result) => Ok(result),
                        Err(_) => {
                            return Err(());
                        }
                    };
                }
            }
            Err(_) => {
                return Err(());
            }
        }

        Err(())
    }
}
