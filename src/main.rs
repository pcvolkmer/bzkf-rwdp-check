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

use crate::cli::{Cli, SubCommand};

mod opal;
mod common;
mod cli;

fn main() -> Result<(), Box<dyn Error>> {
    let term = Term::stdout();

    match Cli::parse().cmd {
        SubCommand::OpalFile { file } => {
            let items = opal::OpalCsvFile::check(Path::new(&file))
                .map_err(|_e| "Kann Datei nicht lesen")?;

            let _ = term.write_line(&style("Anzahl der Conditions nach ICD-Gruppe").yellow().to_string());
            items.iter().for_each(|item| {
                let _ = term.write_line(&format!("{:<20}={:>6}", item.name, item.size));
            });
        },
        SubCommand::Database { .. } => {
            todo!("Not implemented yet")
        }
    }

    Ok(())
}
