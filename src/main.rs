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

use std::collections::HashMap;
use std::error::Error;

use clap::Parser;
use console::{style, Term};
use csv::WriterBuilder;
use itertools::Itertools;

use crate::cli::{Cli, SubCommand};
use crate::common::{Check, DiffRecord, Icd10GroupSize};
use crate::database::DatabaseSource;
use crate::lkrexport::{to_database_id, LkrExportProtocolFile, Meldung};

mod cli;
mod common;
mod database;
mod lkrexport;
mod opal;
mod resources;

fn request_password_if_none(password: Option<String>) -> String {
    if let Some(password) = password {
        password
    } else {
        let password = dialoguer::Password::new()
            .with_prompt("Password")
            .interact()
            .unwrap_or_default();
        let _ = Term::stdout().clear_last_lines(1);
        password
    }
}

fn sanitize_year(year: &str) -> String {
    if year.len() == 4 {
        year.to_string()
    } else {
        format!("2{:0>3}", year)
    }
}

fn print_items(items: &[Icd10GroupSize]) {
    let term = Term::stdout();
    let _ = term.write_line(
        &style("Anzahl der Conditions nach ICD-10-Gruppe")
            .yellow()
            .to_string(),
    );
    items.iter().for_each(|item| {
        let _ = term.write_line(&format!(
            "{:<20} {:<6} ={:>6}",
            item.name,
            item.schema_version.as_ref().unwrap_or(&String::new()),
            item.size
        ));
    });
    let sum: usize = items
        .iter()
        .filter(|item| item.name != "Other")
        .map(|item| item.size)
        .sum();
    let _ = term.write_line(&style("─".repeat(35)).dim().to_string());
    let _ = term.write_line(
        &style(format!(
            "{:<20} {:<6} ={:>6}",
            "Summe (C**.*/D**.*)", "", sum
        ))
        .dim()
        .to_string(),
    );
    let sum: usize = items.iter().map(|item| item.size).sum();
    let _ = term.write_line(
        &style(format!("{:<20} {:<6} ={:>6}", "Gesamtsumme", "", sum))
            .dim()
            .to_string(),
    );
    let _ = term.write_line(&style("─".repeat(35)).dim().to_string());
}

