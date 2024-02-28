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

use std::error::Error;
use std::path::Path;

use clap::Parser;
use console::{style, Term};
use csv::Writer;
use itertools::Itertools;

use crate::cli::{Cli, SubCommand};
use crate::common::{Check, DiffRecord, Icd10GroupSize};
use crate::database::DatabaseSource;

mod cli;
mod common;
mod database;
mod opal;
mod resources;

fn print_items(items: &[Icd10GroupSize]) {
    let term = Term::stdout();
    let _ = term.write_line(
        &style("Anzahl der Conditions nach ICD-10-Gruppe")
            .yellow()
            .to_string(),
    );
    items.iter().for_each(|item| {
        let _ = term.write_line(&format!("{:<20}={:>6}", item.name, item.size));
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    let term = Term::stdout();

    match Cli::parse().cmd {
        SubCommand::OpalFile { file } => {
            let items = opal::OpalCsvFile::check(Path::new(&file))
                .map_err(|_e| "Kann Datei nicht lesen")?;

            print_items(&items);
        }
        SubCommand::Database {
            database,
            host,
            password,
            port,
            user,
            year,
        } => {
            let password = if let Some(password) = password {
                password
            } else {
                let password = dialoguer::Password::new()
                    .with_prompt("Password")
                    .interact()
                    .unwrap_or_default();
                let _ = term.clear_last_lines(1);
                password
            };

            let year = if year.len() == 4 {
                year
            } else {
                format!("2{:0>3}", year)
            };

            let _ = term.write_line(
                &style(format!("Warte auf Daten für das Diagnosejahr {}...", year))
                    .blue()
                    .to_string(),
            );

            let db = DatabaseSource::new(&database, &host, &password, port, &user);
            let items = db
                .check(&year)
                .map_err(|_e| "Fehler bei Zugriff auf die Datenbank")?;

            let _ = term.clear_last_lines(1);

            print_items(&items);
        }
        SubCommand::Export {
            pat_id,
            database,
            host,
            password,
            port,
            user,
            output,
            year,
        } => {
            let password = if let Some(password) = password {
                password
            } else {
                let password = dialoguer::Password::new()
                    .with_prompt("Password")
                    .interact()
                    .unwrap_or_default();
                let _ = term.clear_last_lines(1);
                password
            };

            let year = if year.len() == 4 {
                year
            } else {
                format!("2{:0>3}", year)
            };

            let _ = term.write_line(
                &style(format!("Warte auf Daten für das Diagnosejahr {}...", year))
                    .blue()
                    .to_string(),
            );

            let db = DatabaseSource::new(&database, &host, &password, port, &user);
            let items = db
                .export(&year, pat_id)
                .map_err(|_e| "Fehler bei Zugriff auf die Datenbank")?;

            let _ = term.clear_last_lines(1);

            let mut writer = Writer::from_path(Path::new(&output)).expect("writeable file");

            items
                .iter()
                .for_each(|item| writer.serialize(item).unwrap());

            let _ = term.write_line(
                &style(format!(
                    "{} Conditions für das Jahr {} in Datei '{}' exportiert",
                    items.len(),
                    year,
                    output
                ))
                .green()
                .to_string(),
            );
        }
        SubCommand::Compare {
            database,
            host,
            password,
            port,
            user,
            file,
            year,
        } => {
            let password = if let Some(password) = password {
                password
            } else {
                let password = dialoguer::Password::new()
                    .with_prompt("Password")
                    .interact()
                    .unwrap_or_default();
                let _ = term.clear_last_lines(1);
                password
            };

            let year = if year.len() == 4 {
                year
            } else {
                format!("2{:0>3}", year)
            };

            let _ = term.write_line(
                &style(format!("Warte auf Daten für das Diagnosejahr {}...", year))
                    .blue()
                    .to_string(),
            );

            let db = DatabaseSource::new(&database, &host, &password, port, &user);
            let db_items = db
                .export(&year, false)
                .map_err(|_e| "Fehler bei Zugriff auf die Datenbank")?;

            let _ = term.clear_last_lines(1);

            let csv_items = opal::OpalCsvFile::export(Path::new(&file))
                .map_err(|_e| "Kann Datei nicht lesen")?;

            let mut not_in_csv = db_items
                .iter()
                .filter(|db_item| {
                    !csv_items
                        .iter()
                        .map(|csv_item| &csv_item.condition_id)
                        .contains(&db_item.condition_id)
                })
                .collect::<Vec<_>>();

            let _ = term.write_line(
                &style(format!(
                    "{} Conditions aus der Datenbank für das Jahr {} - aber nicht in Datei '{}'",
                    not_in_csv.len(),
                    year,
                    file
                ))
                .green()
                .to_string(),
            );

            let _ = term.write_line(&format!("{:<64}   {:<5}", "Condition-ID", "ICD10"));

            not_in_csv.sort_by_key(|item| item.condition_id.to_string());

            not_in_csv
                .iter()
                .for_each(|item| match Check::is_relevant(&item.icd_10_code) {
                    true => {
                        let _ = term.write_line(&format!(
                            "{}   {:<5}",
                            item.condition_id,
                            style(&item.icd_10_code).bold().red()
                        ));
                    }
                    false => {
                        let _ = term.write_line(&format!(
                            "{}   {:<5}",
                            item.condition_id, item.icd_10_code
                        ));
                    }
                });

            let mut not_in_db = csv_items
                .iter()
                .filter(|csv_item| {
                    !db_items
                        .iter()
                        .map(|db_item| &db_item.condition_id)
                        .contains(&csv_item.condition_id)
                })
                .collect::<Vec<_>>();

            let _ = term.write_line(
                &style(format!(
                    "{} Conditions aus Datei '{}' - aber nicht in der Datenbank für das Jahr {}",
                    not_in_db.len(),
                    file,
                    year
                ))
                .green()
                .to_string(),
            );

            let _ = term.write_line(&format!("{:<64}   {:<5}", "Condition-ID", "ICD10"));

            not_in_db.sort_by_key(|item| item.condition_id.to_string());

            not_in_db
                .iter()
                .for_each(|item| match Check::is_relevant(&item.icd_10_code) {
                    true => {
                        let _ = term.write_line(&format!(
                            "{}   {:<5}",
                            item.condition_id,
                            style(&item.icd_10_code).bold().red()
                        ));
                    }
                    false => {
                        let _ = term.write_line(&format!(
                            "{}   {:<5}",
                            item.condition_id, item.icd_10_code
                        ));
                    }
                });

            let mut icd10diff = csv_items
                .iter()
                .filter(|csv_item| {
                    db_items
                        .iter()
                        .map(|db_item| &db_item.condition_id)
                        .contains(&csv_item.condition_id)
                })
                .filter(|csv_item| {
                    !db_items
                        .iter()
                        .map(|db_item| format!("{}-{}", db_item.condition_id, db_item.icd_10_code))
                        .contains(&format!(
                            "{}-{}",
                            csv_item.condition_id, csv_item.icd_10_code
                        ))
                })
                .map(|csv_item| DiffRecord {
                    condition_id: csv_item.condition_id.to_string(),
                    csv_icd10_code: csv_item.icd_10_code.to_string(),
                    db_icd10_code: db_items
                        .iter()
                        .filter(|db_item| db_item.condition_id == csv_item.condition_id)
                        .collect_vec()
                        .first()
                        .unwrap()
                        .icd_10_code
                        .to_string(),
                })
                .collect::<Vec<_>>();

            let _ = term.write_line(
                &style(format!(
                    "{} Conditions mit Unterschied im ICD10-Code",
                    icd10diff.len()
                ))
                .green()
                .to_string(),
            );

            icd10diff.sort_by_key(|item| item.condition_id.to_string());

            let _ = term.write_line(&format!(
                "{:<64}   {:<5}   {:<5}",
                "Condition-ID", "CSV", "DB"
            ));

            icd10diff.iter().for_each(|item| {
                let _ = term.write_line(&format!(
                    "{}   {}   {}",
                    item.condition_id,
                    match Check::is_relevant(&item.csv_icd10_code) {
                        true => style(format!("{:<5}", item.csv_icd10_code)).bold().red(),
                        _ => style(format!("{:<5}", item.csv_icd10_code)),
                    },
                    match Check::is_relevant(&item.db_icd10_code) {
                        true => style(format!("{:<5}", item.db_icd10_code)).bold().red(),
                        _ => style(format!("{:<5}", item.db_icd10_code)),
                    }
                ));
            });
        }
    }

    Ok(())
}
