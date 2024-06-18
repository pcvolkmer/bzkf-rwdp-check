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

use clap::{Parser, Subcommand};
use regex::Regex;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
#[command(arg_required_else_help(true), disable_help_flag(true))]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    #[command(about = "Ermittelt die Prüfwerte aus einem CSV-File für OPAL")]
    OpalFile {
        #[arg(short, long, help = "CSV-File für Opal")]
        file: PathBuf,
    },
    #[command(about = "Ermittelt die Prüfwerte aus der Onkostar-Datenbank")]
    Database {
        #[arg(short = 'D', long, help = "Datenbank-Name", default_value = "onkostar")]
        database: String,
        #[arg(
            short = 'h',
            long,
            help = "Datenbank-Host",
            default_value = "localhost"
        )]
        host: String,
        #[arg(short = 'P', long, help = "Datenbank-Host", default_value = "3306")]
        port: u16,
        #[arg(
            short = 'p',
            long,
            help = "Passwort. Wenn nicht angegeben, wird danach gefragt"
        )]
        password: Option<String>,
        #[arg(short = 'u', long, help = "Benutzername")]
        user: String,
        #[arg(short = 'y', long, help = "Jahr der Diagnose")]
        year: String,
        #[arg(long, value_parser = value_is_date, help = "Ignoriere LKR-Exporte seit Datum")]
        ignore_exports_since: Option<String>,
        #[arg(long, help = "Meldungen mit externer Diagnose einschließen")]
        include_extern: bool,
        #[arg(
            long,
            help = "Meldungen mit Meldeanlass 'histologie_zytologie' einschließen"
        )]
        include_histo_zyto: bool,
        #[arg(long, help = "Meldungen mit oBDS-Schema-version anzeigen")]
        schema_versions: bool,
    },
    #[command(
        about = "Erstellt eine (reduzierte) CSV-Datei zum direkten Vergleich mit der OPAL-CSV-Datei"
    )]
    Export {
        #[arg(long, help = "Export mit Klartext-Patienten-ID")]
        pat_id: bool,
        #[arg(short = 'D', long, help = "Datenbank-Name", default_value = "onkostar")]
        database: String,
        #[arg(
            short = 'h',
            long,
            help = "Datenbank-Host",
            default_value = "localhost"
        )]
        host: String,
        #[arg(short = 'P', long, help = "Datenbank-Host", default_value = "3306")]
        port: u16,
        #[arg(
            short = 'p',
            long,
            help = "Passwort. Wenn nicht angegeben, wird danach gefragt"
        )]
        password: Option<String>,
        #[arg(short = 'u', long, help = "Benutzername")]
        user: String,
        #[arg(short = 'o', long, help = "Ausgabedatei")]
        output: PathBuf,
        #[arg(short = 'y', long, help = "Jahr der Diagnose")]
        year: String,
        #[arg(long, value_parser = value_is_date, help = "Ignoriere LKR-Exporte seit Datum")]
        ignore_exports_since: Option<String>,
        #[arg(long, help = "Export mit Trennzeichen ';' für Excel")]
        xls_csv: bool,
        #[arg(long, help = "Meldungen mit externer Diagnose einschließen")]
        include_extern: bool,
        #[arg(
            long,
            help = "Meldungen mit Meldeanlass 'histologie_zytologie' einschließen"
        )]
        include_histo_zyto: bool,
    },
    #[command(about = "Abgleich zwischen CSV-Datei für OPAL und Onkostar-Datenbank")]
    Compare {
        #[arg(long, help = "Klartext-Patienten-ID anzeigen")]
        pat_id: bool,
        #[arg(short = 'D', long, help = "Datenbank-Name", default_value = "onkostar")]
        database: String,
        #[arg(
            short = 'h',
            long,
            help = "Datenbank-Host",
            default_value = "localhost"
        )]
        host: String,
        #[arg(short = 'P', long, help = "Datenbank-Host", default_value = "3306")]
        port: u16,
        #[arg(
            short = 'p',
            long,
            help = "Passwort. Wenn nicht angegeben, wird danach gefragt"
        )]
        password: Option<String>,
        #[arg(short = 'u', long, help = "Benutzername")]
        user: String,
        #[arg(short, long, help = "CSV-File für Opal")]
        file: PathBuf,
        #[arg(short = 'y', long, help = "Jahr der Diagnose")]
        year: String,
        #[arg(long, value_parser = value_is_date, help = "Ignoriere LKR-Exporte seit Datum")]
        ignore_exports_since: Option<String>,
        #[arg(long, help = "Meldungen mit externer Diagnose einschließen")]
        include_extern: bool,
        #[arg(
            long,
            help = "Meldungen mit Meldeanlass 'histologie_zytologie' einschließen"
        )]
        include_histo_zyto: bool,
    },
    #[command(about = "Abgleich zwischen LKR-Export-Protokoll und Onkostar-Datenbank")]
    CheckExport {
        #[arg(short = 'D', long, help = "Datenbank-Name", default_value = "onkostar")]
        database: String,
        #[arg(
            short = 'h',
            long,
            help = "Datenbank-Host",
            default_value = "localhost"
        )]
        host: String,
        #[arg(short = 'P', long, help = "Datenbank-Host", default_value = "3306")]
        port: u16,
        #[arg(
            short = 'p',
            long,
            help = "Passwort. Wenn nicht angegeben, wird danach gefragt"
        )]
        password: Option<String>,
        #[arg(short = 'u', long, help = "Benutzername")]
        user: String,
        #[arg(short, long, help = "LKR-Export-Protokoll-Datei")]
        file: PathBuf,
        #[arg(long, alias = "export-package", help = "Exportpaketnummer", default_value = "0")]
        package: u16,
    },
}

fn value_is_date(value: &str) -> Result<String, String> {
    let re = Regex::new(r"^[0-9]{4}-[0-1][0-9]-[0-3][0-9]$").unwrap();
    if re.is_match(value) {
        Ok(value.into())
    } else {
        Err(format!(
            "Ungültiges Datum '{}', bitte im Format 'yyyy-mm-dd' angeben",
            value
        ))
    }
}
