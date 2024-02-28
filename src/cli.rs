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
        file: String,
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
    },
}
