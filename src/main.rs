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

use crate::cli::{Cli, SubCommand};
use crate::common::Icd10GroupSize;
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
        }
    }

    Ok(())
}
