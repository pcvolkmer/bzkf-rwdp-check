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

use crate::common::Icd10GroupSize;
use crate::resources::SQL_QUERY;

pub struct DatabaseSource(String);

impl DatabaseSource {
    pub fn new(database: &str, host: &str, password: &str, port: u16, user: &str) -> Self {
        let password = urlencoding::encode(password);
        let url = format!("mysql://{user}:{password}@{host}:{port}/{database}");
        DatabaseSource(url)
    }

    pub fn check(&self, year: &str) -> Result<Vec<Icd10GroupSize>, ()> {
        match Pool::new(self.0.as_str()) {
            Ok(pool) => {
                if let Ok(mut connection) = pool.get_conn() {
                    return match connection.exec_map(
                        SQL_QUERY,
                        params! {"year" => year},
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
            Err(e) => {
                println!("{}", e);
                return Err(());
            }
        }

        Err(())
    }
}
