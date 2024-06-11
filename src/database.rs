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

use std::time::Duration;

use mysql::prelude::Queryable;
use mysql::{params, Pool};

use crate::common::{ExportData, Icd10GroupSize};
use crate::resources::{EXPORTED_TO_LKR, EXPORT_QUERY, SQL_QUERY};

pub struct DatabaseSource(String);

impl DatabaseSource {
    pub fn new(database: &str, host: &str, password: &str, port: u16, user: &str) -> Self {
        let password = urlencoding::encode(password);
        let url = format!("mysql://{user}:{password}@{host}:{port}/{database}");
        DatabaseSource(url)
    }

    pub fn check(
        &self,
        year: &str,
        ignore_exports_since: &str,
        include_extern: bool,
        include_histo_zyto: bool,
    ) -> Result<Vec<Icd10GroupSize>, ()> {
        match Pool::new(self.0.as_str()) {
            Ok(pool) => {
                if let Ok(mut connection) = pool.try_get_conn(Duration::from_secs(3)) {
                    return match connection.exec_map(
                        SQL_QUERY,
                        params! {
                            "year" => year,
                            "ignore_exports_since" => ignore_exports_since,
                            "include_extern" => if include_extern { 1 } else { 0 },
                            "include_histo_zyto" => if include_histo_zyto { 1 } else { 0 }
                        },
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
        include_extern: bool,
        include_histo_zyto: bool,
    ) -> Result<Vec<ExportData>, ()> {
        match Pool::new(self.0.as_str()) {
            Ok(pool) => {
                if let Ok(mut connection) = pool.try_get_conn(Duration::from_secs(3)) {
                    return match connection.exec_map(
                        EXPORT_QUERY,
                        params! {
                            "year" => year,
                            "ignore_exports_since" => ignore_exports_since,
                            "include_extern" => if include_extern { 1 } else { 0 },
                            "include_histo_zyto" => if include_histo_zyto { 1 } else { 0 }
                        },
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

    pub fn exported(&self, export_id: u16) -> Result<Vec<(String, String)>, ()> {
        match Pool::new(self.0.as_str()) {
            Ok(pool) => {
                if let Ok(mut connection) = pool.try_get_conn(Duration::from_secs(3)) {
                    return match connection.exec_map(
                        EXPORTED_TO_LKR,
                        params! {
                            "export_id" => export_id,
                        },
                        |(id, xml_data)| (id, xml_data),
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
