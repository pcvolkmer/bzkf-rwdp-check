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

pub const SQL_QUERY: &str = include_str!("query.sql");

pub const SQL_QUERY_WITH_SCHEMA_VERSION: &str = include_str!("query_with_schema_version.sql");

pub const EXPORT_QUERY: &str = include_str!("export.sql");

pub const EXPORTED_TO_LKR: &str = include_str!("exported-to-lkr.sql");