fn print_extern_notice(include_extern: bool) {
    let _ = Term::stdout().write_line(
        format!(
            "{} Die Datenbankanfrage schließt Meldungen mit externer Diagnose {}.",
            style("Hinweis:").bold().underlined(),
            match include_extern {
                true => style("ein").yellow(),
                false => style("nicht ein (Standard)").green(),
            }
        )
        .as_str(),
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let term = Term::stdout();

    match Cli::parse().cmd {
        SubCommand::OpalFile { file } => {
            let items =
                opal::OpalCsvFile::check(file.as_path()).map_err(|_e| "Kann Datei nicht lesen")?;

            print_items(&items);
        }
        SubCommand::Database {
            database,
            host,
            password,
            port,
            user,
            year,
            ignore_exports_since,
            ignore_non_obds_2,
            include_extern,
            include_histo_zyto,
            schema_versions,
        } => {
            let password = request_password_if_none(password);
            let year = sanitize_year(&year);

            let _ = term.write_line(
                &style(format!("Warte auf Daten für das Diagnosejahr {}...", year))
                    .blue()
                    .bright()
                    .to_string(),
            );

            let db = DatabaseSource::new(&database, &host, &password, port, &user);
            let items = db
                .check(
                    &year,
                    &ignore_exports_since.unwrap_or("9999-12-31".into()),
                    ignore_non_obds_2,
                    include_extern,
                    include_histo_zyto,
                    schema_versions,
                )
                .map_err(|_e| "Fehler bei Zugriff auf die Datenbank")?;

            let _ = term.clear_last_lines(1);

            print_extern_notice(include_extern);
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
            ignore_exports_since,
            ignore_non_obds_2,
            xls_csv,
            include_extern,
            include_histo_zyto,
        } => {
            let password = request_password_if_none(password);
            let year = sanitize_year(&year);

            let _ = term.write_line(
                &style(format!("Warte auf Daten für das Diagnosejahr {}...", year))
                    .blue()
                    .bright()
                    .to_string(),
            );

            let db = DatabaseSource::new(&database, &host, &password, port, &user);
            let items = db
                .export(
                    &year,
                    &ignore_exports_since.unwrap_or("9999-12-31".into()),
                    ignore_non_obds_2,
                    pat_id,
                    include_extern,
                    include_histo_zyto,
                )
                .map_err(|_e| "Fehler bei Zugriff auf die Datenbank")?;

            let _ = term.clear_last_lines(1);

            let writer_builder = &mut WriterBuilder::new();
            let mut writer_builder = writer_builder.has_headers(true);
            if xls_csv {
                writer_builder = writer_builder.delimiter(b';');
            }
            let mut writer = writer_builder
                .from_path(output.as_path())
                .expect("writeable file");

            items
                .iter()
                .for_each(|item| writer.serialize(item).unwrap());

            let _ = term.write_line(
                &style(format!(
                    "{} Conditions für das Jahr {} in Datei '{}' exportiert",
                    items.len(),
                    year,
                    output.to_str().unwrap_or_default()
                ))
                .green()
                .to_string(),
            );

            print_extern_notice(include_extern);
        }
        SubCommand::Compare {
            pat_id,
            database,
            host,
            password,
            port,
            user,
            file,
            year,
            ignore_exports_since,
            ignore_non_obds_2,
            include_extern,
            include_histo_zyto,
        } => {
            let password = request_password_if_none(password);
            let year = sanitize_year(&year);

            let _ = term.write_line(
                &style(format!("Warte auf Daten für das Diagnosejahr {}...", year))
                    .blue()
                    .bright()
                    .to_string(),
            );

            let db = DatabaseSource::new(&database, &host, &password, port, &user);
            let db_items = db
                .export(
                    &year,
                    &ignore_exports_since.unwrap_or("9999-12-31".into()),
                    ignore_non_obds_2,
                    pat_id,
                    include_extern,
                    include_histo_zyto,
                )
                .map_err(|_e| "Fehler bei Zugriff auf die Datenbank")?;

            let _ = term.clear_last_lines(1);

            let csv_items =
                opal::OpalCsvFile::export(file.as_path()).map_err(|_e| "Kann Datei nicht lesen")?;

            let mut not_in_csv = db_items
                .iter()
                .filter(|&db_item| {
                    !csv_items
                        .iter()
                        .map(|csv_item| &csv_item.condition_id)
                        .contains(&db_item.condition_id)
                })
                .collect::<Vec<_>>();

            print_extern_notice(include_extern);

            let _ = term.write_line(
                &style(format!(
                    "{} Conditions aus der Datenbank für das Jahr {} - aber nicht in Datei '{}'",
                    not_in_csv.len(),
                    year,
                    file.to_str().unwrap_or_default()
                ))
                .green()
                .to_string(),
            );

            let _ = term.write_line(&format!(
                "{:<64}   {:<10}   {:<5}   {:<5}   {}",
                "Condition-ID", "Datum", "ICD10", "", "PAT-ID"
            ));

            not_in_csv.sort_by_key(|item| item.condition_id.to_string());

            not_in_csv
                .iter()
                .for_each(|&item| match Check::is_relevant(&item.icd_10_code) {
                    true => {
                        let _ = term.write_line(&format!(
                            "{:<64}   {:<10}   {:<5}   {:<5}   {}",
                            item.condition_id,
                            item.diagnosis_date,
                            style(&item.icd_10_code).bold().red(),
                            "",
                            match &item.pat_id {
                                Some(ref pat_id) => pat_id.to_string(),
                                _ => "".to_string(),
                            }
                        ));
                    }
                    false => {
                        let _ = term.write_line(&format!(
                            "{:<64}   {:<10}   {:<5}   {:<5}   {}",
                            item.condition_id,
                            item.diagnosis_date,
                            item.icd_10_code,
                            "",
                            match &item.pat_id {
                                Some(ref pat_id) => pat_id.to_string(),
                                _ => "".to_string(),
                            }
                        ));
                    }
                });

            let mut not_in_db = csv_items
                .iter()
                .filter(|&csv_item| {
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
                    file.to_str().unwrap_or_default(),
                    year
                ))
                .green()
                .to_string(),
            );

            let _ = term.write_line(&format!(
                "{:<64}   {:<10}   {:<5}",
                "Condition-ID", "Datum", "ICD10"
            ));

            not_in_db.sort_by_key(|item| item.condition_id.to_string());

            not_in_db
                .iter()
                .for_each(|item| match Check::is_relevant(&item.icd_10_code) {
                    true => {
                        let _ = term.write_line(&format!(
                            "{:<64}   {:<10}   {:<5}",
                            item.condition_id,
                            item.diagnosis_date,
                            style(&item.icd_10_code).bold().red()
                        ));
                    }
                    false => {
                        let _ = term.write_line(&format!(
                            "{:<64}   {:<10}   {:<5}",
                            item.condition_id, item.diagnosis_date, item.icd_10_code
                        ));
                    }
                });

            let mut icd10diff = db_items
                .iter()
                .filter(|db_item| {
                    csv_items
                        .iter()
                        .map(|db_item| &db_item.condition_id)
                        .contains(&db_item.condition_id)
                })
                .filter(|db_item| {
                    !csv_items
                        .iter()
                        .map(|csv_item| {
                            format!("{}-{}", csv_item.condition_id, csv_item.icd_10_code)
                        })
                        .contains(&format!("{}-{}", db_item.condition_id, db_item.icd_10_code))
                })
                .map(|db_item| DiffRecord {
                    pat_id: db_item.pat_id.as_ref().map(|pat_id| pat_id.to_string()),
                    condition_id: db_item.condition_id.to_string(),
                    diagnosis_date: db_item.diagnosis_date.to_string(),
                    csv_icd10_code: db_item.icd_10_code.to_string(),
                    db_icd10_code: csv_items
                        .iter()
                        .filter(|csv_item| csv_item.condition_id == db_item.condition_id)
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
                "{:<64}   {:<10}   {:<5}   {:<5}   {}",
                "Condition-ID", "Datum", "CSV", "DB", "PAT-ID"
            ));

            icd10diff.iter().for_each(|item| {
                let _ = term.write_line(&format!(
                    "{:<64}   {:<10}   {:<5}   {:<5}   {}",
                    item.condition_id,
                    item.diagnosis_date,
                    match Check::is_relevant(&item.csv_icd10_code) {
                        true => style(format!("{:<5}", item.csv_icd10_code)).bold().red(),
                        _ => style(format!("{:<5}", item.csv_icd10_code)),
                    },
                    match Check::is_relevant(&item.db_icd10_code) {
                        true => style(format!("{:<5}", item.db_icd10_code)).bold().red(),
                        _ => style(format!("{:<5}", item.db_icd10_code)),
                    },
                    match &item.pat_id {
                        Some(ref pat_id) => pat_id.to_string(),
                        _ => "".to_string(),
                    }
                ));
            });
        }
        SubCommand::CheckExport {
            database,
            host,
            password,
            port,
            user,
            file,
            package,
        } => {
            let password = request_password_if_none(password);

            let _ = term.write_line(
                &style(format!(
                    "Warte auf Daten für den LKR-Export '{}'...",
                    package
                ))
                .blue()
                .bright()
                .to_string(),
            );

            let db = DatabaseSource::new(&database, &host, &password, port, &user);

            let db_entries = db
                .exported(package)
                .map_err(|_e| "Fehler bei Zugriff auf die Datenbank")?;

            let db_meldungen = db_entries
                .iter()
                .map(|entry| LkrExportProtocolFile::parse(&entry.1))
                .filter(|entry| entry.is_ok())
                .flat_map(|entry| entry.unwrap().meldungen())
                .filter(|meldung| meldung.id().is_some())
                .map(|meldung| (meldung.id().unwrap(), meldung))
                .collect::<HashMap<_, _>>();

            let xml_meldungen = LkrExportProtocolFile::parse_file(file.as_path())
                .map_err(|_e| "Fehler bei Zugriff auf die Protokolldatei")?
                .meldungen()
                .into_iter()
                .filter(|meldung| meldung.id().is_some())
                .map(|meldung| (meldung.id().unwrap(), meldung))
                .collect::<HashMap<_, _>>();

            let missing_xml_ids = db_meldungen
                .keys()
                .filter(|&key| !xml_meldungen.contains_key(key))
                .collect_vec();

            let _ = term.clear_last_lines(1);

            let _ = term.write_line(
                &style(format!(
                    "{} Datenbankeinträge mit {} Meldungen abgerufen",
                    db_entries.len(),
                    db_meldungen.len()
                ))
                .green()
                .to_string(),
            );

            fn print_missing_ids(missing_ids: &[&String], term: &Term) {
                missing_ids.iter().sorted().for_each(|&item| {
                    let _ = term.write_line(&format!(
                        "{} ({})",
                        item,
                        to_database_id(item).unwrap_or("?".into())
                    ));
                });
            }

            if db_meldungen.len() != xml_meldungen.len() {
                let _ = term.write_line(
                    &style("\nNicht übereinstimmende Anzahl an Meldungen:")
                        .yellow()
                        .to_string(),
                );
                let _ = term.write_line(&format!(
                    "Datenbank:      {:>10}\nProtokolldatei: {:>10}",
                    db_meldungen.len(),
                    xml_meldungen.len()
                ));

                let missing_db_ids = xml_meldungen
                    .keys()
                    .filter(|&key| !db_meldungen.contains_key(key))
                    .collect_vec();

                if !missing_db_ids.is_empty() {
                    let _ = term.write_line(
                        &style("\nIn der Datenbank fehlende Meldungen:")
                            .yellow()
                            .to_string(),
                    );

                    print_missing_ids(&missing_db_ids, &term);
                }

                if !missing_xml_ids.is_empty() {
                    let _ = term.write_line(
                        &style("\nIn der Protokolldatei fehlende Meldungen:")
                            .yellow()
                            .to_string(),
                    );

                    print_missing_ids(&missing_xml_ids, &term);
                }
            }

            let multiple_meldung_entries = db_entries
                .iter()
                .map(|(lkr_meldung, meldung)| (lkr_meldung, LkrExportProtocolFile::parse(meldung)))
                .filter_map(|(lkr_meldung, meldung)| {
                    if meldung.unwrap().meldungen().len() > 1 {
                        Some(lkr_meldung)
                    } else {
                        None
                    }
                })
                .sorted()
                .collect_vec();

            if !multiple_meldung_entries.is_empty() {
                let _ = term.write_line(
                    &style("\nFolgende Einträge in `lkr_meldung_export` haben mehrere Meldungsinhalte in `xml_daten`:")
                        .yellow()
                        .to_string(),
                );

                multiple_meldung_entries.iter().for_each(|&item| {
                    let _ = term.write_line(&item.to_string());
                });
            }

            let different_content = db_meldungen
                .iter()
                .filter(|(id, _)| !missing_xml_ids.contains(id))
                .filter(|(id, meldung)| {
                    xml_meldungen
                        .get(&id.to_string())
                        .unwrap_or(&Meldung {
                            raw_value: String::new(),
                        })
                        .sanitized_xml_string()
                        != meldung.sanitized_xml_string()
                })
                .map(|(_, meldung)| meldung.id().unwrap_or("?".into()))
                .collect_vec();

            if !different_content.is_empty() {
                let _ = term.write_line(
                    &style(&format!(
                        "\nFolgende {} Meldungen unterscheiden sich in der Datenbank und der Protokolldatei:",
                        different_content.len()
                    ))
                    .yellow()
                    .to_string(),
                );

                let _ = term.write_line(
                    "Dies kann auch aufgrund der verwendeten XML-Encodierung auftreten und bedeutet nicht immer eine inhaltliche Abweichung."
                );

                different_content
                    .iter()
                    .sorted_by(|&id1, &id2| {
                        to_database_id(id1)
                            .unwrap_or_default()
                            .cmp(&to_database_id(id2).unwrap_or_default())
                    })
                    .for_each(|id| {
                        let _ = term.write_line(&format!(
                            "{} ({})",
                            id,
                            to_database_id(id).unwrap_or("?".into())
                        ));
                    });
            }
        }
    }

    Ok(())
}
